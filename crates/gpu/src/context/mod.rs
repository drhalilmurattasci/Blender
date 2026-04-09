mod device;
mod surface;

pub use device::GpuDevice;
pub use surface::GpuSurface;

use crate::GpuResult;

/// Top-level GPU context combining a device and an optional surface.
pub struct GpuContext<'window> {
    pub instance: wgpu::Instance,
    pub device: GpuDevice,
    pub surface: Option<GpuSurface<'window>>,
}

impl<'window> GpuContext<'window> {
    /// Initialize a GPU context without a surface (headless / compute-only).
    pub async fn new_headless() -> GpuResult<Self> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let device = GpuDevice::new(&instance, None).await?;

        Ok(Self {
            instance,
            device,
            surface: None,
        })
    }

    /// Initialize a GPU context with a surface for presentation.
    pub async fn new_with_surface(
        surface: wgpu::Surface<'window>,
        width: u32,
        height: u32,
    ) -> GpuResult<Self> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let device = GpuDevice::new(&instance, Some(&surface)).await?;
        let gpu_surface = GpuSurface::new(
            surface,
            &device.adapter,
            &device.device,
            width,
            height,
        )?;

        Ok(Self {
            instance,
            device,
            surface: Some(gpu_surface),
        })
    }

    /// Resize the surface if present.
    pub fn resize(&mut self, width: u32, height: u32) {
        if let Some(ref mut surface) = self.surface {
            surface.resize(&self.device.device, width, height);
        }
    }
}
