/// A film buffer that accumulates radiance samples.
pub struct Film {
    pub width: u32,
    pub height: u32,
    /// RGB accumulated radiance per pixel.
    pub pixels: Vec<[f32; 3]>,
    /// Sample counts per pixel.
    pub sample_counts: Vec<u32>,
}

impl Film {
    /// Create a new film with the given resolution.
    pub fn new(width: u32, height: u32) -> Self {
        let count = (width * height) as usize;
        Self {
            width,
            height,
            pixels: vec![[0.0; 3]; count],
            sample_counts: vec![0; count],
        }
    }

    /// Add a radiance sample at the given pixel.
    pub fn add_sample(&mut self, x: u32, y: u32, color: [f32; 3]) {
        let idx = (y * self.width + x) as usize;
        if idx < self.pixels.len() {
            self.pixels[idx][0] += color[0];
            self.pixels[idx][1] += color[1];
            self.pixels[idx][2] += color[2];
            self.sample_counts[idx] += 1;
        }
    }

    /// Get the averaged color at a pixel.
    pub fn get_pixel(&self, x: u32, y: u32) -> [f32; 3] {
        let idx = (y * self.width + x) as usize;
        let count = self.sample_counts[idx].max(1) as f32;
        [
            self.pixels[idx][0] / count,
            self.pixels[idx][1] / count,
            self.pixels[idx][2] / count,
        ]
    }

    /// Clear the film.
    pub fn clear(&mut self) {
        self.pixels.fill([0.0; 3]);
        self.sample_counts.fill(0);
    }

    /// Get the total number of pixels.
    pub fn pixel_count(&self) -> usize {
        (self.width * self.height) as usize
    }

    /// Convert the film to an 8-bit RGBA image buffer with sRGB gamma encoding.
    pub fn to_rgba8(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(self.pixel_count() * 4);
        for y in 0..self.height {
            for x in 0..self.width {
                let c = self.get_pixel(x, y);
                buf.push(Self::linear_to_srgb(c[0]));
                buf.push(Self::linear_to_srgb(c[1]));
                buf.push(Self::linear_to_srgb(c[2]));
                buf.push(255);
            }
        }
        buf
    }

    /// Convert a linear-space value to sRGB gamma-encoded byte.
    fn linear_to_srgb(c: f32) -> u8 {
        let c = c.clamp(0.0, 1.0);
        let srgb = if c <= 0.0031308 {
            12.92 * c
        } else {
            1.055 * c.powf(1.0 / 2.4) - 0.055
        };
        (srgb * 255.0 + 0.5) as u8
    }

    /// Apply simple Reinhard tone mapping in place.
    pub fn tone_map_reinhard(&self) -> Vec<[f32; 3]> {
        self.pixels
            .iter()
            .zip(&self.sample_counts)
            .map(|(px, &count)| {
                let c = if count > 0 {
                    let n = count as f32;
                    [px[0] / n, px[1] / n, px[2] / n]
                } else {
                    [0.0; 3]
                };
                [
                    c[0] / (1.0 + c[0]),
                    c[1] / (1.0 + c[1]),
                    c[2] / (1.0 + c[2]),
                ]
            })
            .collect()
    }
}
