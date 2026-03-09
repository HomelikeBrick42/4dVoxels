pub mod camera;

use crate::camera::{Camera, RenderView};
use chunks::{Block, BlockId, BlockType, Chunk, Chunks};
use core::time::Duration;
use math::{NoE2Rotor, Vector2, Vector3, Vector4};
use rand::seq::IndexedRandom;
use renderer::{
    app::{App, run_app},
    ray_tracing::{self, CameraBuffer, OutputTexture},
    ui::{self, TextureView},
};
use std::collections::HashSet;
use winit::keyboard::KeyCode;

pub struct Game {
    ray_tracing: ray_tracing::Renderer,
    ui: ui::Renderer,

    xyz_output_texture: OutputTexture,
    xyz_texture_view: TextureView,
    xwz_output_texture: OutputTexture,
    xwz_texture_view: TextureView,
    xyw_output_texture: OutputTexture,
    xyw_texture_view: TextureView,

    camera: Camera,
    xyz_camera_buffer: CameraBuffer,
    xwz_camera_buffer: CameraBuffer,
    xyw_camera_buffer: CameraBuffer,

    chunks: Chunks,
}

impl App for Game {
    const NAME: &str = "4d Ray Tracer";
    const FEATURES: wgpu::Features = wgpu::Features::FLOAT32_FILTERABLE;
    const PRESENT_MODE: wgpu::PresentMode = wgpu::PresentMode::AutoNoVsync;
    const COLOR_OP: wgpu::Operations<wgpu::Color> = wgpu::Operations {
        load: wgpu::LoadOp::DontCare(unsafe { wgpu::LoadOpDontCare::enabled() }), // main ray tracing view should cover the whole screen
        store: wgpu::StoreOp::Store,
    };
    const DEPTH_OP: Option<wgpu::Operations<f32>> = Some(wgpu::Operations {
        load: wgpu::LoadOp::Clear(0.0),
        store: wgpu::StoreOp::Store,
    });
    const FIXED_UPDATE_INTERVAL: Duration = Duration::from_secs(1).checked_div(100).unwrap();

    fn new(device: wgpu::Device, queue: wgpu::Queue) -> Self {
        let ray_tracing = ray_tracing::Renderer::new(device.clone(), queue.clone());
        let ui = ui::Renderer::new(device.clone(), queue.clone());

        let xyz_output_texture =
            ray_tracing.create_output_texture(1, 1, wgpu::TextureUsages::TEXTURE_BINDING);
        let xyz_texture_view = ui.create_texture_view(xyz_output_texture.texture_view());
        let xwz_output_texture =
            ray_tracing.create_output_texture(1, 1, wgpu::TextureUsages::TEXTURE_BINDING);
        let xwz_texture_view = ui.create_texture_view(xwz_output_texture.texture_view());
        let xyw_output_texture =
            ray_tracing.create_output_texture(1, 1, wgpu::TextureUsages::TEXTURE_BINDING);
        let xyw_texture_view = ui.create_texture_view(xyw_output_texture.texture_view());

        let camera = Camera {
            position: Vector4 {
                x: -2.0,
                y: 1.0,
                z: 0.0,
                w: 0.0,
            },
            base_rotation: NoE2Rotor::identity(),
            xy_rotation: 0.0,
        };
        let xyz_camera_buffer =
            ray_tracing.create_camera_buffer(&camera.into_render(RenderView::XYZ));
        let xwz_camera_buffer =
            ray_tracing.create_camera_buffer(&camera.into_render(RenderView::XWZ));
        let xyw_camera_buffer =
            ray_tracing.create_camera_buffer(&camera.into_render(RenderView::XYW));

        let mut chunks = Chunks::new();
        let grass = chunks.push_block(Block {
            color: Vector3 {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },
            typ: BlockType::Solid,
        });

        *chunks.get_chunk_mut() = Chunk::new(|_| {
            [BlockId::AIR, BlockId::AIR, BlockId::AIR, grass]
                .choose(&mut rand::rng())
                .copied()
                .unwrap()
        });

        Self {
            ray_tracing,
            ui,

            xyz_output_texture,
            xyz_texture_view,
            xwz_output_texture,
            xwz_texture_view,
            xyw_output_texture,
            xyw_texture_view,

            camera,
            xyz_camera_buffer,
            xwz_camera_buffer,
            xyw_camera_buffer,

            chunks,
        }
    }

