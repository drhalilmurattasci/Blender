/// Wraps a compiled `wgpu::ShaderModule`.
pub struct GpuShader {
    pub module: wgpu::ShaderModule,
    pub label: String,
}

impl GpuShader {
    /// Create a shader module from WGSL source code.
    pub fn from_wgsl(device: &wgpu::Device, label: &str, source: &str) -> Self {
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(label),
            source: wgpu::ShaderSource::Wgsl(source.into()),
        });

        Self {
            module,
            label: label.to_string(),
        }
    }

    /// Get a reference to the underlying shader module.
    pub fn module(&self) -> &wgpu::ShaderModule {
        &self.module
    }
}
