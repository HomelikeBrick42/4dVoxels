mod camera_buffer;
mod objects_buffer;
mod output_texture;

pub use camera_buffer::*;
use chunks::Chunks;
use objects_buffer::*;
pub use output_texture::*;

pub struct Renderer {
    device: wgpu::Device,
    queue: wgpu::Queue,

    output_texture_bind_group_layout: wgpu::BindGroupLayout,
    camera_bind_group_layout: wgpu::BindGroupLayout,

    objects_bind_group_layout: wgpu::BindGroupLayout,
    objects_buffer: ObjectsBuffer,

    ray_tracing_compute_pipeline: wgpu::ComputePipeline,
}

impl Renderer {
    pub fn new(device: wgpu::Device, queue: wgpu::Queue) -> Self {
        let ray_tracing_shader = device.create_shader_module(wgpu::include_wgsl!(concat!(
            env!("OUT_DIR"),
            "/shaders/ray_tracing.wgsl",
        )));

        let output_texture_bind_group_layout = OutputTexture::bind_group_layout(&device);
        let camera_bind_group_layout = CameraBuffer::bind_group_layout(&device);

        let objects_bind_group_layout = ObjectsBuffer::bind_group_layout(&device);
        let objects_buffer = ObjectsBuffer::new(&device, &queue, &objects_bind_group_layout);

        let ray_tracing_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Ray Tracing Pipeline Layout"),
                bind_group_layouts: &[
                    &output_texture_bind_group_layout,
                    &camera_bind_group_layout,
                    &objects_bind_group_layout,
                ],
                immediate_size: 0,
            });
        let ray_tracing_compute_pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Ray Tracing Compute Pipeline"),
                layout: Some(&ray_tracing_pipeline_layout),
                module: &ray_tracing_shader,
                entry_point: Some("trace_rays"),
                compilation_options: Default::default(),
                cache: None,
            });

        Self {
            device,
            queue,

            output_texture_bind_group_layout,
            camera_bind_group_layout,

            objects_bind_group_layout,
            objects_buffer,

            ray_tracing_compute_pipeline,
        }
    }

    pub fn create_output_texture(
        &self,
        width: u32,
        height: u32,
        extra_usages: wgpu::TextureUsages,
    ) -> OutputTexture {
        OutputTexture::new(
            &self.device,
            width,
            height,
            extra_usages,
            &self.output_texture_bind_group_layout,
        )
    }

    pub fn create_camera_buffer(&self, camera: &Camera) -> CameraBuffer {
        CameraBuffer::new(
            &self.device,
            self.queue.clone(),
            camera,
            &self.camera_bind_group_layout,
        )
    }

    pub fn upload_chunks(&mut self, chunks: &Chunks) {
        self.objects_buffer.write_buffer(
            &self.device,
            &self.queue,
            &self.objects_bind_group_layout,
            chunks.block_list(),
            std::slice::from_ref(chunks.get_chunk()),
        );
    }

    pub fn render(
        &self,
        output_texture: &OutputTexture,
        camera_buffer: &CameraBuffer,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Ray Tracing Compute Pass"),
            timestamp_writes: None,
        });

        let output_size = output_texture.size();

        compute_pass.set_pipeline(&self.ray_tracing_compute_pipeline);
        compute_pass.set_bind_group(0, output_texture.bind_group(), &[]);
        compute_pass.set_bind_group(1, camera_buffer.bind_group(), &[]);
        compute_pass.set_bind_group(2, self.objects_buffer.bind_group(), &[]);
        compute_pass.dispatch_workgroups(
            output_size.width.div_ceil(16),
            output_size.height.div_ceil(16),
            1,
        );
    }
}
