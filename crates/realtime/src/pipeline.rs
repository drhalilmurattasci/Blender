use crate::{
    bloom::{BloomConfig, BloomPass},
    dof::{DepthOfFieldPass, DofConfig},
    gbuffer::GBufferPass,
    lighting::LightingPass,
    shadow::{ShadowConfig, ShadowPass},
    ssao::{SsaoConfig, SsaoPass},
    ssr::{SsrConfig, SsrPass},
    taa::{TaaConfig, TaaPass},
    volumetric::{VolumetricConfig, VolumetricPass},
    RealtimeResult,
};

/// Configuration for the full realtime rendering pipeline.
#[derive(Debug, Clone)]
pub struct PipelineConfig {
    pub width: u32,
    pub height: u32,
    pub enable_ssao: bool,
    pub enable_ssr: bool,
    pub enable_bloom: bool,
    pub enable_dof: bool,
    pub enable_taa: bool,
    pub enable_volumetric: bool,
    pub shadow: ShadowConfig,
    pub ssao: SsaoConfig,
    pub ssr: SsrConfig,
    pub bloom: BloomConfig,
    pub dof: DofConfig,
    pub taa: TaaConfig,
    pub volumetric: VolumetricConfig,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            width: 1920,
            height: 1080,
            enable_ssao: true,
            enable_ssr: true,
            enable_bloom: true,
            enable_dof: false,
            enable_taa: true,
            enable_volumetric: false,
            shadow: ShadowConfig::default(),
            ssao: SsaoConfig::default(),
            ssr: SsrConfig::default(),
            bloom: BloomConfig::default(),
            dof: DofConfig::default(),
            taa: TaaConfig::default(),
            volumetric: VolumetricConfig::default(),
        }
    }
}

/// The full realtime rendering pipeline combining all passes.
pub struct RealtimePipeline {
    pub config: PipelineConfig,
    pub gbuffer: GBufferPass,
    pub shadow: ShadowPass,
    pub lighting: LightingPass,
    pub ssao: Option<SsaoPass>,
    pub ssr: Option<SsrPass>,
    pub bloom: Option<BloomPass>,
    pub dof: Option<DepthOfFieldPass>,
    pub taa: Option<TaaPass>,
    pub volumetric: Option<VolumetricPass>,
}

impl RealtimePipeline {
    /// Create the full pipeline.
    pub fn new(device: &wgpu::Device, config: PipelineConfig) -> RealtimeResult<Self> {
        let gbuffer = GBufferPass::new(device, config.width, config.height);
        let shadow = ShadowPass::new(device, config.shadow.clone());
        let lighting = LightingPass::new(device, config.width, config.height);

        let ssao = if config.enable_ssao {
            Some(SsaoPass::new(device, config.width, config.height, config.ssao.clone()))
        } else {
            None
        };

        let ssr = if config.enable_ssr {
            Some(SsrPass::new(device, config.width, config.height, config.ssr.clone()))
        } else {
            None
        };

        let bloom = if config.enable_bloom {
            Some(BloomPass::new(device, config.width, config.height, config.bloom.clone()))
        } else {
            None
        };

        let dof = if config.enable_dof {
            Some(DepthOfFieldPass::new(device, config.width, config.height, config.dof.clone()))
        } else {
            None
        };

        let taa = if config.enable_taa {
            Some(TaaPass::new(device, config.width, config.height, config.taa.clone()))
        } else {
            None
        };

        let volumetric = if config.enable_volumetric {
            Some(VolumetricPass::new(device, config.width, config.height, config.volumetric.clone()))
        } else {
            None
        };

        Ok(Self {
            config,
            gbuffer,
            shadow,
            lighting,
            ssao,
            ssr,
            bloom,
            dof,
            taa,
            volumetric,
        })
    }

    /// Resize all pipeline passes.
    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        self.config.width = width;
        self.config.height = height;
        self.gbuffer.resize(device, width, height);
        self.lighting.resize(device, width, height);

        if let Some(ref mut ssao) = self.ssao {
            ssao.resize(device, width, height);
        }
        if let Some(ref mut ssr) = self.ssr {
            ssr.resize(device, width, height);
        }
        if let Some(ref mut bloom) = self.bloom {
            bloom.resize(device, width, height);
        }
        if let Some(ref mut dof) = self.dof {
            dof.resize(device, width, height);
        }
        if let Some(ref mut taa) = self.taa {
            taa.resize(device, width, height);
        }
        if let Some(ref mut vol) = self.volumetric {
            vol.resize(device, width, height);
        }
    }

    /// Advance frame state (TAA jitter, etc.).
    pub fn advance_frame(&mut self) {
        if let Some(ref mut taa) = self.taa {
            taa.advance_frame();
        }
    }

    /// Get the current TAA jitter offset, if TAA is enabled.
    pub fn jitter_offset(&self) -> (f32, f32) {
        self.taa.as_ref().map_or((0.0, 0.0), |t| t.jitter_offset())
    }
}
