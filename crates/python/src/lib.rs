//! # forge3d-python
//!
//! Python bindings for Forge3D via PyO3. Exposes mesh, scene, animation,
//! and utility types to Python scripting.

pub mod anim;
pub mod auto_bind;
pub mod mesh;
pub mod scene;
pub mod ui;

use pyo3::prelude::*;

/// Initialize the `forge3d` Python module.
#[pymodule]
fn forge3d(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Register sub-modules.
    register_math(m)?;
    mesh::register(m)?;
    scene::register(m)?;
    anim::register(m)?;
    ui::register(m)?;
    Ok(())
}

/// Register basic math types (Vector3, Quaternion, Matrix4).
fn register_math(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyVector3>()?;
    m.add_class::<PyQuaternion>()?;
    m.add_class::<PyMatrix4>()?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Math wrappers
// ---------------------------------------------------------------------------

/// A 3-component vector exposed to Python.
#[pyclass(name = "Vector3")]
#[derive(Debug, Clone)]
pub struct PyVector3 {
    #[pyo3(get, set)]
    pub x: f64,
    #[pyo3(get, set)]
    pub y: f64,
    #[pyo3(get, set)]
    pub z: f64,
}

#[pymethods]
impl PyVector3 {
    #[new]
    fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    fn length(&self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    fn normalized(&self) -> Self {
        let len = self.length();
        if len < 1e-12 {
            Self::new(0.0, 0.0, 0.0)
        } else {
            Self::new(self.x / len, self.y / len, self.z / len)
        }
    }

    fn dot(&self, other: &PyVector3) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    fn cross(&self, other: &PyVector3) -> Self {
        Self::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }

    fn __repr__(&self) -> String {
        format!("Vector3({}, {}, {})", self.x, self.y, self.z)
    }

    fn __add__(&self, other: &PyVector3) -> Self {
        Self::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }

    fn __sub__(&self, other: &PyVector3) -> Self {
        Self::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }

    fn __mul__(&self, scalar: f64) -> Self {
        Self::new(self.x * scalar, self.y * scalar, self.z * scalar)
    }

    fn __neg__(&self) -> Self {
        Self::new(-self.x, -self.y, -self.z)
    }
}

/// A quaternion exposed to Python (x, y, z, w).
#[pyclass(name = "Quaternion")]
#[derive(Debug, Clone)]
pub struct PyQuaternion {
    #[pyo3(get, set)]
    pub x: f64,
    #[pyo3(get, set)]
    pub y: f64,
    #[pyo3(get, set)]
    pub z: f64,
    #[pyo3(get, set)]
    pub w: f64,
}

#[pymethods]
impl PyQuaternion {
    #[new]
    #[pyo3(signature = (w=1.0, x=0.0, y=0.0, z=0.0))]
    fn new(w: f64, x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z, w }
    }

    fn magnitude(&self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w).sqrt()
    }

    fn normalized(&self) -> Self {
        let m = self.magnitude();
        if m < 1e-12 {
            Self::new(1.0, 0.0, 0.0, 0.0)
        } else {
            Self {
                x: self.x / m,
                y: self.y / m,
                z: self.z / m,
                w: self.w / m,
            }
        }
    }

    fn conjugate(&self) -> Self {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
            w: self.w,
        }
    }

    fn __repr__(&self) -> String {
        format!("Quaternion(w={}, x={}, y={}, z={})", self.w, self.x, self.y, self.z)
    }
}

/// A 4x4 matrix exposed to Python (column-major storage).
#[pyclass(name = "Matrix4")]
#[derive(Debug, Clone)]
pub struct PyMatrix4 {
    /// Column-major 4x4 matrix data.
    pub data: [f64; 16],
}

#[pymethods]
impl PyMatrix4 {
    #[new]
    fn new() -> Self {
        Self {
            data: [
                1.0, 0.0, 0.0, 0.0,
                0.0, 1.0, 0.0, 0.0,
                0.0, 0.0, 1.0, 0.0,
                0.0, 0.0, 0.0, 1.0,
            ],
        }
    }

    /// Create an identity matrix.
    #[staticmethod]
    fn identity() -> Self {
        Self::new()
    }

    /// Get element at (row, col).
    fn get(&self, row: usize, col: usize) -> PyResult<f64> {
        if row >= 4 || col >= 4 {
            return Err(pyo3::exceptions::PyIndexError::new_err("index out of range"));
        }
        Ok(self.data[col * 4 + row])
    }

    /// Set element at (row, col).
    fn set(&mut self, row: usize, col: usize, value: f64) -> PyResult<()> {
        if row >= 4 || col >= 4 {
            return Err(pyo3::exceptions::PyIndexError::new_err("index out of range"));
        }
        self.data[col * 4 + row] = value;
        Ok(())
    }

    fn __repr__(&self) -> String {
        format!("Matrix4({:?})", &self.data)
    }
}
