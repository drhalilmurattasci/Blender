use forge3d_math::Vec3;

/// Axis-aligned bounding box.
#[derive(Debug, Clone, Copy)]
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
}

impl Aabb {
    /// Create a new AABB from min and max corners.
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    /// Create an empty (invalid) AABB.
    pub fn empty() -> Self {
        Self {
            min: Vec3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY),
            max: Vec3::new(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY),
        }
    }

    /// Expand this AABB to include a point.
    pub fn expand_point(&mut self, p: Vec3) {
        self.min = Vec3::new(self.min.x.min(p.x), self.min.y.min(p.y), self.min.z.min(p.z));
        self.max = Vec3::new(self.max.x.max(p.x), self.max.y.max(p.y), self.max.z.max(p.z));
    }

    /// Expand this AABB to include another AABB.
    pub fn expand_aabb(&mut self, other: &Aabb) {
        self.expand_point(other.min);
        self.expand_point(other.max);
    }

    /// Get the center of the AABB.
    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    /// Get the extent (half-size) of the AABB.
    pub fn extent(&self) -> Vec3 {
        (self.max - self.min) * 0.5
    }

    /// Get the surface area of the AABB (returns 0 for empty/degenerate AABBs).
    pub fn surface_area(&self) -> f32 {
        let d = self.max - self.min;
        if d.x < 0.0 || d.y < 0.0 || d.z < 0.0 {
            return 0.0;
        }
        2.0 * (d.x * d.y + d.y * d.z + d.z * d.x)
    }

    /// Return the axis with the largest extent (0=x, 1=y, 2=z).
    pub fn largest_axis(&self) -> usize {
        let d = self.max - self.min;
        if d.x > d.y && d.x > d.z {
            0
        } else if d.y > d.z {
            1
        } else {
            2
        }
    }

    /// Ray-AABB intersection test (NaN-safe). Returns (t_near, t_far) or None.
    ///
    /// Uses the slab method with careful min/max ordering to handle edge cases
    /// where the ray origin lies on a slab boundary with zero direction component
    /// (which produces 0*inf=NaN). We treat NaN as a miss.
    pub fn intersect_ray(&self, origin: Vec3, inv_dir: Vec3) -> Option<(f32, f32)> {
        let t1 = (self.min.x - origin.x) * inv_dir.x;
        let t2 = (self.max.x - origin.x) * inv_dir.x;
        let t3 = (self.min.y - origin.y) * inv_dir.y;
        let t4 = (self.max.y - origin.y) * inv_dir.y;
        let t5 = (self.min.z - origin.z) * inv_dir.z;
        let t6 = (self.max.z - origin.z) * inv_dir.z;

        let t_near = t1.min(t2).max(t3.min(t4)).max(t5.min(t6));
        let t_far = t1.max(t2).min(t3.max(t4)).min(t5.max(t6));

        // NaN-safe: NaN comparisons return false, so NaN in t_near or t_far
        // causes both conditions to fail, correctly returning None.
        if t_near <= t_far && t_far >= 0.0 {
            Some((t_near, t_far))
        } else {
            None
        }
    }
}

/// A node in the BVH tree.
#[derive(Debug, Clone)]
pub enum BvhNode {
    Leaf {
        bounds: Aabb,
        first_prim: u32,
        prim_count: u32,
    },
    Interior {
        bounds: Aabb,
        left: Box<BvhNode>,
        right: Box<BvhNode>,
        split_axis: usize,
    },
}

impl BvhNode {
    /// Get the bounding box of this node.
    pub fn bounds(&self) -> &Aabb {
        match self {
            BvhNode::Leaf { bounds, .. } => bounds,
            BvhNode::Interior { bounds, .. } => bounds,
        }
    }
}

/// A complete BVH acceleration structure.
pub struct BvhTree {
    pub root: Option<BvhNode>,
    pub primitive_indices: Vec<u32>,
}

impl BvhTree {
    /// Build a BVH from a set of AABBs using the SAH heuristic.
    pub fn build(aabbs: &[Aabb]) -> Self {
        if aabbs.is_empty() {
            return Self {
                root: None,
                primitive_indices: Vec::new(),
            };
        }

        let mut indices: Vec<u32> = (0..aabbs.len() as u32).collect();
        let root = Self::build_recursive(aabbs, &mut indices, 0, aabbs.len());

        Self {
            root: Some(root),
            primitive_indices: indices,
        }
    }

    /// SAH cost constants matching Cycles' defaults.
    const TRAVERSAL_COST: f32 = 1.0;
    const INTERSECT_COST: f32 = 1.0;
    const NUM_BINS: usize = 12;

