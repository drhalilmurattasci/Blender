//! Constraint evaluation context: provides transforms and target lookups.

/// Minimal 4x4 column-major matrix for constraint evaluation.
/// This is used as an intermediate representation; the actual `Mat4` comes from `forge3d-math`.
#[derive(Debug, Clone, Copy)]
pub struct Transform {
    /// Location (translation) component.
    pub location: [f32; 3],
    /// Rotation as quaternion [x, y, z, w].
    pub rotation: [f32; 4],
    /// Scale component.
    pub scale: [f32; 3],
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            location: [0.0; 3],
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale: [1.0; 3],
        }
    }
}

impl Transform {
    /// Identity transform.
    pub fn identity() -> Self {
        Self::default()
    }

    /// Linearly interpolate between two transforms.
    pub fn blend(&self, other: &Transform, factor: f32) -> Transform {
        let f = factor.clamp(0.0, 1.0);
        let inv = 1.0 - f;
        Transform {
            location: [
                self.location[0] * inv + other.location[0] * f,
                self.location[1] * inv + other.location[1] * f,
                self.location[2] * inv + other.location[2] * f,
            ],
            rotation: slerp_quat(self.rotation, other.rotation, f),
            scale: [
                self.scale[0] * inv + other.scale[0] * f,
                self.scale[1] * inv + other.scale[1] * f,
                self.scale[2] * inv + other.scale[2] * f,
            ],
        }
    }
}

/// Simple quaternion SLERP for constraint blending.
fn slerp_quat(a: [f32; 4], b: [f32; 4], t: f32) -> [f32; 4] {
    let mut dot = a[0] * b[0] + a[1] * b[1] + a[2] * b[2] + a[3] * b[3];
    let mut b = b;

    // Ensure shortest path.
    if dot < 0.0 {
        b = [-b[0], -b[1], -b[2], -b[3]];
        dot = -dot;
    }

    if dot > 0.9995 {
        // Very close: use linear interpolation.
        let inv = 1.0 - t;
        let mut result = [
            a[0] * inv + b[0] * t,
            a[1] * inv + b[1] * t,
            a[2] * inv + b[2] * t,
            a[3] * inv + b[3] * t,
        ];
        let len = (result[0] * result[0]
            + result[1] * result[1]
            + result[2] * result[2]
            + result[3] * result[3])
            .sqrt();
        if len > f32::EPSILON {
            for r in &mut result {
                *r /= len;
            }
        }
        return result;
    }

    let theta = dot.acos();
    let sin_theta = theta.sin();
    let wa = ((1.0 - t) * theta).sin() / sin_theta;
    let wb = (t * theta).sin() / sin_theta;

    [
        a[0] * wa + b[0] * wb,
        a[1] * wa + b[1] * wb,
        a[2] * wa + b[2] * wb,
        a[3] * wa + b[3] * wb,
    ]
}

/// Context provided to constraints during evaluation.
/// In a real pipeline this would hold references to the scene and armature;
/// here we keep it minimal and trait-free so it compiles standalone.
pub struct ConstraintContext {
    /// The owner's current transform (will be modified by the constraint).
    pub owner_transform: Transform,
    /// The owner's rest/bind transform.
    pub owner_rest: Transform,
    /// The target's world-space transform (if applicable).
    pub target_transform: Option<Transform>,
    /// The target's rest/bind transform (if applicable).
    pub target_rest: Option<Transform>,
}

impl ConstraintContext {
    /// Create a context with just an owner transform.
    pub fn owner_only(transform: Transform) -> Self {
        Self {
            owner_transform: transform,
            owner_rest: Transform::identity(),
            target_transform: None,
            target_rest: None,
        }
    }

    /// Create a context with owner and target transforms.
    pub fn with_target(owner: Transform, target: Transform) -> Self {
        Self {
            owner_transform: owner,
            owner_rest: Transform::identity(),
            target_transform: Some(target),
            target_rest: Some(Transform::identity()),
        }
    }
}
