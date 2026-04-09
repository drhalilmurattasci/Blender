//! Python bindings for scene and object types.

use pyo3::prelude::*;

/// Register scene types with the Python module.
pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyObject>()?;
    m.add_class::<PyScene>()?;
    Ok(())
}

/// A scene object exposed to Python.
#[pyclass(name = "Object")]
#[derive(Debug, Clone)]
pub struct PyObject {
    #[pyo3(get, set)]
    pub name: String,
    #[pyo3(get, set)]
    pub location: [f64; 3],
    #[pyo3(get, set)]
    pub rotation_euler: [f64; 3],
    #[pyo3(get, set)]
    pub scale: [f64; 3],
    #[pyo3(get)]
    pub object_type: String,
    #[pyo3(get, set)]
    pub visible: bool,
    #[pyo3(get, set)]
    pub selected: bool,
}

#[pymethods]
impl PyObject {
    #[new]
    #[pyo3(signature = (name, object_type="EMPTY".to_string()))]
    fn new(name: String, object_type: String) -> Self {
        Self {
            name,
            location: [0.0; 3],
            rotation_euler: [0.0; 3],
            scale: [1.0, 1.0, 1.0],
            object_type,
            visible: true,
            selected: false,
        }
    }

    fn __repr__(&self) -> String {
        format!("Object(name='{}', type='{}')", self.name, self.object_type)
    }
}

/// A scene container exposed to Python.
#[pyclass(name = "Scene")]
#[derive(Debug, Clone)]
pub struct PyScene {
    #[pyo3(get, set)]
    pub name: String,
    #[pyo3(get, set)]
    pub frame_start: i32,
    #[pyo3(get, set)]
    pub frame_end: i32,
    #[pyo3(get, set)]
    pub frame_current: f64,
    #[pyo3(get, set)]
    pub fps: f64,
    /// Objects in the scene.
    objects: Vec<PyObject>,
}

#[pymethods]
impl PyScene {
    #[new]
    #[pyo3(signature = (name="Scene".to_string()))]
    fn new(name: String) -> Self {
        Self {
            name,
            frame_start: 1,
            frame_end: 250,
            frame_current: 1.0,
            fps: 24.0,
            objects: Vec::new(),
        }
    }

    /// Add an object to the scene.
    fn add_object(&mut self, obj: PyObject) {
        self.objects.push(obj);
    }

    /// Remove an object by name. Returns True if found.
    fn remove_object(&mut self, name: &str) -> bool {
        let before = self.objects.len();
        self.objects.retain(|o| o.name != name);
        self.objects.len() != before
    }

    /// Get an object by name.
    fn get_object(&self, name: &str) -> Option<PyObject> {
        self.objects.iter().find(|o| o.name == name).cloned()
    }

    /// List all object names.
    fn object_names(&self) -> Vec<String> {
        self.objects.iter().map(|o| o.name.clone()).collect()
    }

    /// Number of objects.
    fn object_count(&self) -> usize {
        self.objects.len()
    }

    fn __repr__(&self) -> String {
        format!(
            "Scene(name='{}', objects={}, frames={}..{})",
            self.name,
            self.objects.len(),
            self.frame_start,
            self.frame_end,
        )
    }
}
