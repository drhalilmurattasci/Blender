//! Python bindings for UI scripting: operator registration, panel layout,
//! and menu construction from Python scripts.

use pyo3::prelude::*;

/// Register UI types with the Python module.
pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyUILayout>()?;
    m.add_class::<PyOperatorProperties>()?;
    Ok(())
}

/// A simplified UI layout context exposed to Python for drawing panels.
#[pyclass(name = "UILayout")]
#[derive(Debug, Clone)]
pub struct PyUILayout {
    /// Accumulated layout commands (serialised for the Rust UI to interpret).
    commands: Vec<LayoutCommand>,
}

/// Internal layout command representation.
#[derive(Debug, Clone)]
enum LayoutCommand {
    Label(String),
    Property(String, String),
    Button(String, String),
    Separator,
    Row,
    Column,
}

#[pymethods]
impl PyUILayout {
    #[new]
    fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    /// Add a text label.
    fn label(&mut self, text: &str) {
        self.commands.push(LayoutCommand::Label(text.to_string()));
    }

    /// Add a property field (data_path, property_name).
    fn prop(&mut self, data_path: &str, property: &str) {
        self.commands
            .push(LayoutCommand::Property(data_path.to_string(), property.to_string()));
    }

    /// Add a button that invokes an operator.
    fn operator(&mut self, idname: &str, text: &str) {
        self.commands
            .push(LayoutCommand::Button(idname.to_string(), text.to_string()));
    }

    /// Add a visual separator.
    fn separator(&mut self) {
        self.commands.push(LayoutCommand::Separator);
    }

    /// Begin a horizontal row.
    fn row(&mut self) {
        self.commands.push(LayoutCommand::Row);
    }

    /// Begin a vertical column.
    fn column(&mut self) {
        self.commands.push(LayoutCommand::Column);
    }

    /// Number of accumulated commands.
    fn command_count(&self) -> usize {
        self.commands.len()
    }
}

/// Properties passed to a Python operator invocation.
#[pyclass(name = "OperatorProperties")]
#[derive(Debug, Clone)]
pub struct PyOperatorProperties {
    values: Vec<(String, PyPropertyValue)>,
}

/// A dynamically-typed property value.
#[derive(Debug, Clone)]
enum PyPropertyValue {
    Float(f64),
    Int(i64),
    Bool(bool),
    String(String),
}

#[pymethods]
impl PyOperatorProperties {
    #[new]
    fn new() -> Self {
        Self {
            values: Vec::new(),
        }
    }

    fn set_float(&mut self, name: &str, value: f64) {
        self.values
            .push((name.to_string(), PyPropertyValue::Float(value)));
    }

    fn set_int(&mut self, name: &str, value: i64) {
        self.values
            .push((name.to_string(), PyPropertyValue::Int(value)));
    }

    fn set_bool(&mut self, name: &str, value: bool) {
        self.values
            .push((name.to_string(), PyPropertyValue::Bool(value)));
    }

    fn set_string(&mut self, name: &str, value: &str) {
        self.values
            .push((name.to_string(), PyPropertyValue::String(value.to_string())));
    }

    fn get_float(&self, name: &str) -> Option<f64> {
        self.values.iter().rev().find_map(|(n, v)| {
            if n == name {
                if let PyPropertyValue::Float(f) = v {
                    Some(*f)
                } else {
                    None
                }
            } else {
                None
            }
        })
    }

    fn get_int(&self, name: &str) -> Option<i64> {
        self.values.iter().rev().find_map(|(n, v)| {
            if n == name {
                if let PyPropertyValue::Int(i) = v {
                    Some(*i)
                } else {
                    None
                }
            } else {
                None
            }
        })
    }

    fn get_bool(&self, name: &str) -> Option<bool> {
        self.values.iter().rev().find_map(|(n, v)| {
            if n == name {
                if let PyPropertyValue::Bool(b) = v {
                    Some(*b)
                } else {
                    None
                }
            } else {
                None
            }
        })
    }

    fn get_string(&self, name: &str) -> Option<String> {
        self.values.iter().rev().find_map(|(n, v)| {
            if n == name {
                if let PyPropertyValue::String(s) = v {
                    Some(s.clone())
                } else {
                    None
                }
            } else {
                None
            }
        })
    }
}
