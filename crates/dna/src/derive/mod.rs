//! Property path system for addressing nested DNA values.
//!
//! A [`PropertyPath`] is a dot-separated (and bracket-indexed) path such as
//! `"transform.position.x"` or `"vertices[3].normal"`. It can be used to
//! read and write individual leaf values inside a [`DnaValue`] tree without
//! knowing the full structure at compile time.
//!
//! This is the backbone of the editor's property inspector, keyframe
//! animation curves, and driver expressions.

use crate::schema::DnaValue;

/// A parsed property path that can be used to get / set values inside a
/// [`DnaValue`] tree.
///
/// # Syntax
///
/// ```text
/// "field"                         — top-level field
/// "field.subfield"                — nested struct field
/// "array[0]"                      — array element by index
/// "array[0].x"                    — field of an array element
/// "nested.array[2].name"          — deeply nested
/// "bones[\"Bone\"].location.x"    — bracket-quoted field name (Blender-style)
/// "bones['Bone'].location.x"     — single-quoted bracket field name
/// ```
///
/// # Example
///
/// ```
/// use forge3d_dna::{PropertyPath, DnaValue};
/// use indexmap::IndexMap;
///
/// let mut root = DnaValue::Struct({
///     let mut m = IndexMap::new();
///     m.insert("x".into(), DnaValue::F32(1.0));
///     m.insert("y".into(), DnaValue::F32(2.0));
///     m
/// });
///
/// let path = PropertyPath::parse("x");
/// assert_eq!(path.get(&root), Some(&DnaValue::F32(1.0)));
///
/// path.set(&mut root, DnaValue::F32(10.0)).unwrap();
/// assert_eq!(path.get(&root), Some(&DnaValue::F32(10.0)));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PropertyPath {
    segments: Vec<Segment>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Segment {
    /// A named struct field.
    Field(String),
    /// A numeric array index.
    Index(usize),
}

/// Errors that can occur when setting a value via a property path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PropertyPathError {
    /// A field name was not found in the struct.
    FieldNotFound(String),
    /// An array index was out of bounds.
    IndexOutOfBounds { index: usize, len: usize },
    /// The path tried to descend into a leaf / incompatible value.
    TypeMismatch(String),
    /// The path was empty.
    EmptyPath,
}

impl std::fmt::Display for PropertyPathError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FieldNotFound(name) => write!(f, "field not found: {name}"),
            Self::IndexOutOfBounds { index, len } => {
                write!(f, "index {index} out of bounds (len {len})")
            }
            Self::TypeMismatch(msg) => write!(f, "type mismatch: {msg}"),
            Self::EmptyPath => write!(f, "empty property path"),
        }
    }
}

impl std::error::Error for PropertyPathError {}

impl PropertyPath {
    /// Parses a dot / bracket path string into a [`PropertyPath`].
    ///
    /// Parsing is lenient: empty segments are skipped, and whitespace around
    /// segments is trimmed.
    ///
    /// Bracket notation supports:
    /// - Numeric indices: `array[0]`
    /// - Quoted field names: `bones["Bone"]` or `bones['Bone']` (Blender-style)
    pub fn parse(path: &str) -> Self {
        let mut segments = Vec::new();
        let mut current = String::new();
        let mut in_bracket = false;
        let mut in_quote: Option<char> = None;
        let mut escape_next = false;

        for ch in path.chars() {
            // If the previous character was a backslash inside quotes,
            // push this character verbatim regardless of what it is.
            if escape_next {
                current.push(ch);
                escape_next = false;
                continue;
            }

            match (ch, in_bracket, in_quote) {
                // Handle escape sequences inside quotes.
                ('\\', true, Some(_)) => {
                    escape_next = true;
                }
                // End of a quoted string inside brackets.
                (q, true, Some(quote_char)) if q == quote_char => {
                    in_quote = None;
                }
                // Any character inside a quoted string.
                (c, true, Some(_)) => {
                    current.push(c);
                }
                // Start of a quoted string inside brackets.
                ('"' | '\'', true, None) => {
                    in_quote = Some(ch);
                }
                // Dot separator (outside brackets).
                ('.', false, _) => {
                    Self::flush_segment(&mut current, &mut segments);
                }
                // Opening bracket.
                ('[', false, _) => {
                    Self::flush_segment(&mut current, &mut segments);
                    in_bracket = true;
                }
                // Closing bracket.
                (']', true, None) => {
                    let trimmed = current.trim();
                    if let Ok(idx) = trimmed.parse::<usize>() {
                        segments.push(Segment::Index(idx));
                    } else if !trimmed.is_empty() {
                        // Bracket-quoted field name (e.g., bones["Bone"]).
                        segments.push(Segment::Field(trimmed.to_owned()));
                    }
                    current.clear();
                    in_bracket = false;
                }
                // Regular character.
                (c, _, _) => {
                    current.push(c);
                }
            }
        }
        Self::flush_segment(&mut current, &mut segments);

        Self { segments }
    }

    fn flush_segment(buf: &mut String, out: &mut Vec<Segment>) {
        let trimmed = buf.trim().to_owned();
        if !trimmed.is_empty() {
            out.push(Segment::Field(trimmed));
        }
        buf.clear();
    }

