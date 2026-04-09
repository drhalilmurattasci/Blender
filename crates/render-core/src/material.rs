use forge3d_math::Vec3;

/// Material types supported by the renderer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MaterialType {
    Lambertian,
    Metal,
    Dielectric,
    Emissive,
    CookTorrance,
}

/// A physically-based material definition.
#[derive(Debug, Clone)]
pub struct Material {
    pub material_type: MaterialType,
    pub albedo: Vec3,
    pub roughness: f32,
    pub metallic: f32,
    pub ior: f32,
    pub emission: Vec3,
    pub emission_strength: f32,
    pub normal_map_index: Option<u32>,
    pub albedo_map_index: Option<u32>,
    pub roughness_map_index: Option<u32>,
    pub metallic_map_index: Option<u32>,
}

impl Material {
    /// Create a default Lambertian material.
    pub fn lambertian(albedo: Vec3) -> Self {
        Self {
            material_type: MaterialType::Lambertian,
            albedo,
            roughness: 1.0,
            metallic: 0.0,
            ior: 1.5,
            emission: Vec3::new(0.0, 0.0, 0.0),
            emission_strength: 0.0,
            normal_map_index: None,
            albedo_map_index: None,
            roughness_map_index: None,
            metallic_map_index: None,
        }
    }

    /// Create a metallic material.
    pub fn metal(albedo: Vec3, roughness: f32) -> Self {
        Self {
            material_type: MaterialType::Metal,
            albedo,
            roughness,
            metallic: 1.0,
            ior: 1.5,
            emission: Vec3::new(0.0, 0.0, 0.0),
            emission_strength: 0.0,
            normal_map_index: None,
            albedo_map_index: None,
            roughness_map_index: None,
            metallic_map_index: None,
        }
    }

    /// Create a dielectric (glass) material.
    pub fn dielectric(ior: f32) -> Self {
        Self {
            material_type: MaterialType::Dielectric,
            albedo: Vec3::new(1.0, 1.0, 1.0),
            roughness: 0.0,
            metallic: 0.0,
            ior,
            emission: Vec3::new(0.0, 0.0, 0.0),
            emission_strength: 0.0,
            normal_map_index: None,
            albedo_map_index: None,
            roughness_map_index: None,
            metallic_map_index: None,
        }
    }

    /// Create an emissive material.
    pub fn emissive(color: Vec3, strength: f32) -> Self {
        Self {
            material_type: MaterialType::Emissive,
            albedo: Vec3::new(0.0, 0.0, 0.0),
            roughness: 1.0,
            metallic: 0.0,
            ior: 1.5,
            emission: color,
            emission_strength: strength,
            normal_map_index: None,
            albedo_map_index: None,
            roughness_map_index: None,
            metallic_map_index: None,
        }
    }

    /// Create a PBR Cook-Torrance material.
    pub fn pbr(albedo: Vec3, roughness: f32, metallic: f32) -> Self {
        Self {
            material_type: MaterialType::CookTorrance,
            albedo,
            roughness,
            metallic,
            ior: 1.5,
            emission: Vec3::new(0.0, 0.0, 0.0),
            emission_strength: 0.0,
            normal_map_index: None,
            albedo_map_index: None,
            roughness_map_index: None,
            metallic_map_index: None,
        }
    }

    /// Whether this material emits light.
    pub fn is_emissive(&self) -> bool {
        self.emission_strength > 0.0
    }
}