    fn build_recursive(
        aabbs: &[Aabb],
        indices: &mut [u32],
        start: usize,
        end: usize,
    ) -> BvhNode {
        let count = end - start;

        // Compute bounds of all primitives and centroid bounds for binning.
        let mut bounds = Aabb::empty();
        let mut centroid_bounds = Aabb::empty();
        for &idx in &indices[start..end] {
            bounds.expand_aabb(&aabbs[idx as usize]);
            centroid_bounds.expand_point(aabbs[idx as usize].center());
        }

        // Leaf node for small primitive counts.
        if count <= 4 {
            return BvhNode::Leaf {
                bounds,
                first_prim: start as u32,
                prim_count: count as u32,
            };
        }

        let parent_sa = bounds.surface_area();
        // If parent surface area is zero or degenerate (all primitives overlap at a
        // point/line/plane), SAH ratios are meaningless -- fall back to a leaf or
        // midpoint split.
        if parent_sa <= 0.0 {
            return BvhNode::Leaf {
                bounds,
                first_prim: start as u32,
                prim_count: count as u32,
            };
        }
        let leaf_cost = Self::INTERSECT_COST * count as f32;

        // Evaluate SAH over all three axes using binning.
        let mut best_cost = f32::MAX;
        let mut best_axis = 0usize;
        let mut best_split_bin = 0usize;

        for axis in 0..3 {
            let (cmin, cmax) = match axis {
                0 => (centroid_bounds.min.x, centroid_bounds.max.x),
                1 => (centroid_bounds.min.y, centroid_bounds.max.y),
                _ => (centroid_bounds.min.z, centroid_bounds.max.z),
            };

            // Skip degenerate axis.
            if (cmax - cmin).abs() < 1e-8 {
                continue;
            }

            // Initialize bins.
            let mut bin_counts = [0u32; Self::NUM_BINS];
            let mut bin_bounds = [Aabb::empty(); Self::NUM_BINS];

            let scale = Self::NUM_BINS as f32 / (cmax - cmin);

            // Assign primitives to bins.
            for &idx in &indices[start..end] {
                let c = aabbs[idx as usize].center();
                let cv = match axis {
                    0 => c.x,
                    1 => c.y,
                    _ => c.z,
                };
                let bin = ((cv - cmin) * scale).min(Self::NUM_BINS as f32 - 1.0).max(0.0) as usize;
                bin_counts[bin] += 1;
                bin_bounds[bin].expand_aabb(&aabbs[idx as usize]);
            }

            // Sweep from left to right to compute prefix bounds and counts.
            let mut left_counts = [0u32; Self::NUM_BINS - 1];
            let mut left_bounds = [Aabb::empty(); Self::NUM_BINS - 1];
            let mut acc_bounds = Aabb::empty();
            let mut acc_count = 0u32;

            for i in 0..(Self::NUM_BINS - 1) {
                acc_count += bin_counts[i];
                acc_bounds.expand_aabb(&bin_bounds[i]);
                left_counts[i] = acc_count;
                left_bounds[i] = acc_bounds;
            }

            // Sweep from right to left and evaluate SAH cost at each split.
            acc_bounds = Aabb::empty();
            acc_count = 0;

            for i in (0..(Self::NUM_BINS - 1)).rev() {
                acc_count += bin_counts[i + 1];
                acc_bounds.expand_aabb(&bin_bounds[i + 1]);

                if left_counts[i] == 0 || acc_count == 0 {
                    continue;
                }

                let left_sa = left_bounds[i].surface_area();
                let right_sa = acc_bounds.surface_area();

                // SAH cost = traverse_cost + (SA_left/SA_parent)*N_left*intersect_cost
                //          + (SA_right/SA_parent)*N_right*intersect_cost
                let cost = Self::TRAVERSAL_COST
                    + (left_sa / parent_sa) * left_counts[i] as f32 * Self::INTERSECT_COST
                    + (right_sa / parent_sa) * acc_count as f32 * Self::INTERSECT_COST;

                if cost < best_cost {
                    best_cost = cost;
                    best_axis = axis;
                    best_split_bin = i;
                }
            }
        }

        // If no split improves over the leaf cost, create a leaf.
        if best_cost >= leaf_cost {
            return BvhNode::Leaf {
                bounds,
                first_prim: start as u32,
                prim_count: count as u32,
            };
        }

        // Partition primitives according to the best split.
        let (cmin, cmax) = match best_axis {
            0 => (centroid_bounds.min.x, centroid_bounds.max.x),
            1 => (centroid_bounds.min.y, centroid_bounds.max.y),
            _ => (centroid_bounds.min.z, centroid_bounds.max.z),
        };
        let scale = Self::NUM_BINS as f32 / (cmax - cmin);

        // Lomuto partition: elements with bin <= best_split_bin go left.
        let slice = &mut indices[start..end];
        let mut left_end = 0usize;
        for i in 0..slice.len() {
            let c = aabbs[slice[i] as usize].center();
            let cv = match best_axis {
                0 => c.x,
                1 => c.y,
                _ => c.z,
            };
            let bin = ((cv - cmin) * scale).min(Self::NUM_BINS as f32 - 1.0).max(0.0) as usize;
            if bin <= best_split_bin {
                slice.swap(i, left_end);
                left_end += 1;
            }
        }

        let mid = start + left_end;

        // Fallback: if partition ended up degenerate, split in half.
        let mid = if mid == start || mid == end {
            let axis = best_axis;
            indices[start..end].sort_by(|&a, &b| {
                let ca = aabbs[a as usize].center();
                let cb = aabbs[b as usize].center();
                let va = match axis {
                    0 => ca.x,
                    1 => ca.y,
                    _ => ca.z,
                };
                let vb = match axis {
                    0 => cb.x,
                    1 => cb.y,
                    _ => cb.z,
                };
                va.partial_cmp(&vb).unwrap_or(std::cmp::Ordering::Equal)
            });
            start + count / 2
        } else {
            mid
        };

        let left = Self::build_recursive(aabbs, indices, start, mid);
        let right = Self::build_recursive(aabbs, indices, mid, end);

        BvhNode::Interior {
            bounds,
            left: Box::new(left),
            right: Box::new(right),
            split_axis: best_axis,
        }
    }
}
