use forge3d_math::Vec3;
use forge3d_render_core::bvh::{Aabb, BvhNode, BvhTree};
use forge3d_render_core::light::Light;
use forge3d_render_core::material::Material;

use crate::integrator::HitInfo;

/// A triangle in the scene.
#[derive(Debug, Clone)]
pub struct Triangle {
    pub v0: Vec3,
    pub v1: Vec3,
    pub v2: Vec3,
    pub n0: Vec3,
    pub n1: Vec3,
    pub n2: Vec3,
    pub uv0: (f32, f32),
    pub uv1: (f32, f32),
    pub uv2: (f32, f32),
    pub material_id: u32,
}

impl Triangle {
    /// Compute the AABB of this triangle.
    pub fn aabb(&self) -> Aabb {
        let mut aabb = Aabb::empty();
        aabb.expand_point(self.v0);
        aabb.expand_point(self.v1);
        aabb.expand_point(self.v2);
        aabb
    }

    /// Moller-Trumbore ray-triangle intersection.
    pub fn intersect(&self, origin: Vec3, direction: Vec3) -> Option<(f32, f32, f32)> {
        let edge1 = self.v1 - self.v0;
        let edge2 = self.v2 - self.v0;
        let h = direction.cross(edge2);
        let a = edge1.dot(h);

        if a.abs() < 1e-8 {
            return None;
        }

        let f = 1.0 / a;
        let s = origin - self.v0;
        let u = f * s.dot(h);

        if !(0.0..=1.0).contains(&u) {
            return None;
        }

        let q = s.cross(edge1);
        let v = f * direction.dot(q);

        if v < 0.0 || u + v > 1.0 {
            return None;
        }

        let t = f * edge2.dot(q);

        if t > 1e-6 {
            Some((t, u, v))
        } else {
            None
        }
    }
}

/// Scene representation for the path tracer.
pub struct PathtracerScene {
    pub triangles: Vec<Triangle>,
    pub materials: Vec<Material>,
    pub lights: Vec<Light>,
    pub bvh: Option<BvhTree>,
    pub environment_color: Vec3,
}

impl PathtracerScene {
    pub fn new() -> Self {
        Self {
            triangles: Vec::new(),
            materials: Vec::new(),
            lights: Vec::new(),
            bvh: None,
            environment_color: Vec3::new(0.1, 0.1, 0.15),
        }
    }

    /// Add a triangle to the scene.
    pub fn add_triangle(&mut self, triangle: Triangle) {
        self.triangles.push(triangle);
    }

    /// Add a material, returning its index.
    pub fn add_material(&mut self, material: Material) -> u32 {
        let idx = self.materials.len() as u32;
        self.materials.push(material);
        idx
    }

    /// Add a light to the scene.
    pub fn add_light(&mut self, light: Light) {
        self.lights.push(light);
    }

    /// Build the BVH acceleration structure.
    pub fn build_bvh(&mut self) {
        let aabbs: Vec<Aabb> = self.triangles.iter().map(|t| t.aabb()).collect();
        self.bvh = Some(BvhTree::build(&aabbs));
    }

    /// Intersect a ray with the scene using BVH traversal, returning the closest hit.
    pub fn intersect(&self, origin: Vec3, direction: Vec3) -> Option<HitInfo> {
        let inv_dir = Vec3::new(1.0 / direction.x, 1.0 / direction.y, 1.0 / direction.z);

        if let Some(ref bvh) = self.bvh {
            if let Some(ref root) = bvh.root {
                let mut closest_t = f32::MAX;
                let mut closest_hit: Option<HitInfo> = None;
                self.intersect_bvh_node(root, origin, direction, inv_dir, &bvh.primitive_indices, &mut closest_t, &mut closest_hit);
                return closest_hit;
            }
        }

        // Fallback: brute-force when no BVH is built.
        self.intersect_brute_force(origin, direction)
    }

    fn intersect_bvh_node(
        &self,
        node: &BvhNode,
        origin: Vec3,
        direction: Vec3,
        inv_dir: Vec3,
        prim_indices: &[u32],
        closest_t: &mut f32,
        closest_hit: &mut Option<HitInfo>,
    ) {
        match node {
            BvhNode::Leaf { first_prim, prim_count, bounds, .. } => {
                if bounds.intersect_ray(origin, inv_dir).is_none() {
                    return;
                }
                let start = *first_prim as usize;
                let end = start + *prim_count as usize;
                for &idx in &prim_indices[start..end] {
                    let tri = &self.triangles[idx as usize];
                    if let Some((t, u, v)) = tri.intersect(origin, direction) {
                        if t < *closest_t {
                            *closest_t = t;
                            let w = 1.0 - u - v;
                            let normal = (tri.n0 * w + tri.n1 * u + tri.n2 * v).normalize();
                            let uv_x = tri.uv0.0 * w + tri.uv1.0 * u + tri.uv2.0 * v;
                            let uv_y = tri.uv0.1 * w + tri.uv1.1 * u + tri.uv2.1 * v;
                            *closest_hit = Some(HitInfo {
                                position: origin + direction * t,
                                normal,
                                uv: (uv_x, uv_y),
                                t,
                                material_id: tri.material_id,
                            });
                        }
                    }
                }
            }
            BvhNode::Interior { bounds, left, right, split_axis } => {
                if bounds.intersect_ray(origin, inv_dir).is_none() {
                    return;
                }

                // Traverse near child first based on ray direction for early termination.
                let dir_component = match split_axis {
                    0 => direction.x,
                    1 => direction.y,
                    _ => direction.z,
                };
                let (first, second) = if dir_component >= 0.0 {
                    (left.as_ref(), right.as_ref())
                } else {
                    (right.as_ref(), left.as_ref())
                };

                self.intersect_bvh_node(first, origin, direction, inv_dir, prim_indices, closest_t, closest_hit);

                // Only traverse far child if its bounding box is closer than the best hit.
                if let Some((t_near, _)) = second.bounds().intersect_ray(origin, inv_dir) {
                    if t_near < *closest_t {
                        self.intersect_bvh_node(second, origin, direction, inv_dir, prim_indices, closest_t, closest_hit);
                    }
                }
            }
        }
    }

    /// Brute-force intersection fallback.
    fn intersect_brute_force(&self, origin: Vec3, direction: Vec3) -> Option<HitInfo> {
        let mut closest_t = f32::MAX;
        let mut closest_hit: Option<HitInfo> = None;

        for tri in &self.triangles {
            if let Some((t, u, v)) = tri.intersect(origin, direction) {
                if t < closest_t {
                    closest_t = t;
                    let w = 1.0 - u - v;
                    let normal = (tri.n0 * w + tri.n1 * u + tri.n2 * v).normalize();
                    let uv_x = tri.uv0.0 * w + tri.uv1.0 * u + tri.uv2.0 * v;
                    let uv_y = tri.uv0.1 * w + tri.uv1.1 * u + tri.uv2.1 * v;

                    closest_hit = Some(HitInfo {
                        position: origin + direction * t,
                        normal,
                        uv: (uv_x, uv_y),
                        t,
                        material_id: tri.material_id,
                    });
                }
            }
        }

        closest_hit
    }

    /// Get the environment color for a given direction.
    pub fn environment(&self, _direction: Vec3) -> Vec3 {
        self.environment_color
    }
}

impl Default for PathtracerScene {
    fn default() -> Self {
        Self::new()
    }
}
