use forge3d_math::Vec3;

/// Trait for sample generators.
pub trait Sampler {
    /// Generate the next 1D sample in [0, 1).
    fn next_1d(&mut self) -> f32;

    /// Generate the next 2D sample in [0, 1)^2.
    fn next_2d(&mut self) -> (f32, f32) {
        (self.next_1d(), self.next_1d())
    }
}

/// Halton low-discrepancy sequence sampler.
pub struct HaltonSampler {
    index: u32,
    dimension: u32,
}

impl HaltonSampler {
    pub fn new(start_index: u32) -> Self {
        Self {
            index: start_index,
            dimension: 0,
        }
    }

    /// Reset for a new pixel.
    pub fn start_pixel(&mut self, index: u32) {
        self.index = index;
        self.dimension = 0;
    }

    pub fn halton(index: u32, base: u32) -> f32 {
        let mut f = 1.0f32;
        let mut r = 0.0f32;
        let mut i = index;
        let b = base as f32;

        while i > 0 {
            f /= b;
            r += f * (i % base) as f32;
            i /= base;
        }

        r
    }

    const PRIMES: [u32; 16] = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53];
}

impl Sampler for HaltonSampler {
    fn next_1d(&mut self) -> f32 {
        let dim = self.dimension as usize;
        let base = if dim < Self::PRIMES.len() {
            Self::PRIMES[dim]
        } else {
            2
        };
        self.dimension += 1;
        Self::halton(self.index, base)
    }
}

/// Stratified (jittered) sampler.
pub struct StratifiedSampler {
    strata_x: u32,
    strata_y: u32,
    current_x: u32,
    current_y: u32,
    rng_state: u32,
}

impl StratifiedSampler {
    pub fn new(strata_x: u32, strata_y: u32, seed: u32) -> Self {
        Self {
            strata_x,
            strata_y,
            current_x: 0,
            current_y: 0,
            // xorshift32 has a fixed point at 0, so ensure non-zero seed.
            rng_state: if seed == 0 { 1 } else { seed },
        }
    }

    /// Reset to the beginning of the strata grid.
    pub fn reset(&mut self) {
        self.current_x = 0;
        self.current_y = 0;
    }

    /// Advance to the next stratum.
    pub fn advance(&mut self) {
        self.current_x += 1;
        if self.current_x >= self.strata_x {
            self.current_x = 0;
            self.current_y += 1;
            if self.current_y >= self.strata_y {
                self.current_y = 0;
            }
        }
    }

    /// Total number of samples.
    pub fn total_samples(&self) -> u32 {
        self.strata_x * self.strata_y
    }

    fn next_rng(&mut self) -> f32 {
        // Simple xorshift32
        self.rng_state ^= self.rng_state << 13;
        self.rng_state ^= self.rng_state >> 17;
        self.rng_state ^= self.rng_state << 5;
        (self.rng_state as f32) / (u32::MAX as f32)
    }
}

impl Sampler for StratifiedSampler {
    fn next_1d(&mut self) -> f32 {
        let jitter = self.next_rng();
        (self.current_x as f32 + jitter) / self.strata_x as f32
    }

    fn next_2d(&mut self) -> (f32, f32) {
        let jx = self.next_rng();
        let jy = self.next_rng();
        let u = (self.current_x as f32 + jx) / self.strata_x as f32;
        let v = (self.current_y as f32 + jy) / self.strata_y as f32;
        (u, v)
    }
}

/// Sample a point on a hemisphere with cosine-weighted distribution.
pub fn cosine_hemisphere_sample(u1: f32, u2: f32) -> Vec3 {
    let r = u1.sqrt();
    let theta = 2.0 * std::f32::consts::PI * u2;
    let x = r * theta.cos();
    let z = r * theta.sin();
    let y = (1.0 - u1).sqrt();
    Vec3::new(x, y, z)
}

/// Sample a point uniformly on a sphere.
pub fn uniform_sphere_sample(u1: f32, u2: f32) -> Vec3 {
    let z = 1.0 - 2.0 * u1;
    let r = (1.0 - z * z).sqrt();
    let theta = 2.0 * std::f32::consts::PI * u2;
    Vec3::new(r * theta.cos(), r * theta.sin(), z)
}

/// Compute the PDF for cosine-weighted hemisphere sampling.
pub fn cosine_hemisphere_pdf(cos_theta: f32) -> f32 {
    cos_theta / std::f32::consts::PI
}
