use bytemuck::Pod;
use wgpu::util::DeviceExt;

/// A typed storage buffer backed by a `wgpu::Buffer`.
pub struct StorageBuffer<T: Pod> {
    pub buffer: wgpu::Buffer,
    len: usize,
    _marker: std::marker::PhantomData<T>,
}

impl<T: Pod> StorageBuffer<T> {
    /// Create a new storage buffer from a slice of data.
    pub fn new(device: &wgpu::Device, label: &str, data: &[T]) -> Self {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(label),
            contents: bytemuck::cast_slice(data),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
        });

        Self {
            buffer,
            len: data.len(),
            _marker: std::marker::PhantomData,
        }
    }

    /// Create an empty storage buffer with the given capacity (number of elements).
    pub fn empty(device: &wgpu::Device, label: &str, capacity: usize) -> Self {
        let size = (capacity * std::mem::size_of::<T>()) as u64;
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        Self {
            buffer,
            len: 0,
            _marker: std::marker::PhantomData,
        }
    }

    /// Update the buffer contents.
    pub fn write(&mut self, queue: &wgpu::Queue, data: &[T]) {
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(data));
        self.len = data.len();
    }

    /// Number of elements currently stored.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Whether the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Get a binding resource for this buffer.
    pub fn binding(&self) -> wgpu::BindingResource<'_> {
        self.buffer.as_entire_binding()
    }
}
