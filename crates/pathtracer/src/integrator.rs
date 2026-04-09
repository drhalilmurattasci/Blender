use forge3d_math::Vec3;
use forge3d_render_core::light::Light;
use forge3d_render_core::sampling::{cosine_hemisphere_pdf, cosine_hemisphere_sample, Sampler};

use crate::scene::PathtracerScene;

/// Configuration for the path integrator.
#[derive(Debug, Clone)]
pub struct IntegratorConfig {
    pub max_bounces: u32,
    pub russian_roulette_depth: u32,
    pub clamp_value: f32,
}

impl Default for IntegratorConfig {
    fn default() -> Self {
        Self {
            max_bounces: 8,
            russian_roulette_depth: 3,
            clamp_value: 100.0,
        }
    }
}

/// A simple unidirectional path integrator.
pub struct PathIntegrator {
    pub config: IntegratorConfig,
}

impl PathIntegrator {
    pub fn new(config: IntegratorConfig) -> Self {
        Self { config }
    }

    /// Trace a single path and return the radiance estimate.
    ///
    /// Uses next event estimation (direct light sampling) with MIS balance heuristic,
    /// matching the approach used in Blender Cycles.
    pub fn trace(
        &self,
        scene: &PathtracerScene,
        ray_origin: Vec3,
        ray_dir: Vec3,
        sampler: &mut dyn Sampler,
    ) -> Vec3 {
        let mut throughput = Vec3::new(1.0, 1.0, 1.0);
        let mut radiance = Vec3::new(0.0, 0.0, 0.0);
        let mut origin = ray_origin;
        let mut direction = ray_dir;

        for bounce in 0..self.config.max_bounces {
            let hit = scene.intersect(origin, direction);
            let hit = match hit {
                Some(h) => h,
                None => {
                    // Sky / environment contribution
                    let sky = scene.environment(direction);
                    radiance = radiance + throughput * sky;
                    break;
                }
            };

            let material = &scene.materials[hit.material_id as usize];

            // Add emission (only on first bounce or via MIS for subsequent bounces)
            if material.is_emissive() {
                let e = material.emission * material.emission_strength;
                radiance = radiance + throughput * e;
                break;
            }

            // Russian roulette — use luminance-based survival probability as in Cycles.
            if bounce >= self.config.russian_roulette_depth {
                let lum = 0.2126 * throughput.x + 0.7152 * throughput.y + 0.0722 * throughput.z;
                let p = lum.min(0.95).max(0.05);
                if sampler.next_1d() > p {
                    break;
                }
                throughput = throughput * (1.0 / p);
            }

            let normal = hit.normal;
            let hit_pos = hit.position + normal * 0.0001;

            // Build local frame from normal.
            let tangent = if normal.x.abs() > 0.9 {
                Vec3::new(0.0, 1.0, 0.0).cross(normal).normalize()
            } else {
                Vec3::new(1.0, 0.0, 0.0).cross(normal).normalize()
            };
            let bitangent = normal.cross(tangent);

            // --- Next Event Estimation (direct light sampling) ---
            if !scene.lights.is_empty() {
                // Pick one light uniformly at random.
                let light_idx = (sampler.next_1d() * scene.lights.len() as f32)
                    .min(scene.lights.len() as f32 - 1.0) as usize;
                let light_pick_pdf = 1.0 / scene.lights.len() as f32;

                let (light_dir, light_dist, light_intensity, light_pdf) =
                    Self::sample_light(&scene.lights[light_idx], hit_pos, sampler);

                let cos_theta_light = normal.dot(light_dir);
                if cos_theta_light > 0.0 && light_pdf > 0.0 {
                    // Shadow ray test.
                    let shadow_hit = scene.intersect(hit_pos, light_dir);
                    let in_shadow = match shadow_hit {
                        Some(ref sh) => sh.t < light_dist - 0.001,
                        None => false,
                    };

                    if !in_shadow {
                        // Lambertian BSDF: albedo / PI
                        let bsdf_val = material.albedo * (1.0 / std::f32::consts::PI);
                        let bsdf_pdf = cosine_hemisphere_pdf(cos_theta_light);

                        // MIS balance heuristic: w = pdf_light / (pdf_light + pdf_bsdf)
                        let combined_light_pdf = light_pdf * light_pick_pdf;
                        let mis_weight = combined_light_pdf / (combined_light_pdf + bsdf_pdf);

                        let contrib = light_intensity * bsdf_val * cos_theta_light
                            * (mis_weight / combined_light_pdf);
                        radiance = radiance + throughput * contrib;
                    }
                }
            }

            // --- BSDF sampling (indirect) ---
            let (u1, u2) = sampler.next_2d();
            let local_dir = cosine_hemisphere_sample(u1, u2);

            direction = tangent * local_dir.x + normal * local_dir.y + bitangent * local_dir.z;
            direction = direction.normalize();

            // For Lambertian: throughput *= BSDF * cos_theta / PDF
            // BSDF = albedo/PI, cos_theta = local_dir.y, PDF = cos_theta/PI
            // Result: throughput *= albedo (the PI and cos_theta cancel)
            throughput = throughput * material.albedo;

            // Offset origin to avoid self-intersection
            origin = hit_pos;

            // Clamp fireflies
            let max_comp = throughput.x.max(throughput.y).max(throughput.z);
            if max_comp > self.config.clamp_value {
                throughput = throughput * (self.config.clamp_value / max_comp);
            }
        }

        radiance
    }

    /// Sample a light source and return (direction, distance, intensity_color, pdf).
    fn sample_light(
        light: &Light,
        hit_pos: Vec3,
        _sampler: &mut dyn Sampler,
    ) -> (Vec3, f32, Vec3, f32) {
        match light {
            Light::Point(pl) => {
                let to_light = pl.position - hit_pos;
                let dist = to_light.length();
                let dir = to_light * (1.0 / dist);
                let intensity = pl.color * pl.attenuation(dist);
                // Point light has a delta PDF; we use 1.0 and the geometry term handles it.
                (dir, dist, intensity, 1.0)
            }
            Light::Directional(dl) => {
                let dir = dl.direction * -1.0;
                let intensity = dl.color * dl.intensity;
                (dir, f32::MAX, intensity, 1.0)
            }
            Light::Spot(sl) => {
                let to_light = sl.position - hit_pos;
                let dist = to_light.length();
                let dir = to_light * (1.0 / dist);
                let atten = sl.intensity / (dist * dist).max(0.0001);
                let angular = sl.angular_attenuation(dir * -1.0);
                let intensity = sl.color * atten * angular;
                (dir, dist, intensity, 1.0)
            }
        }
    }
}

/// Hit information from a ray-scene intersection.
#[derive(Debug, Clone)]
pub struct HitInfo {
    pub position: Vec3,
    pub normal: Vec3,
    pub uv: (f32, f32),
    pub t: f32,
    pub material_id: u32,
}
