//! Python bindings for mesh data.

use pyo3::prelude::*;

/// Register mesh types with the Python module.
pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyMeshData>()?;
    m.add_class::<PyMeshVertex>()?;
    Ok(())
}

/// A mesh vertex exposed to Python.
#[pyclass(name = "MeshVertex")]
#[derive(Debug, Clone)]
pub struct PyMeshVertex {
    #[pyo3(get)]
    pub index: usize,
    #[pyo3(get, set)]
    pub co: [f64; 3],
    #[pyo3(get, set)]
    pub normal: [f64; 3],
}

#[pymethods]
impl PyMeshVertex {
    #[new]
    fn new(index: usize, co: [f64; 3]) -> Self {
        Self {
            index,
            co,
            normal: [0.0, 0.0, 1.0],
        }
    }

    fn __repr__(&self) -> String {
        format!("MeshVertex(index={}, co={:?})", self.index, self.co)
    }
}

/// Simplified mesh data exposed to Python.
#[pyclass(name = "MeshData")]
#[derive(Debug, Clone)]
pub struct PyMeshData {
    /// Flat array of vertex positions [x0, y0, z0, x1, y1, z1, ...].
    positions: Vec<f64>,
    /// Face indices (triangle list).
    indices: Vec<u32>,
}

#[pymethods]
impl PyMeshData {
    #[new]
    fn new() -> Self {
        Self {
            positions: Vec::new(),
            indices: Vec::new(),
        }
    }

    /// Number of vertices.
    fn vertex_count(&self) -> usize {
        self.positions.len() / 3
    }

    /// Number of triangles.
    fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }

    /// Add a vertex, returning its index.
    fn add_vertex(&mut self, x: f64, y: f64, z: f64) -> usize {
        let idx = self.vertex_count();
        self.positions.extend_from_slice(&[x, y, z]);
        idx
    }

    /// Add a triangle face from three vertex indices.
    fn add_triangle(&mut self, a: u32, b: u32, c: u32) {
        self.indices.extend_from_slice(&[a, b, c]);
    }

    /// Get vertex position by index.
    fn get_vertex(&self, index: usize) -> PyResult<[f64; 3]> {
        let base = index * 3;
        if base + 3 > self.positions.len() {
            return Err(pyo3::exceptions::PyIndexError::new_err("vertex index out of range"));
        }
        Ok([self.positions[base], self.positions[base + 1], self.positions[base + 2]])
    }

    /// Set vertex position by index.
    fn set_vertex(&mut self, index: usize, x: f64, y: f64, z: f64) -> PyResult<()> {
        let base = index * 3;
        if base + 3 > self.positions.len() {
            return Err(pyo3::exceptions::PyIndexError::new_err("vertex index out of range"));
        }
        self.positions[base] = x;
        self.positions[base + 1] = y;
        self.positions[base + 2] = z;
        Ok(())
    }

    /// Get flat positions array.
    fn get_positions(&self) -> Vec<f64> {
        self.positions.clone()
    }

    /// Get flat indices array.
    fn get_indices(&self) -> Vec<u32> {
        self.indices.clone()
    }

    fn __repr__(&self) -> String {
        format!(
            "MeshData(vertices={}, triangles={})",
            self.vertex_count(),
            self.triangle_count()
        )
    }
}
