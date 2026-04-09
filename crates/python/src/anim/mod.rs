//! Python bindings for animation types.

use pyo3::prelude::*;

/// Register animation types with the Python module.
pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyKeyframe>()?;
    m.add_class::<PyFCurve>()?;
    m.add_class::<PyAction>()?;
    Ok(())
}

/// A single keyframe exposed to Python.
#[pyclass(name = "Keyframe")]
#[derive(Debug, Clone)]
pub struct PyKeyframe {
    #[pyo3(get, set)]
    pub frame: f64,
    #[pyo3(get, set)]
    pub value: f64,
    #[pyo3(get, set)]
    pub interpolation: String,
    /// Left handle position (frame, value).
    #[pyo3(get, set)]
    pub handle_left: [f64; 2],
    /// Right handle position (frame, value).
    #[pyo3(get, set)]
    pub handle_right: [f64; 2],
}

#[pymethods]
impl PyKeyframe {
    #[new]
    #[pyo3(signature = (frame, value, interpolation="BEZIER".to_string()))]
    fn new(frame: f64, value: f64, interpolation: String) -> Self {
        Self {
            frame,
            value,
            interpolation,
            handle_left: [frame - 1.0, value],
            handle_right: [frame + 1.0, value],
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "Keyframe(frame={}, value={}, interp='{}')",
            self.frame, self.value, self.interpolation
        )
    }
}

/// An F-Curve (function curve) exposed to Python.
#[pyclass(name = "FCurve")]
#[derive(Debug, Clone)]
pub struct PyFCurve {
    #[pyo3(get, set)]
    pub data_path: String,
    #[pyo3(get, set)]
    pub array_index: u32,
    keyframes: Vec<PyKeyframe>,
}

#[pymethods]
impl PyFCurve {
    #[new]
    fn new(data_path: String, array_index: u32) -> Self {
        Self {
            data_path,
            array_index,
            keyframes: Vec::new(),
        }
    }

    /// Insert a keyframe at the given frame and value.
    fn keyframe_insert(&mut self, frame: f64, value: f64) {
        // Remove existing keyframe at the same frame.
        self.keyframes.retain(|k| (k.frame - frame).abs() > 1e-6);
        self.keyframes.push(PyKeyframe::new(frame, value, "BEZIER".into()));
        self.keyframes.sort_by(|a, b| a.frame.partial_cmp(&b.frame).unwrap());
    }

    /// Evaluate the curve at a given frame (linear interpolation for simplicity).
    fn evaluate(&self, frame: f64) -> f64 {
        if self.keyframes.is_empty() {
            return 0.0;
        }
        if self.keyframes.len() == 1 || frame <= self.keyframes[0].frame {
            return self.keyframes[0].value;
        }
        let last = &self.keyframes[self.keyframes.len() - 1];
        if frame >= last.frame {
            return last.value;
        }
        // Find surrounding keyframes.
        for w in self.keyframes.windows(2) {
            let a = &w[0];
            let b = &w[1];
            if frame >= a.frame && frame <= b.frame {
                let t = (frame - a.frame) / (b.frame - a.frame);
                return a.value + t * (b.value - a.value);
            }
        }
        last.value
    }

    /// Number of keyframes.
    fn keyframe_count(&self) -> usize {
        self.keyframes.len()
    }

    fn __repr__(&self) -> String {
        format!(
            "FCurve(data_path='{}', index={}, keys={})",
            self.data_path,
            self.array_index,
            self.keyframes.len()
        )
    }
}

/// An action (collection of FCurves) exposed to Python.
#[pyclass(name = "Action")]
#[derive(Debug, Clone)]
pub struct PyAction {
    #[pyo3(get, set)]
    pub name: String,
    fcurves: Vec<PyFCurve>,
}

#[pymethods]
impl PyAction {
    #[new]
    fn new(name: String) -> Self {
        Self {
            name,
            fcurves: Vec::new(),
        }
    }

    /// Add an FCurve to this action.
    fn add_fcurve(&mut self, fcurve: PyFCurve) {
        self.fcurves.push(fcurve);
    }

    /// Get an FCurve by data path and array index.
    fn get_fcurve(&self, data_path: &str, array_index: u32) -> Option<PyFCurve> {
        self.fcurves
            .iter()
            .find(|f| f.data_path == data_path && f.array_index == array_index)
            .cloned()
    }

    /// Number of FCurves.
    fn fcurve_count(&self) -> usize {
        self.fcurves.len()
    }

    fn __repr__(&self) -> String {
        format!("Action(name='{}', fcurves={})", self.name, self.fcurves.len())
    }
}