    /// Returns the number of segments in this path.
    pub fn len(&self) -> usize {
        self.segments.len()
    }

    /// Returns `true` if the path has no segments.
    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }

    /// Returns `true` if a field name requires bracket-quote notation
    /// (i.e., it contains characters that would break dot-path parsing).
    fn needs_bracket_quote(name: &str) -> bool {
        name.contains('.')
            || name.contains('[')
            || name.contains(']')
            || name.contains(' ')
            || name.chars().next().is_some_and(|c| c.is_ascii_digit())
    }

    /// Converts this path back to a dot/bracket string.
    pub fn to_string_path(&self) -> String {
        let mut out = String::new();
        for (i, seg) in self.segments.iter().enumerate() {
            match seg {
                Segment::Field(name) if Self::needs_bracket_quote(name) => {
                    // Emit bracket-quoted notation: ["name"]
                    out.push_str("[\"");
                    out.push_str(name);
                    out.push_str("\"]");
                }
                Segment::Field(name) => {
                    if i > 0 {
                        out.push('.');
                    }
                    out.push_str(name);
                }
                Segment::Index(idx) => {
                    out.push('[');
                    out.push_str(&idx.to_string());
                    out.push(']');
                }
            }
        }
        out
    }

    /// Gets a shared reference to the value at this path, or `None` if the
    /// path doesn't match the value structure.
    pub fn get<'v>(&self, root: &'v DnaValue) -> Option<&'v DnaValue> {
        let mut current = root;
        for seg in &self.segments {
            match (seg, current) {
                (Segment::Field(name), DnaValue::Struct(map)) => {
                    current = map.get(name.as_str())?;
                }
                (Segment::Index(idx), DnaValue::Array(arr)) => {
                    current = arr.get(*idx)?;
                }
                _ => return None,
            }
        }
        Some(current)
    }

    /// Gets a mutable reference to the value at this path.
    pub fn get_mut<'v>(&self, root: &'v mut DnaValue) -> Option<&'v mut DnaValue> {
        let mut current = root;
        for seg in &self.segments {
            match (seg, current) {
                (Segment::Field(name), DnaValue::Struct(map)) => {
                    current = map.get_mut(name.as_str())?;
                }
                (Segment::Index(idx), DnaValue::Array(arr)) => {
                    current = arr.get_mut(*idx)?;
                }
                _ => return None,
            }
        }
        Some(current)
    }

    /// Sets the value at this path, returning the old value on success.
    pub fn set(
        &self,
        root: &mut DnaValue,
        value: DnaValue,
    ) -> Result<DnaValue, PropertyPathError> {
        if self.segments.is_empty() {
            return Err(PropertyPathError::EmptyPath);
        }

        let target = self
            .get_mut(root)
            .ok_or_else(|| PropertyPathError::FieldNotFound(self.to_string_path()))?;

        let old = std::mem::replace(target, value);
        Ok(old)
    }
}

