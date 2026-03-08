mod texture;

use bytemuck::NoUninit;
use math::Vector2;
pub use texture::*;

#[derive(Debug, Clone, Copy, NoUninit)]
#[repr(C)]
struct Quad {
    position: Vector2<f32>,
    size: Vector2<f32>,
}

enum Layer {
    Quads {
        quads_start: u32,
        quads_end: u32,
        texture_bind_group: wgpu::BindGroup,
    },
}

pub struct Renderer {
    device: wgpu::Device,
    queue: wgpu::Queue,

    quads_buffer: wgpu::Buffer,
    quads_bind_group_layout: wgpu::BindGroupLayout,
    quads_bind_group: wgpu::BindGroup,

    texture_view_bind_group_layout: wgpu::BindGroupLayout,

    aspect: f32,
    quads: Vec<Quad>,
    layers: Vec<Layer>,

    quads_render_pipeline: wgpu::RenderPipeline,
}

impl Renderer {
    pub fn new(device: wgpu::Device, queue: wgpu::Queue) -> Self {
        let quads_buffer = Self::quads_buffer(&device, 0);
        let quads_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Quads Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });
        let quads_bind_group =
            Self::quads_bind_group(&device, &quads_bind_group_layout, &quads_buffer);

        let texture_view_bind_group_layout = TextureView::bind_group_layout(&device);

        let quads_shader = device.create_shader_module(wgpu::include_wgsl!(concat!(
            env!("OUT_DIR"),
            "/shaders/quads.wgsl"
        )));
        let quads_render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Quads Pipeline Layout"),
                bind_group_layouts: &[&quads_bind_group_layout, &texture_view_bind_group_layout],
                immediate_size: 0,
            });
        let quads_render_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Quads Render Pipeline"),
                layout: Some(&quads_render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &quads_shader,
                    entry_point: Some("vertex"),
                    compilation_options: Default::default(),
                    buffers: &[],
                },
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleStrip,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Cw,
                    cull_mode: None,
                    unclipped_depth: false,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    conservative: false,
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float,
                    depth_write_enabled: false,
                    depth_compare: wgpu::CompareFunction::Always,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                fragment: Some(wgpu::FragmentState {
                    module: &quads_shader,
                    entry_point: Some("fragment"),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::Bgra8Unorm,
                        blend: None,
                        write_mask: wgpu::ColorWrites::all(),
                    })],
                }),
                multiview_mask: None,
                cache: None,
            });

        Self {
            device,
            queue,

            quads_buffer,
            quads_bind_group_layout,
            quads_bind_group,

            texture_view_bind_group_layout,

            aspect: 1.0,
            quads: vec![],
            layers: vec![],

            quads_render_pipeline,
        }
    }

    pub fn create_texture_view(&self, texture_view: &wgpu::TextureView) -> TextureView {
        TextureView::new(
            &self.device,
            texture_view,
            &self.texture_view_bind_group_layout,
        )
    }

    pub fn set_screen_size(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }

    pub fn aspect(&self) -> f32 {
        self.aspect
    }

    pub fn push_quad(
        &mut self,
        position: Vector2<f32>,
        size: Vector2<f32>,
        texture_view: &TextureView,
    ) {
        let index = self.quads.len();
        self.quads.push(Quad {
            position: Vector2 {
                x: position.x / self.aspect,
                y: position.y,
            },
            size: Vector2 {
                x: size.x / self.aspect,
                y: size.y,
            },
        });

        if let Some(Layer::Quads {
            quads_start: _,
            quads_end,
            texture_bind_group,
        }) = self.layers.last_mut()
            && *texture_bind_group == *texture_view.bind_group()
        {
            *quads_end = self.quads.len() as _;
        } else {
            self.layers.push(Layer::Quads {
                quads_start: index as _,
                quads_end: self.quads.len() as _,
                texture_bind_group: texture_view.bind_group().clone(),
            });
        }
    }

    pub fn draw(&mut self, render_pass: &mut wgpu::RenderPass) {
        if size_of_val::<[_]>(&self.quads) > self.quads_buffer.size() as _ {
            self.quads_buffer = Self::quads_buffer(&self.device, self.quads.len());
            self.quads_bind_group = Self::quads_bind_group(
                &self.device,
                &self.quads_bind_group_layout,
                &self.quads_buffer,
            );
        }
        self.queue
            .write_buffer(&self.quads_buffer, 0, bytemuck::cast_slice(&self.quads));
        self.quads.clear();

        for layer in self.layers.drain(..) {
            match layer {
                Layer::Quads {
                    quads_start,
                    quads_end,
                    texture_bind_group,
                } => {
                    render_pass.set_pipeline(&self.quads_render_pipeline);
                    render_pass.set_bind_group(0, &self.quads_bind_group, &[]);
                    render_pass.set_bind_group(1, &texture_bind_group, &[]);
                    render_pass.draw(0..4, quads_start..quads_end);
                }
            }
        }
    }

    fn quads_buffer(device: &wgpu::Device, length: usize) -> wgpu::Buffer {
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Quads Buffer"),
            size: (length.max(1) * size_of::<Quad>()) as _,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    fn quads_bind_group(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        quads_buffer: &wgpu::Buffer,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Quads Bind Group"),
            layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: quads_buffer.as_entire_binding(),
            }],
        })
    }
}
