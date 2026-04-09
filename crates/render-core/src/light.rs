use forge3d_math::Vec3;

/// Light types in the scene.
#[derive(Debug, Clone)]
pub enum Light {
    Point(PointLight),
    Directional(DirectionalLight),
    Spot(SpotLight),
}

/// A point light with position and intensity.
#[derive(Debug, Clone)]
pub struct PointLight {
    pub position: Vec3,
    pub color: Vec3,
    pub intensity: f32,
    pub radius: f32,
}

impl PointLight {
    pub fn new(position: Vec3, color: Vec3, intensity: f32) -> Self {
        Self {
            position,
            color,
            intensity,
            radius: 0.0,
        }
    }

    /// Compute the attenuation at a given distance.
    pub fn attenuation(&self, distance: f32) -> f32 {
        self.intensity / (distance * distance).max(0.0001)
    }
}

/// A directional light (like the sun).
#[derive(Debug, Clone)]
pub struct DirectionalLight {
    pub direction: Vec3,
    pub color: Vec3,
    pub intensity: f32,
    pub angular_diameter: f32,
}

impl DirectionalLight {
    pub fn new(direction: Vec3, color: Vec3, intensity: f32) -> Self {
        Self {
            direction: direction.normalize(),
            color,
            intensity,
            angular_diameter: 0.00935, // approximate sun
        }
    }
}

/// A spotlight with position, direction, and cone angle.
#[derive(Debug, Clone)]
pub struct SpotLight {
    pub position: Vec3,
    pub direction: Vec3,
    pub color: Vec3,
    pub intensity: f32,
    pub inner_angle: f32,
    pub outer_angle: f32,
}

impl SpotLight {
    pub fn new(
        position: Vec3,
        direction: Vec3,
        color: Vec3,
        intensity: f32,
        inner_angle: f32,
        outer_angle: f32,
    ) -> Self {
        Self {
            position,
            direction: direction.normalize(),
            color,
            intensity,
            inner_angle,
            outer_angle,
        }
    }

    /// Compute the angular falloff for a given direction from the light to the surface.
    /// Uses smoothstep (Hermite) interpolation matching Blender EEVEE/Cycles.
    pub fn angular_attenuation(&self, to_surface: Vec3) -> f32 {
        let cos_angle = to_surface.normalize().dot(self.direction);
        let cos_inner = self.inner_angle.cos();
        let cos_outer = self.outer_angle.cos();

        if cos_angle > cos_inner {
            1.0
        } else if cos_angle > cos_outer {
            let t = ((cos_angle - cos_outer) / (cos_inner - cos_outer)).clamp(0.0, 1.0);
            // smoothstep: 3t^2 - 2t^3 (matches Cycles/EEVEE smooth falloff)
            t * t * (3.0 - 2.0 * t)
        } else {
            0.0
        }
    }
}
