use bytemuck::NoUninit;
use wgpu::util::DeviceExt as _;

#[derive(Debug, Clone, Copy, NoUninit)]
#[repr(C)]
struct ObjectsInfo {
    foo: u32,
}

pub struct ObjectsBuffer {
    #[expect(unused)]
    objects_info_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

impl ObjectsBuffer {
    pub(crate) fn new(
        device: &wgpu::Device,
        #[expect(unused)] queue: &wgpu::Queue,
        bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let mut objects_info_bytes = [0; size_of::<ObjectsInfo>().next_multiple_of(16)];
        objects_info_bytes[..size_of::<ObjectsInfo>()]
            .copy_from_slice(bytemuck::bytes_of(&ObjectsInfo { foo: 0 }));
        let objects_info_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Ray Tracing Objects Info Buffer"),
            contents: &objects_info_bytes,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group = Self::make_bind_group(device, bind_group_layout, &objects_info_buffer);

        Self {
            objects_info_buffer,
            bind_group,
        }
    }

    pub(super) fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    pub(super) fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Ray Tracing Objects Bind Group Layout"),
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

    fn make_bind_group(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        objects_info_buffer: &wgpu::Buffer,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Ray Tracing Objects Bind Group"),
            layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: objects_info_buffer.as_entire_binding(),
            }],
        })
    }
}