impl std::fmt::Display for PropertyPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string_path())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indexmap::IndexMap;

    fn sample_struct() -> DnaValue {
        let mut inner = IndexMap::new();
        inner.insert("x".into(), DnaValue::F32(1.0));
        inner.insert("y".into(), DnaValue::F32(2.0));
        inner.insert("z".into(), DnaValue::F32(3.0));

        let mut root = IndexMap::new();
        root.insert("position".into(), DnaValue::Struct(inner));
        root.insert(
            "tags".into(),
            DnaValue::Array(vec![
                DnaValue::String("a".into()),
                DnaValue::String("b".into()),
            ]),
        );
        DnaValue::Struct(root)
    }

    #[test]
    fn parse_simple_field() {
        let p = PropertyPath::parse("position.x");
        assert_eq!(p.len(), 2);
        let root = sample_struct();
        assert_eq!(p.get(&root), Some(&DnaValue::F32(1.0)));
    }

    #[test]
    fn parse_array_index() {
        let p = PropertyPath::parse("tags[0]");
        let root = sample_struct();
        assert_eq!(p.get(&root), Some(&DnaValue::String("a".into())));
    }

    #[test]
    fn set_value() {
        let p = PropertyPath::parse("position.y");
        let mut root = sample_struct();
        let old = p.set(&mut root, DnaValue::F32(99.0)).unwrap();
        assert_eq!(old, DnaValue::F32(2.0));
        assert_eq!(p.get(&root), Some(&DnaValue::F32(99.0)));
    }

    #[test]
    fn roundtrip_path_string() {
        let p = PropertyPath::parse("nested.array[2].name");
        assert_eq!(p.to_string_path(), "nested.array[2].name");
    }

    #[test]
    fn missing_field_returns_none() {
        let p = PropertyPath::parse("nonexistent");
        let root = sample_struct();
        assert_eq!(p.get(&root), None);
    }

    #[test]
    fn bracket_quoted_field_name() {
        // Blender-style: pose.bones["Bone"].location
        let mut bones_map = IndexMap::new();
        let mut bone_fields = IndexMap::new();
        bone_fields.insert("location".into(), DnaValue::Vec3([1.0, 2.0, 3.0]));
        bones_map.insert("Bone".into(), DnaValue::Struct(bone_fields));

        let mut root_map = IndexMap::new();
        root_map.insert("pose".into(), DnaValue::Struct({
            let mut m = IndexMap::new();
            m.insert("bones".into(), DnaValue::Struct(bones_map));
            m
        }));
        let root = DnaValue::Struct(root_map);

        let p = PropertyPath::parse("pose.bones[\"Bone\"].location");
        assert_eq!(p.get(&root), Some(&DnaValue::Vec3([1.0, 2.0, 3.0])));
    }

    #[test]
    fn single_quoted_bracket_field() {
        let mut inner = IndexMap::new();
        inner.insert("My Key".into(), DnaValue::F32(42.0));
        let root = DnaValue::Struct(inner);

        let p = PropertyPath::parse("['My Key']");
        assert_eq!(p.get(&root), Some(&DnaValue::F32(42.0)));
    }

    #[test]
    fn blender_path_location_x() {
        // "location.x" - simple dot path
        let mut loc = IndexMap::new();
        loc.insert("x".into(), DnaValue::F32(1.0));
        loc.insert("y".into(), DnaValue::F32(2.0));
        loc.insert("z".into(), DnaValue::F32(3.0));
        let mut root_map = IndexMap::new();
        root_map.insert("location".into(), DnaValue::Struct(loc));
        let root = DnaValue::Struct(root_map);

        let p = PropertyPath::parse("location.x");
        assert_eq!(p.get(&root), Some(&DnaValue::F32(1.0)));
    }

    #[test]
    fn blender_path_pose_bones_location() {
        // "pose.bones[\"Bone\"].location" - Blender bone path
        let mut bone_fields = IndexMap::new();
        bone_fields.insert("location".into(), DnaValue::Vec3([4.0, 5.0, 6.0]));

        let mut bones_map = IndexMap::new();
        bones_map.insert("Bone".into(), DnaValue::Struct(bone_fields));

        let mut pose_map = IndexMap::new();
        pose_map.insert("bones".into(), DnaValue::Struct(bones_map));

        let mut root_map = IndexMap::new();
        root_map.insert("pose".into(), DnaValue::Struct(pose_map));
        let root = DnaValue::Struct(root_map);

        let p = PropertyPath::parse("pose.bones[\"Bone\"].location");
        assert_eq!(p.get(&root), Some(&DnaValue::Vec3([4.0, 5.0, 6.0])));
    }

    #[test]
    fn blender_path_modifiers_array_count() {
        // "modifiers[\"Array\"].count" - Blender modifier path
        let mut array_fields = IndexMap::new();
        array_fields.insert("count".into(), DnaValue::I32(5));

        let mut mods_map = IndexMap::new();
        mods_map.insert("Array".into(), DnaValue::Struct(array_fields));

        let mut root_map = IndexMap::new();
        root_map.insert("modifiers".into(), DnaValue::Struct(mods_map));
        let root = DnaValue::Struct(root_map);

        let p = PropertyPath::parse("modifiers[\"Array\"].count");
        assert_eq!(p.get(&root), Some(&DnaValue::I32(5)));
    }

    #[test]
    fn escape_in_quoted_bracket() {
        // Test that escaped quotes inside brackets work.
        let mut inner = IndexMap::new();
        inner.insert("key\"quote".into(), DnaValue::F32(99.0));
        let root = DnaValue::Struct(inner);

        let p = PropertyPath::parse("[\"key\\\"quote\"]");
        assert_eq!(p.get(&root), Some(&DnaValue::F32(99.0)));
    }

    #[test]
    fn nested_struct_get_set() {
        // Verify DnaValue nested struct get/set works.
        let mut inner = IndexMap::new();
        inner.insert("value".into(), DnaValue::F32(1.0));

        let mut mid = IndexMap::new();
        mid.insert("child".into(), DnaValue::Struct(inner));

        let mut root_map = IndexMap::new();
        root_map.insert("parent".into(), DnaValue::Struct(mid));
        let mut root = DnaValue::Struct(root_map);

        let path = PropertyPath::parse("parent.child.value");
        assert_eq!(path.get(&root), Some(&DnaValue::F32(1.0)));

        let old = path.set(&mut root, DnaValue::F32(42.0)).unwrap();
        assert_eq!(old, DnaValue::F32(1.0));
        assert_eq!(path.get(&root), Some(&DnaValue::F32(42.0)));
    }

    #[test]
    fn path_to_string_bracket_quoted() {
        // "Array" is a plain field name, so to_string_path normalizes it to dot notation.
        let p = PropertyPath::parse("modifiers[\"Array\"].count");
        let s = p.to_string_path();
        assert_eq!(s, "modifiers.Array.count");

        // Field names with special characters DO get bracket-quoted.
        let p2 = PropertyPath::parse("[\"field.with.dots\"].value");
        let s2 = p2.to_string_path();
        assert_eq!(s2, "[\"field.with.dots\"].value");
    }
}
