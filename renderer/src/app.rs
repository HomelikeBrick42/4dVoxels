use std::{
    collections::HashSet,
    sync::Arc,
    time::{Duration, Instant},
};
use winit::keyboard::KeyCode;
use winit::{
    application::ApplicationHandler,
    event::{KeyEvent, StartCause, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::PhysicalKey,
    window::{Window, WindowAttributes, WindowId},
};

pub trait App {
    const NAME: &str;
    const FEATURES: wgpu::Features;
    const PRESENT_MODE: wgpu::PresentMode;
    const COLOR_OP: wgpu::Operations<wgpu::Color>;
    const DEPTH_OP: Option<wgpu::Operations<f32>>;
    const FIXED_UPDATE_INTERVAL: Duration;

    fn new(device: wgpu::Device, queue: wgpu::Queue) -> Self;

    fn key(&mut self, #[expect(unused)] key: KeyCode, #[expect(unused)] pressed: bool) {}

    fn fixed_update(&mut self, #[expect(unused)] ts: f32) {}

    fn update(&mut self, #[expect(unused)] keys: &HashSet<KeyCode>, #[expect(unused)] dt: f32) {}

    fn render<'a>(
        &'a mut self,
        #[expect(unused)] width: u32,
        #[expect(unused)] height: u32,
        #[expect(unused)] encoder: &mut wgpu::CommandEncoder,
    ) -> impl FnOnce(&mut wgpu::RenderPass) + use<'a, Self>;
}

struct AppState<A: App> {
    instance: wgpu::Instance,
    device: wgpu::Device,
    queue: wgpu::Queue,
    app: A,
    keys: HashSet<KeyCode>,
    last_time: Option<Instant>,
    dt: Duration,
    fixed_time: Duration,
    window_state: Option<WindowState>,
}

struct WindowState {
    window: Arc<Window>,
    surface_config: wgpu::SurfaceConfiguration,
    surface: wgpu::Surface<'static>,
    depth_texture_view: wgpu::TextureView,
}

impl<A: App> ApplicationHandler for AppState<A> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.suspended(event_loop);

        let window = Arc::new(
            event_loop
                .create_window(WindowAttributes::default().with_title(A::NAME))
                .unwrap(),
        );
        self.window_state = Some(WindowState {
            window: window.clone(),
            surface_config: wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: wgpu::TextureFormat::Bgra8Unorm,
                width: 1,
                height: 1,
                present_mode: A::PRESENT_MODE,
                desired_maximum_frame_latency: 2,
                alpha_mode: wgpu::CompositeAlphaMode::Opaque,
                view_formats: vec![],
            },
            surface: self.instance.create_surface(window).unwrap(),
            depth_texture_view: Self::depth_texture(&self.device, 1, 1),
        });
        self.resize();
    }

    fn suspended(&mut self, #[expect(unused)] event_loop: &ActiveEventLoop) {
        self.last_time = None;
        self.dt = Duration::ZERO;
        self.fixed_time = Duration::ZERO;
        self.window_state = None;
    }

    fn new_events(
        &mut self,
        #[expect(unused)] event_loop: &ActiveEventLoop,
        #[expect(unused)] cause: StartCause,
    ) {
        let time = Instant::now();
        self.dt = time - self.last_time.unwrap_or(time);
        self.last_time = Some(time);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(WindowState { window, .. }) = &self.window_state else {
            return;
        };
        if window.id() != window_id {
            return;
        }

        match event {
            WindowEvent::CloseRequested | WindowEvent::Destroyed => event_loop.exit(),

            WindowEvent::Resized(_) => {
                self.resize();
                self.render();
            }

            WindowEvent::KeyboardInput {
                device_id: _,
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(key),
                        state,
                        ..
                    },
                is_synthetic: _,
            } => {
                self.app.key(key, state.is_pressed());
                if state.is_pressed() {
                    self.keys.insert(key);
                } else {
                    self.keys.remove(&key);
                }
            }

            _ => {}
        }
    }

    fn about_to_wait(&mut self, #[expect(unused)] event_loop: &ActiveEventLoop) {
        'fixed_update: {
            self.fixed_time += self.dt;
            for _ in 0..(1.0 / A::FIXED_UPDATE_INTERVAL.as_secs_f32()) as _ {
                if self.fixed_time < A::FIXED_UPDATE_INTERVAL {
                    break 'fixed_update;
                }

                self.fixed_time -= A::FIXED_UPDATE_INTERVAL;
                self.app
                    .fixed_update(A::FIXED_UPDATE_INTERVAL.as_secs_f32());
            }
            println!(
                "Lagging too far behind, skipping {:.1} fixed updates",
                self.fixed_time.as_secs_f32() / A::FIXED_UPDATE_INTERVAL.as_secs_f32()
            );
            self.fixed_time = Duration::ZERO;
        }

        self.app.update(&self.keys, self.dt.as_secs_f32());
        self.render();
    }
}

