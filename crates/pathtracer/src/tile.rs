/// A rectangular tile for work distribution.
#[derive(Debug, Clone)]
pub struct Tile {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

/// Generates and schedules tiles for rendering.
pub struct TileScheduler {
    pub image_width: u32,
    pub image_height: u32,
    pub tile_size: u32,
}

impl TileScheduler {
    pub fn new(image_width: u32, image_height: u32, tile_size: u32) -> Self {
        Self {
            image_width,
            image_height,
            tile_size,
        }
    }

    /// Generate all tiles covering the image.
    pub fn generate_tiles(&self) -> Vec<Tile> {
        let mut tiles = Vec::new();
        let mut y = 0;
        while y < self.image_height {
            let mut x = 0;
            while x < self.image_width {
                let w = (self.tile_size).min(self.image_width - x);
                let h = (self.tile_size).min(self.image_height - y);
                tiles.push(Tile {
                    x,
                    y,
                    width: w,
                    height: h,
                });
                x += self.tile_size;
            }
            y += self.tile_size;
        }
        tiles
    }

    /// Generate tiles in a spiral order from the center outward.
    pub fn generate_tiles_spiral(&self) -> Vec<Tile> {
        let mut tiles = self.generate_tiles();

        let center_x = self.image_width as f32 / 2.0;
        let center_y = self.image_height as f32 / 2.0;

        tiles.sort_by(|a, b| {
            let da = {
                let dx = (a.x as f32 + a.width as f32 / 2.0) - center_x;
                let dy = (a.y as f32 + a.height as f32 / 2.0) - center_y;
                dx * dx + dy * dy
            };
            let db = {
                let dx = (b.x as f32 + b.width as f32 / 2.0) - center_x;
                let dy = (b.y as f32 + b.height as f32 / 2.0) - center_y;
                dx * dx + dy * dy
            };
            da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
        });

        tiles
    }

    /// Number of tiles.
    pub fn tile_count(&self) -> u32 {
        let cols = (self.image_width + self.tile_size - 1) / self.tile_size;
        let rows = (self.image_height + self.tile_size - 1) / self.tile_size;
        cols * rows
    }
}
