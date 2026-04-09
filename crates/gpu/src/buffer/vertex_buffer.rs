use bytemuck::Pod;
use wgpu::util::DeviceExt;

/// A vertex buffer with an associated layout.
pub struct VertexBuffer {
    pub buffer: wgpu::Buffer,
    pub layout: wgpu::VertexBufferLayout<'static>,
    pub vertex_count: u32,
}

impl VertexBuffer {
    /// Create a new vertex buffer from vertex data.
    pub fn new<V: Pod>(
        device: &wgpu::Device,
        label: &str,
        vertices: &[V],
        layout: wgpu::VertexBufferLayout<'static>,
    ) -> Self {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(label),
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        Self {
            buffer,
            layout,
            vertex_count: vertices.len() as u32,
        }
    }

    /// Slice the buffer for rendering.
    pub fn slice(&self) -> wgpu::BufferSlice<'_> {
        self.buffer.slice(..)
    }
}