    fn key(&mut self, key: KeyCode, pressed: bool) {
        #[expect(clippy::match_single_binding)]
        match (key, pressed) {
            _ => {}
        }
    }

    fn fixed_update(&mut self, #[expect(unused)] ts: f32) {}

    fn update(&mut self, keys: &HashSet<KeyCode>, dt: f32) {
        self.camera.update(keys, dt);
    }

    fn render<'a>(
        &'a mut self,
        width: u32,
        height: u32,
        encoder: &mut wgpu::CommandEncoder,
    ) -> impl FnOnce(&mut wgpu::RenderPass) + use<'a> {
        self.ray_tracing.upload_chunks(&self.chunks);

        {
            let xyz_output_size = self.xyz_output_texture.size();
            if xyz_output_size.width != width || xyz_output_size.height != height {
                self.xyz_output_texture = self.ray_tracing.create_output_texture(
                    width,
                    height,
                    wgpu::TextureUsages::TEXTURE_BINDING,
                );
                self.xyz_texture_view = self
                    .ui
                    .create_texture_view(self.xyz_output_texture.texture_view());
            }
        }
        {
            let xwz_output_size = self.xwz_output_texture.size();
            if xwz_output_size.width != width / 4 || xwz_output_size.height != height / 4 {
                self.xwz_output_texture = self.ray_tracing.create_output_texture(
                    width / 4,
                    height / 4,
                    wgpu::TextureUsages::TEXTURE_BINDING,
                );
                self.xwz_texture_view = self
                    .ui
                    .create_texture_view(self.xwz_output_texture.texture_view());
            }
        }
        {
            let xyw_output_size = self.xyw_output_texture.size();
            if xyw_output_size.width != width / 4 || xyw_output_size.height != height / 4 {
                self.xyw_output_texture = self.ray_tracing.create_output_texture(
                    width / 4,
                    height / 4,
                    wgpu::TextureUsages::TEXTURE_BINDING,
                );
                self.xyw_texture_view = self
                    .ui
                    .create_texture_view(self.xyw_output_texture.texture_view());
            }
        }

        self.ui.set_screen_size(width, height);

        self.xyz_camera_buffer
            .write(&self.camera.into_render(RenderView::XYZ));
        self.ray_tracing
            .render(&self.xyz_output_texture, &self.xyz_camera_buffer, encoder);
        self.ui.push_quad(
            Vector2 { x: 0.0, y: 0.0 },
            Vector2 {
                x: self.ui.aspect() * 2.0,
                y: 2.0,
            },
            &self.xyz_texture_view,
        );

        self.xwz_camera_buffer
            .write(&self.camera.into_render(RenderView::XWZ));
        self.ray_tracing
            .render(&self.xwz_output_texture, &self.xwz_camera_buffer, encoder);
        self.ui.push_quad(
            Vector2 {
                x: self.ui.aspect() - 0.25,
                y: -0.75,
            },
            Vector2 { x: 0.5, y: 0.5 },
            &self.xwz_texture_view,
        );

        self.xyw_camera_buffer
            .write(&self.camera.into_render(RenderView::XYW));
        self.ray_tracing
            .render(&self.xyw_output_texture, &self.xyw_camera_buffer, encoder);
        self.ui.push_quad(
            Vector2 {
                x: self.ui.aspect() - 0.25,
                y: 0.75,
            },
            Vector2 { x: 0.5, y: 0.5 },
            &self.xyw_texture_view,
        );

        move |render_pass| {
            self.ui.draw(render_pass);
        }
    }
}

fn main() {
    run_app::<Game>()
}
