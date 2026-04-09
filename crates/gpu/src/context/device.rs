use std::sync::Arc;

use tracing::info;

use crate::{GpuError, GpuResult};

/// Wraps a `wgpu::Device` and its associated `wgpu::Queue`.
pub struct GpuDevice {
    pub device: Arc<wgpu::Device>,
    pub queue: Arc<wgpu::Queue>,
    pub adapter: wgpu::Adapter,
}

impl GpuDevice {
    /// Create a new `GpuDevice` by requesting an adapter and device.
    pub async fn new(instance: &wgpu::Instance, compatible_surface: Option<&wgpu::Surface<'_>>) -> GpuResult<Self> {
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface,
            })
            .await
            .ok_or(GpuError::AdapterNotFound)?;

        let adapter_info = adapter.get_info();
        info!(
            "Selected GPU: {} ({:?})",
            adapter_info.name, adapter_info.backend
        );

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("forge3d-device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::Performance,
                },
                None,
            )
            .await?;

        Ok(Self {
            device: Arc::new(device),
            queue: Arc::new(queue),
            adapter,
        })
    }

    /// Write data to the given buffer.
    pub fn write_buffer(&self, buffer: &wgpu::Buffer, offset: u64, data: &[u8]) {
        self.queue.write_buffer(buffer, offset, data);
    }

    /// Submit command buffers to the queue.
    pub fn submit(&self, command_buffers: impl IntoIterator<Item = wgpu::CommandBuffer>) {
        self.queue.submit(command_buffers);
    }

    /// Create a command encoder.
    pub fn create_command_encoder(&self, label: &str) -> wgpu::CommandEncoder {
        self.device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some(label),
            })
    }
}
