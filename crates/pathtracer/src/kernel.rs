use forge3d_render_core::camera::Camera;
use forge3d_render_core::film::Film;
use forge3d_render_core::sampling::{HaltonSampler, Sampler};

use crate::integrator::PathIntegrator;
use crate::scene::PathtracerScene;
use crate::tile::Tile;

/// A ray-tracing kernel that processes tiles.
pub struct RayKernel {
    pub integrator: PathIntegrator,
    pub samples_per_pixel: u32,
}

impl RayKernel {
    pub fn new(integrator: PathIntegrator, samples_per_pixel: u32) -> Self {
        Self {
            integrator,
            samples_per_pixel,
        }
    }

    /// Render a single tile into the film.
    pub fn render_tile(
        &self,
        scene: &PathtracerScene,
        camera: &Camera,
        film: &mut Film,
        tile: &Tile,
    ) {
        let mut sampler = HaltonSampler::new(0);

        for y in tile.y..tile.y + tile.height {
            for x in tile.x..tile.x + tile.width {
                if x >= film.width || y >= film.height {
                    continue;
                }

                let pixel_index = y * film.width + x;
                sampler.start_pixel(pixel_index);

                for _s in 0..self.samples_per_pixel {
                    let (jx, jy) = sampler.next_2d();
                    let u = (x as f32 + jx) / film.width as f32;
                    let v = (y as f32 + jy) / film.height as f32;

                    let ray_dir = camera.ray_direction(u, 1.0 - v);
                    let color = self.integrator.trace(
                        scene,
                        camera.position,
                        ray_dir,
                        &mut sampler,
                    );

                    film.add_sample(x, y, [color.x, color.y, color.z]);
                }
            }
        }
    }

    /// Render a single sample for every pixel in the tile (progressive mode).
    pub fn render_tile_progressive(
        &self,
        scene: &PathtracerScene,
        camera: &Camera,
        film: &mut Film,
        tile: &Tile,
        sample_index: u32,
    ) {
        let mut sampler = HaltonSampler::new(sample_index);

        for y in tile.y..tile.y + tile.height {
            for x in tile.x..tile.x + tile.width {
                if x >= film.width || y >= film.height {
                    continue;
                }

                let pixel_index = y * film.width + x;
                sampler.start_pixel(pixel_index + sample_index * film.width * film.height);

                let (jx, jy) = sampler.next_2d();
                let u = (x as f32 + jx) / film.width as f32;
                let v = (y as f32 + jy) / film.height as f32;

                let ray_dir = camera.ray_direction(u, 1.0 - v);
                let color = self.integrator.trace(
                    scene,
                    camera.position,
                    ray_dir,
                    &mut sampler,
                );

                film.add_sample(x, y, [color.x, color.y, color.z]);
            }
        }
    }
}
