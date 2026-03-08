use bytemuck::NoUninit;
use math::Vector4;
use wgpu::util::DeviceExt as _;

#[derive(Debug, Clone, Copy, NoUninit)]
#[repr(C)]
pub struct Camera {
    pub position: Vector4<f32>,
    pub forward: Vector4<f32>,
    pub up: Vector4<f32>,
    pub right: Vector4<f32>,
    pub fov: f32,
    pub samples: u32,
    pub max_bounces: u32,
    pub random_seed: u32,
}

pub struct CameraBuffer {
    queue: wgpu::Queue,
    buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

impl CameraBuffer {
    pub(crate) fn new(
        device: &wgpu::Device,
        queue: wgpu::Queue,
        camera: &Camera,
        layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let mut camera_bytes = [0; size_of::<Camera>().next_multiple_of(16)];
        camera_bytes[..size_of::<Camera>()].copy_from_slice(bytemuck::bytes_of(camera));
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Ray Tracing Camera Buffer"),
            contents: &camera_bytes,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Ray Tracing Camera Bind Group"),
            layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });

        Self {
            queue,
            buffer,
            bind_group,
        }
    }

    pub fn write(&mut self, camera: &Camera) {
        self.queue
            .write_buffer(&self.buffer, 0, bytemuck::bytes_of(camera));
    }

    pub(super) fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    pub(super) fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Ray Tracing Camera Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        })
    }
}
