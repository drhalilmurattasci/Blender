/// Wrapper around a `wgpu::RenderPipeline` with metadata.
pub struct RenderPipelineWrapper {
    pub pipeline: wgpu::RenderPipeline,
    pub label: String,
}

impl RenderPipelineWrapper {
    /// Create from an existing pipeline.
    pub fn new(pipeline: wgpu::RenderPipeline, label: impl Into<String>) -> Self {
        Self {
            pipeline,
            label: label.into(),
        }
    }

    /// Bind this pipeline to a render pass.
    pub fn bind<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>) {
        pass.set_pipeline(&self.pipeline);
    }
}
