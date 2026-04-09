//! Parallel evaluation of the dependency graph using rayon.

mod scheduler;

pub use scheduler::evaluate_parallel;
