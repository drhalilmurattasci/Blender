/// Helpers for working with texture formats.

/// Returns the byte size per pixel for common texture formats.
pub fn bytes_per_pixel(format: wgpu::TextureFormat) -> u32 {
    match format {
        wgpu::TextureFormat::R8Unorm | wgpu::TextureFormat::R8Snorm | wgpu::TextureFormat::R8Uint | wgpu::TextureFormat::R8Sint => 1,
        wgpu::TextureFormat::Rg8Unorm | wgpu::TextureFormat::Rg8Snorm | wgpu::TextureFormat::Rg8Uint | wgpu::TextureFormat::Rg8Sint => 2,
        wgpu::TextureFormat::R16Float | wgpu::TextureFormat::R16Uint | wgpu::TextureFormat::R16Sint => 2,
        wgpu::TextureFormat::Rgba8Unorm | wgpu::TextureFormat::Rgba8UnormSrgb | wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Bgra8UnormSrgb => 4,
        wgpu::TextureFormat::Rgba8Snorm | wgpu::TextureFormat::Rgba8Uint | wgpu::TextureFormat::Rgba8Sint => 4,
        wgpu::TextureFormat::R32Float | wgpu::TextureFormat::R32Uint | wgpu::TextureFormat::R32Sint => 4,
        wgpu::TextureFormat::Rg16Float | wgpu::TextureFormat::Rg16Uint | wgpu::TextureFormat::Rg16Sint => 4,
        wgpu::TextureFormat::Rgba16Float | wgpu::TextureFormat::Rgba16Uint | wgpu::TextureFormat::Rgba16Sint => 8,
        wgpu::TextureFormat::Rg32Float | wgpu::TextureFormat::Rg32Uint | wgpu::TextureFormat::Rg32Sint => 8,
        wgpu::TextureFormat::Rgba32Float | wgpu::TextureFormat::Rgba32Uint | wgpu::TextureFormat::Rgba32Sint => 16,
        wgpu::TextureFormat::Depth32Float => 4,
        wgpu::TextureFormat::Depth24Plus => 4,
        wgpu::TextureFormat::Depth24PlusStencil8 => 4,
        _ => 4, // conservative default
    }
}

/// Returns true if the format is a depth or depth-stencil format.
pub fn is_depth_format(format: wgpu::TextureFormat) -> bool {
    matches!(
        format,
        wgpu::TextureFormat::Depth16Unorm
            | wgpu::TextureFormat::Depth32Float
            | wgpu::TextureFormat::Depth24Plus
            | wgpu::TextureFormat::Depth24PlusStencil8
            | wgpu::TextureFormat::Depth32FloatStencil8
    )
}

/// Returns true if the format is an sRGB format.
pub fn is_srgb(format: wgpu::TextureFormat) -> bool {
    matches!(
        format,
        wgpu::TextureFormat::Rgba8UnormSrgb | wgpu::TextureFormat::Bgra8UnormSrgb
    )
}

/// Returns an HDR-capable color format.
pub fn hdr_color_format() -> wgpu::TextureFormat {
    wgpu::TextureFormat::Rgba16Float
}

/// Returns the default depth format.
pub fn default_depth_format() -> wgpu::TextureFormat {
    wgpu::TextureFormat::Depth32Float
}
