use super::module::GpuShader;

/// Descriptor for building a render pipeline.
pub struct RenderPipelineDesc<'a> {
    pub label: &'a str,
    pub shader: &'a GpuShader,
    pub vertex_entry: &'a str,
    pub fragment_entry: &'a str,
    pub vertex_layouts: &'a [wgpu::VertexBufferLayout<'a>],
    pub color_formats: &'a [Option<wgpu::TextureFormat>],
    pub depth_format: Option<wgpu::TextureFormat>,
    pub cull_mode: Option<wgpu::Face>,
    pub bind_group_layouts: &'a [&'a wgpu::BindGroupLayout],
    pub blend_state: Option<wgpu::BlendState>,
    pub topology: wgpu::PrimitiveTopology,
}

impl<'a> RenderPipelineDesc<'a> {
    /// Build the wgpu render pipeline from this descriptor.
    pub fn build(&self, device: &wgpu::Device) -> wgpu::RenderPipeline {
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(&format!("{}_layout", self.label)),
            bind_group_layouts: self.bind_group_layouts,
            push_constant_ranges: &[],
        });

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(self.label),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: self.shader.module(),
                entry_point: Some(self.vertex_entry),
                buffers: self.vertex_layouts,
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: self.shader.module(),
                entry_point: Some(self.fragment_entry),
                targets: self
                    .color_formats
                    .iter()
                    .map(|fmt| {
                        fmt.map(|format| wgpu::ColorTargetState {
                            format,
                            blend: self.blend_state,
                            write_mask: wgpu::ColorWrites::ALL,
                        })
                    })
                    .collect::<Vec<_>>()
                    .as_slice(),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: self.topology,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: self.cull_mode,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: self.depth_format.map(|format| wgpu::DepthStencilState {
                format,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        })
    }
}

/// Descriptor for building a compute pipeline.
pub struct ComputePipelineDesc<'a> {
    pub label: &'a str,
    pub shader: &'a GpuShader,
    pub entry_point: &'a str,
    pub bind_group_layouts: &'a [&'a wgpu::BindGroupLayout],
}

impl<'a> ComputePipelineDesc<'a> {
    /// Build the wgpu compute pipeline from this descriptor.
    pub fn build(&self, device: &wgpu::Device) -> wgpu::ComputePipeline {
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(&format!("{}_layout", self.label)),
            bind_group_layouts: self.bind_group_layouts,
            push_constant_ranges: &[],
        });

        device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some(self.label),
            layout: Some(&pipeline_layout),
            module: self.shader.module(),
            entry_point: Some(self.entry_point),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        })
    }
}
