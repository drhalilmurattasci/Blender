/// Preset sampler configurations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SamplerPreset {
    Linear,
    Nearest,
    Anisotropic,
    Shadow,
}

impl SamplerPreset {
    /// Create a wgpu sampler descriptor from this preset.
    pub fn descriptor(&self) -> wgpu::SamplerDescriptor<'static> {
        match self {
            SamplerPreset::Linear => wgpu::SamplerDescriptor {
                label: Some("linear_sampler"),
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::FilterMode::Linear,
                address_mode_u: wgpu::AddressMode::Repeat,
                address_mode_v: wgpu::AddressMode::Repeat,
                address_mode_w: wgpu::AddressMode::Repeat,
                ..Default::default()
            },
            SamplerPreset::Nearest => wgpu::SamplerDescriptor {
                label: Some("nearest_sampler"),
                mag_filter: wgpu::FilterMode::Nearest,
                min_filter: wgpu::FilterMode::Nearest,
                mipmap_filter: wgpu::FilterMode::Nearest,
                address_mode_u: wgpu::AddressMode::Repeat,
                address_mode_v: wgpu::AddressMode::Repeat,
                address_mode_w: wgpu::AddressMode::Repeat,
                ..Default::default()
            },
            SamplerPreset::Anisotropic => wgpu::SamplerDescriptor {
                label: Some("aniso_sampler"),
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::FilterMode::Linear,
                address_mode_u: wgpu::AddressMode::Repeat,
                address_mode_v: wgpu::AddressMode::Repeat,
                address_mode_w: wgpu::AddressMode::Repeat,
                anisotropy_clamp: 16,
                ..Default::default()
            },
            SamplerPreset::Shadow => wgpu::SamplerDescriptor {
                label: Some("shadow_sampler"),
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::FilterMode::Nearest,
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                compare: Some(wgpu::CompareFunction::LessEqual),
                ..Default::default()
            },
        }
    }

    /// Create a wgpu sampler from this preset.
    pub fn create(&self, device: &wgpu::Device) -> wgpu::Sampler {
        device.create_sampler(&self.descriptor())
    }
}
