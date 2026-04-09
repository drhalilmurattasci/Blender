/// Wrapper around a `wgpu::ComputePipeline` with metadata.
pub struct ComputePipelineWrapper {
    pub pipeline: wgpu::ComputePipeline,
    pub label: String,
}

impl ComputePipelineWrapper {
    /// Create from an existing pipeline.
    pub fn new(pipeline: wgpu::ComputePipeline, label: impl Into<String>) -> Self {
        Self {
            pipeline,
            label: label.into(),
        }
    }

    /// Bind this pipeline to a compute pass.
    pub fn bind<'a>(&'a self, pass: &mut wgpu::ComputePass<'a>) {
        pass.set_pipeline(&self.pipeline);
    }

    /// Dispatch workgroups.
    pub fn dispatch<'a>(
        &'a self,
        pass: &mut wgpu::ComputePass<'a>,
        x: u32,
        y: u32,
        z: u32,
    ) {
        pass.set_pipeline(&self.pipeline);
        pass.dispatch_workgroups(x, y, z);
    }
}
