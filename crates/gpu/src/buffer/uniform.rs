use bytemuck::Pod;
use wgpu::util::DeviceExt;

/// A typed uniform buffer backed by a `wgpu::Buffer`.
pub struct UniformBuffer<T: Pod> {
    pub buffer: wgpu::Buffer,
    _marker: std::marker::PhantomData<T>,
}

impl<T: Pod> UniformBuffer<T> {
    /// Create a new uniform buffer initialized with the given data.
    pub fn new(device: &wgpu::Device, label: &str, data: &T) -> Self {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(label),
            contents: bytemuck::bytes_of(data),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        Self {
            buffer,
            _marker: std::marker::PhantomData,
        }
    }

    /// Update the buffer contents.
    pub fn write(&self, queue: &wgpu::Queue, data: &T) {
        queue.write_buffer(&self.buffer, 0, bytemuck::bytes_of(data));
    }

    /// Get a binding resource for this buffer.
    pub fn binding(&self) -> wgpu::BindingResource<'_> {
        self.buffer.as_entire_binding()
    }
}
