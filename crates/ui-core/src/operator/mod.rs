//! Operators: the command pattern for all user-visible actions.
//!
//! An operator has an `idname` (e.g. `"OBJECT_OT_delete"`), optional poll,
//! invoke, execute, and modal phases.

use ahash::AHashMap;
use std::any::Any;

/// Result of an operator invocation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OperatorResult {
    /// Operator completed successfully.
    Finished,
    /// Operator was cancelled (no changes made).
    Cancelled,
    /// Operator entered modal mode (continues receiving events).
    RunningModal,
    /// Operator passed through (did not handle the event).
    PassThrough,
    /// Operator finished and should push undo state.
    FinishedUndo,
}

/// Contextual data passed to operators.
pub struct OperatorContext<'a> {
    /// Arbitrary context data keyed by type-erased name.
    data: AHashMap<&'a str, &'a dyn Any>,
}

impl<'a> OperatorContext<'a> {
    /// Create an empty context.
    pub fn new() -> Self {
        Self {
            data: AHashMap::new(),
        }
    }

    /// Insert a context value.
    pub fn insert(&mut self, key: &'a str, value: &'a dyn Any) {
        self.data.insert(key, value);
    }

    /// Retrieve a typed context value.
    pub fn get<T: 'static>(&self, key: &str) -> Option<&T> {
        self.data.get(key).and_then(|v| v.downcast_ref::<T>())
    }
}

impl Default for OperatorContext<'_> {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait that all operators implement.
pub trait Operator: Send + Sync {
    /// Unique identifier (e.g. `"MESH_OT_subdivide"`).
    fn idname(&self) -> &str;

    /// Human-readable label for menus and tooltips.
    fn label(&self) -> &str {
        self.idname()
    }

    /// Optional description.
    fn description(&self) -> &str {
        ""
    }

    /// Poll: return `true` if this operator can run in the current context.
    fn poll(&self, _ctx: &OperatorContext<'_>) -> bool {
        true
    }

    /// Execute the operator. Called for non-interactive invocations.
    fn execute(&mut self, ctx: &OperatorContext<'_>) -> OperatorResult;

    /// Invoke: entry point for interactive (user-triggered) runs.
    /// By default, just calls `execute`.
    fn invoke(&mut self, ctx: &OperatorContext<'_>) -> OperatorResult {
        self.execute(ctx)
    }

    /// Modal: called repeatedly while the operator is in modal mode
    /// (i.e., after `invoke` returned `RunningModal`). Each event is
    /// dispatched here until the operator returns `Finished`, `Cancelled`,
    /// or `FinishedUndo`.
    ///
    /// Returning `PassThrough` allows the event to continue down the
    /// handler chain while keeping the modal operator alive.
    fn modal(&mut self, _ctx: &OperatorContext<'_>) -> OperatorResult {
        OperatorResult::Finished
    }

    /// Whether this operator should push an undo step after execution.
    /// Blender's OPTYPE_UNDO flag equivalent.
    fn uses_undo(&self) -> bool {
        true
    }
}

/// Registry of all known operators, keyed by idname.
pub struct OperatorRegistry {
    factories: AHashMap<String, Box<dyn Fn() -> Box<dyn Operator>>>,
}

impl OperatorRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            factories: AHashMap::new(),
        }
    }

    /// Register an operator factory function.
    pub fn register<F>(&mut self, idname: impl Into<String>, factory: F)
    where
        F: Fn() -> Box<dyn Operator> + 'static,
    {
        self.factories.insert(idname.into(), Box::new(factory));
    }

    /// Create an operator instance by idname.
    pub fn create(&self, idname: &str) -> Option<Box<dyn Operator>> {
        self.factories.get(idname).map(|f| f())
    }

    /// Returns `true` if an operator with this idname is registered.
    pub fn contains(&self, idname: &str) -> bool {
        self.factories.contains_key(idname)
    }

    /// Iterate all registered idnames.
    pub fn idnames(&self) -> impl Iterator<Item = &str> {
        self.factories.keys().map(|s| s.as_str())
    }
}

impl Default for OperatorRegistry {
    fn default() -> Self {
        Self::new()
    }
}