impl<A: App> AppState<A> {
    fn depth_texture(device: &wgpu::Device, width: u32, height: u32) -> wgpu::TextureView {
        device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("Depth Texture"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth32Float,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            })
            .create_view(&Default::default())
    }

    fn resize(&mut self) {
        let Some(WindowState {
            window,
            surface_config,
            surface,
            depth_texture_view,
        }) = &mut self.window_state
        else {
            return;
        };

        let size = window.inner_size();
        surface_config.width = size.width;
        surface_config.height = size.height;
        if surface_config.width > 0 && surface_config.height > 0 {
            surface.configure(&self.device, surface_config);
            *depth_texture_view =
                Self::depth_texture(&self.device, surface_config.width, surface_config.height);
        }
    }

    fn render(&mut self) {
        let Some(WindowState {
            surface,
            depth_texture_view,
            ..
        }) = &self.window_state
        else {
            return;
        };

        let surface_texture = match surface.get_current_texture() {
            Err(wgpu::SurfaceError::Timeout) => return,
            Err(wgpu::SurfaceError::Outdated | wgpu::SurfaceError::Lost) => {
                self.resize();
                return;
            }
            e => e.unwrap(),
        };

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        {
            let app_rendering = self.app.render(
                surface_texture.texture.width(),
                surface_texture.texture.height(),
                &mut encoder,
            );

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Main Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &surface_texture.texture.create_view(&Default::default()),
                    depth_slice: None,
                    resolve_target: None,
                    ops: A::COLOR_OP,
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: depth_texture_view,
                    depth_ops: A::DEPTH_OP,
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            app_rendering(&mut render_pass);
        }
        self.queue.submit(core::iter::once(encoder.finish()));

        let suboptimal = surface_texture.suboptimal;
        surface_texture.present();
        if suboptimal {
            self.resize();
        }
    }
}

pub fn run_app<A: App>() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all().with_env(),
        flags: wgpu::InstanceFlags::from_build_config().with_env(),
        memory_budget_thresholds: wgpu::MemoryBudgetThresholds {
            for_resource_creation: None,
            for_device_loss: None,
        },
        backend_options: wgpu::BackendOptions::default().with_env(),
    });

    let (device, queue) = pollster::block_on(async {
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptionsBase {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: None,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("Wgpu Device"),
                required_features: A::FEATURES,
                required_limits: adapter.limits(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                memory_hints: wgpu::MemoryHints::Performance,
                trace: wgpu::Trace::Off,
            })
            .await
            .unwrap();

        (device, queue)
    });

    let app = A::new(device.clone(), queue.clone());
    let mut app_state = AppState {
        instance,
        device,
        queue,
        app,
        keys: HashSet::new(),
        last_time: None,
        dt: Duration::ZERO,
        fixed_time: Duration::ZERO,
        window_state: None,
    };
    event_loop.run_app(&mut app_state).unwrap();
}
