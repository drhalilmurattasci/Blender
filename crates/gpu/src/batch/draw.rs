/// A single draw command.
#[derive(Debug, Clone)]
pub struct DrawCommand {
    pub vertex_range: std::ops::Range<u32>,
    pub instance_range: std::ops::Range<u32>,
    pub index_buffer: Option<wgpu::Buffer>,
    pub index_count: u32,
}

impl DrawCommand {
    /// Create a non-indexed draw command.
    pub fn vertices(vertex_count: u32) -> Self {
        Self {
            vertex_range: 0..vertex_count,
            instance_range: 0..1,
            index_buffer: None,
            index_count: 0,
        }
    }

    /// Create an indexed draw command.
    pub fn indexed(index_count: u32, index_buffer: wgpu::Buffer) -> Self {
        Self {
            vertex_range: 0..0,
            instance_range: 0..1,
            index_buffer: Some(index_buffer),
            index_count,
        }
    }

    /// Set the number of instances.
    pub fn with_instances(mut self, count: u32) -> Self {
        self.instance_range = 0..count;
        self
    }
}

/// A batch of draw commands sharing the same pipeline and bind groups.
pub struct GpuBatch {
    pub commands: Vec<DrawCommand>,
    pub vertex_buffers: Vec<wgpu::Buffer>,
}

impl GpuBatch {
    /// Create a new empty batch.
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
            vertex_buffers: Vec::new(),
        }
    }

    /// Add a vertex buffer to the batch.
    pub fn add_vertex_buffer(&mut self, buffer: wgpu::Buffer) -> usize {
        let idx = self.vertex_buffers.len();
        self.vertex_buffers.push(buffer);
        idx
    }

    /// Add a draw command.
    pub fn add_command(&mut self, command: DrawCommand) {
        self.commands.push(command);
    }

    /// Execute the batch on a render pass.
    pub fn execute<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>) {
        for (i, vb) in self.vertex_buffers.iter().enumerate() {
            pass.set_vertex_buffer(i as u32, vb.slice(..));
        }

        for cmd in &self.commands {
            if let Some(ref ib) = cmd.index_buffer {
                pass.set_index_buffer(ib.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..cmd.index_count, 0, cmd.instance_range.clone());
            } else {
                pass.draw(cmd.vertex_range.clone(), cmd.instance_range.clone());
            }
        }
    }
}

impl Default for GpuBatch {
    fn default() -> Self {
        Self::new()
    }
}
