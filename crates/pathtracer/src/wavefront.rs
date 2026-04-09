use forge3d_math::Vec3;
use forge3d_render_core::camera::Camera;
use forge3d_render_core::film::Film;
use forge3d_render_core::sampling::{HaltonSampler, Sampler};

use crate::integrator::PathIntegrator;
use crate::scene::PathtracerScene;

/// Ray state for wavefront path tracing.
#[derive(Debug, Clone)]
pub struct RayState {
    pub origin: Vec3,
    pub direction: Vec3,
    pub throughput: Vec3,
    pub radiance: Vec3,
    pub pixel_x: u32,
    pub pixel_y: u32,
    pub bounce: u32,
    pub sample_index: u32,
    pub active: bool,
}

/// Wavefront path tracer that processes rays in bulk per bounce.
pub struct WavefrontPathtracer {
    pub integrator: PathIntegrator,
    pub max_rays: usize,
}

impl WavefrontPathtracer {
    pub fn new(integrator: PathIntegrator, max_rays: usize) -> Self {
        Self {
            integrator,
            max_rays,
        }
    }

    /// Generate primary rays for a set of pixels.
    pub fn generate_primary_rays(
        &self,
        camera: &Camera,
        film: &Film,
        sample_index: u32,
    ) -> Vec<RayState> {
        let mut rays = Vec::with_capacity(film.pixel_count().min(self.max_rays));
        let mut sampler = HaltonSampler::new(sample_index);

        for y in 0..film.height {
            for x in 0..film.width {
                if rays.len() >= self.max_rays {
                    return rays;
                }

                sampler.start_pixel(y * film.width + x + sample_index * film.width * film.height);
                let (jx, jy) = sampler.next_2d();
                let u = (x as f32 + jx) / film.width as f32;
                let v = (y as f32 + jy) / film.height as f32;

                let direction = camera.ray_direction(u, 1.0 - v);
                rays.push(RayState {
                    origin: camera.position,
                    direction,
                    throughput: Vec3::new(1.0, 1.0, 1.0),
                    radiance: Vec3::new(0.0, 0.0, 0.0),
                    pixel_x: x,
                    pixel_y: y,
                    bounce: 0,
                    sample_index,
                    active: true,
                });
            }
        }

        rays
    }

    /// Process one bounce for all active rays.
    pub fn process_bounce(
        &self,
        scene: &PathtracerScene,
        rays: &mut [RayState],
    ) {
        for ray in rays.iter_mut() {
            if !ray.active {
                continue;
            }

            let hit = scene.intersect(ray.origin, ray.direction);
            match hit {
                None => {
                    let sky = scene.environment(ray.direction);
                    ray.radiance = ray.radiance + ray.throughput * sky;
                    ray.active = false;
                }
                Some(hit_info) => {
                    let material = match scene.materials.get(hit_info.material_id as usize) {
                        Some(m) => m,
                        None => {
                            ray.active = false;
                            continue;
                        }
                    };

                    if material.is_emissive() {
                        let e = material.emission * material.emission_strength;
                        ray.radiance = ray.radiance + ray.throughput * e;
                        ray.active = false;
                        continue;
                    }

                    // Russian roulette after a few bounces.
                    if ray.bounce >= self.integrator.config.russian_roulette_depth {
                        let lum = 0.2126 * ray.throughput.x
                            + 0.7152 * ray.throughput.y
                            + 0.0722 * ray.throughput.z;
                        let p = lum.min(0.95).max(0.05);
                        // Include sample_index to decorrelate across progressive frames.
                        let rr_idx = ray.pixel_y * 10000 + ray.pixel_x
                            + ray.bounce * 200000
                            + ray.sample_index * 3000000;
                        let rr_sample = HaltonSampler::halton(rr_idx, 5);
                        if rr_sample > p {
                            ray.active = false;
                            continue;
                        }
                        ray.throughput = ray.throughput * (1.0 / p);
                    }

                    // Diffuse bounce using deterministic low-discrepancy sequence.
                    // Include sample_index to decorrelate across progressive frames.
                    let idx = ray.pixel_y * 10000 + ray.pixel_x
                        + ray.bounce * 100000
                        + ray.sample_index * 1000000;
                    let u1 = HaltonSampler::halton(idx, 2);
                    let u2 = HaltonSampler::halton(idx, 3);

                    let normal = hit_info.normal;
                    let tangent = if normal.x.abs() > 0.9 {
                        Vec3::new(0.0, 1.0, 0.0).cross(normal).normalize()
                    } else {
                        Vec3::new(1.0, 0.0, 0.0).cross(normal).normalize()
                    };
                    let bitangent = normal.cross(tangent);

                    let local = forge3d_render_core::sampling::cosine_hemisphere_sample(u1, u2);
                    ray.direction = (tangent * local.x + normal * local.y + bitangent * local.z).normalize();
                    ray.origin = hit_info.position + normal * 0.0001;
                    // For Lambertian BSDF with cosine sampling: throughput *= albedo
                    // (BSDF=albedo/PI, cos_theta and PI cancel with pdf=cos_theta/PI)
                    ray.throughput = ray.throughput * material.albedo;
                    ray.bounce += 1;

                    if ray.bounce >= self.integrator.config.max_bounces {
                        ray.active = false;
                    }
                }
            }
        }
    }

    /// Splat completed rays into the film.
    pub fn splat_results(&self, rays: &[RayState], film: &mut Film) {
        for ray in rays {
            film.add_sample(
                ray.pixel_x,
                ray.pixel_y,
                [ray.radiance.x, ray.radiance.y, ray.radiance.z],
            );
        }
    }

    /// Count active rays.
    pub fn active_count(rays: &[RayState]) -> usize {
        rays.iter().filter(|r| r.active).count()
    }
}

// Use HaltonSampler::halton() from render-core for static Halton values.
