use bytemuck::NoUninit;
use chunks::{Block, BlockId, BlockType, Chunk};
use math::Vector3;
use wgpu::util::DeviceExt as _;

#[derive(Debug, Clone, Copy, NoUninit)]
#[repr(C)]
struct ObjectsInfo {
    chunks_count: u32,
}

pub struct ObjectsBuffer {
    objects_info_buffer: wgpu::Buffer,
    block_list_buffer: wgpu::Buffer,
    chunk_list_buffer: wgpu::Buffer,
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
            .copy_from_slice(bytemuck::bytes_of(&ObjectsInfo { chunks_count: 1 }));
        let objects_info_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Ray Tracing Objects Info Buffer"),
            contents: &objects_info_bytes,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let block_list_buffer = Self::make_block_list(
            device,
            &[Block {
                color: Vector3 {
                    x: 1.0,
                    y: 1.0,
                    z: 1.0,
                },
                typ: BlockType::Transparent,
            }],
        );

        let chunk_list_buffer = Self::make_chunk_list(device, &[Chunk::new(|_| BlockId::AIR)]);

        let bind_group = Self::make_bind_group(
            device,
            bind_group_layout,
            &objects_info_buffer,
            &block_list_buffer,
            &chunk_list_buffer,
        );

        Self {
            objects_info_buffer,
            block_list_buffer,
            chunk_list_buffer,
            bind_group,
        }
    }

    pub(super) fn write_buffer(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bind_group_layout: &wgpu::BindGroupLayout,
        block_list: &[Block],
        chunk_list: &[Chunk],
    ) {
        let mut buffers_reallocated = false;

        if size_of_val::<[_]>(block_list) > self.block_list_buffer.size() as _ {
            self.block_list_buffer = Self::make_block_list(device, block_list);
            buffers_reallocated = true;
        } else {
            queue.write_buffer(&self.block_list_buffer, 0, bytemuck::cast_slice(block_list));
        }

        if size_of_val::<[_]>(chunk_list) > self.block_list_buffer.size() as _ {
            self.chunk_list_buffer = Self::make_chunk_list(device, chunk_list);
            buffers_reallocated = true;
        } else {
            queue.write_buffer(&self.chunk_list_buffer, 0, bytemuck::cast_slice(chunk_list));
        }

        if buffers_reallocated {
            self.bind_group = Self::make_bind_group(
                device,
                bind_group_layout,
                &self.objects_info_buffer,
                &self.block_list_buffer,
                &self.chunk_list_buffer,
            );
        }
    }

    pub(super) fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    pub(super) fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Ray Tracing Objects Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        })
    }

    fn make_block_list(device: &wgpu::Device, block_list: &[Block]) -> wgpu::Buffer {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Ray Tracing Block List Buffer"),
            contents: bytemuck::cast_slice(block_list),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        })
    }

    fn make_chunk_list(device: &wgpu::Device, chunk_list: &[Chunk]) -> wgpu::Buffer {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Ray Tracing Chunk List Buffer"),
            contents: bytemuck::cast_slice(chunk_list),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        })
    }

    fn make_bind_group(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        objects_info_buffer: &wgpu::Buffer,
        block_list_buffer: &wgpu::Buffer,
        chunk_list_buffer: &wgpu::Buffer,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Ray Tracing Objects Bind Group"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: objects_info_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: block_list_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: chunk_list_buffer.as_entire_binding(),
                },
            ],
        })
    }
}
