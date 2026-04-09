use forge3d_math::{Mat4, Vec3};

/// Projection type for the camera.
#[derive(Debug, Clone, Copy)]
pub enum Projection {
    Perspective {
        fov_y: f32,
        aspect: f32,
        near: f32,
        far: f32,
    },
    Orthographic {
        width: f32,
        height: f32,
        near: f32,
        far: f32,
    },
}

impl Projection {
    /// Build the projection matrix.
    pub fn matrix(&self) -> Mat4 {
        match *self {
            Projection::Perspective {
                fov_y,
                aspect,
                near,
                far,
            } => Mat4::perspective_rh(fov_y, aspect, near, far),
            Projection::Orthographic {
                width,
                height,
                near,
                far,
            } => {
                let hw = width * 0.5;
                let hh = height * 0.5;
                Mat4::orthographic_rh(-hw, hw, -hh, hh, near, far)
            }
        }
    }
}

/// A camera with position, orientation, and projection.
#[derive(Debug, Clone)]
pub struct Camera {
    pub position: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub projection: Projection,
}

impl Camera {
    /// Create a new perspective camera.
    pub fn perspective(position: Vec3, target: Vec3, fov_y: f32, aspect: f32, near: f32, far: f32) -> Self {
        Self {
            position,
            target,
            up: Vec3::new(0.0, 1.0, 0.0),
            projection: Projection::Perspective {
                fov_y,
                aspect,
                near,
                far,
            },
        }
    }

    /// Get the view matrix.
    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(self.position, self.target, self.up)
    }

    /// Get the projection matrix.
    pub fn projection_matrix(&self) -> Mat4 {
        self.projection.matrix()
    }

    /// Get the combined view-projection matrix.
    pub fn view_projection_matrix(&self) -> Mat4 {
        self.projection_matrix() * self.view_matrix()
    }

    /// Get the forward direction.
    pub fn forward(&self) -> Vec3 {
        (self.target - self.position).normalize()
    }

    /// Get the right direction.
    pub fn right(&self) -> Vec3 {
        self.forward().cross(self.up).normalize()
    }

    /// Generate a ray direction for the given pixel coordinates.
    pub fn ray_direction(&self, u: f32, v: f32) -> Vec3 {
        match self.projection {
            Projection::Perspective { fov_y, aspect, .. } => {
                let half_height = (fov_y * 0.5).tan();
                let half_width = aspect * half_height;

                let forward = self.forward();
                let right = self.right();
                let up = right.cross(forward).normalize();

                let x = (2.0 * u - 1.0) * half_width;
                let y = (2.0 * v - 1.0) * half_height;

                (forward + right * x + up * y).normalize()
            }
            Projection::Orthographic { .. } => self.forward(),
        }
    }
}

/// Simple camera controller for orbit-style navigation.
pub struct CameraController {
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
    pub center: Vec3,
    pub sensitivity: f32,
}

impl CameraController {
    /// Create a new camera controller.
    pub fn new(distance: f32, center: Vec3) -> Self {
        Self {
            yaw: 0.0,
            pitch: 0.3,
            distance,
            center,
            sensitivity: 0.005,
        }
    }

    /// Rotate the camera by mouse delta.
    pub fn rotate(&mut self, dx: f32, dy: f32) {
        self.yaw += dx * self.sensitivity;
        self.pitch = (self.pitch + dy * self.sensitivity).clamp(-1.5, 1.5);
    }

    /// Zoom the camera.
    pub fn zoom(&mut self, delta: f32) {
        self.distance = (self.distance - delta).max(0.1);
    }

    /// Get the camera position from the controller state.
    pub fn camera_position(&self) -> Vec3 {
        let x = self.distance * self.pitch.cos() * self.yaw.sin();
        let y = self.distance * self.pitch.sin();
        let z = self.distance * self.pitch.cos() * self.yaw.cos();
        self.center + Vec3::new(x, y, z)
    }

    /// Apply the controller state to a camera.
    pub fn apply(&self, camera: &mut Camera) {
        camera.position = self.camera_position();
        camera.target = self.center;
    }
}
