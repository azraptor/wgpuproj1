use pollster::FutureExt;
use std::{iter, sync::Arc};
use wgpu::util::DeviceExt;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::*,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::PhysicalKey,
    window::{Window, WindowAttributes, WindowId},
};

// Modules
mod camera;
mod texture;
mod vert;

use crate::camera::{Camera, CameraController, CameraUniform};
use crate::texture::Texture;
use crate::vert::{INDICES, VERTS, Vert};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

// Shader code
// TODO: Make it so that we can load this from a file instead
// of just including it
const WGSL_CODE: wgpu::ShaderModuleDescriptor = wgpu::include_wgsl!("shaders/simple_texture.wgsl");

struct App {
    state: Option<State>,
}

impl App {
    pub fn new() -> Self {
        Self { state: None }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Base window size
        let size = PhysicalSize::new(512, 512);
        // Base window attributes
        let attrib = WindowAttributes::default()
            .with_inner_size(size)
            .with_title("WGPU Program");

        // Create the window
        let window = event_loop.create_window(attrib).unwrap();

        log::warn!("{:?}", size);
        log::warn!("{:?}", window.inner_size());

        if window.inner_size().width == 0 {
            log::warn!("I FUCKING LOVE WASM!");
            let _ = window.request_inner_size(size);
        }

        // WASM canvas element implementation
        #[cfg(target_arch = "wasm32")]
        {
            // Setting size manually to work around winit
            // copied from wgpu tutorial
            use winit::platform::web::WindowExtWebSys;
            web_sys::window()
                .and_then(|win| win.document())
                .and_then(|doc| {
                    let dst = doc.get_element_by_id("game")?;
                    let canvas = web_sys::Element::from(window.canvas()?);
                    dst.append_child(&canvas).ok()?;
                    Some(())
                })
                .expect("Couldn't append canvas to document body.");
        }

        // Configuration specific to desktop
        #[cfg(not(target_arch = "wasm32"))]
        {
            use winit::window::Icon;
            let max_size = PhysicalSize::new(1024, 1024);
            let min_size = size;

            const ICON_DATA: &[u8] = include_bytes!("res/icon.png");

            let (bytes, w, h) = {
                let img = image::load_from_memory(ICON_DATA).unwrap().to_rgba8();
                let (w, h) = img.dimensions();
                let bytes = img.into_raw();
                (bytes, w, h)
            };

            let win_icon = Icon::from_rgba(bytes, w, h).unwrap();

            // Set stuff that only matters for desktops
            window.set_max_inner_size(Some(max_size));
            window.set_min_inner_size(Some(min_size));
            window.set_window_icon(Some(win_icon));
        }

        self.state = Some(State::new(window).block_on());
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        if let Some(state) = self.state.as_mut() {
            if id != state.window.id() {
                return;
            }
        }

        // Event handling
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state,
                        physical_key: PhysicalKey::Code(keycode),
                        ..
                    },
                ..
            } => {
                if let Some(prog_state) = self.state.as_mut() {
                    log::info!("{:?} {}", keycode, state.is_pressed());
                    prog_state
                        .camera_controller
                        .process_events(state.is_pressed(), keycode);
                }
            }
            WindowEvent::Resized(physical_size) => {
                if let Some(state) = self.state.as_mut() {
                    state.resize(physical_size);
                }
            }
            WindowEvent::RedrawRequested => {
                // Redraw the window and gfx
                if let Some(state) = self.state.as_mut() {
                    state.window.request_redraw();

                    state.update();

                    match state.render() {
                        Ok(_) => {}

                        Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                            state.resize(state.size)
                        }
                        Err(wgpu::SurfaceError::OutOfMemory | wgpu::SurfaceError::Other) => {
                            log::error!("Out of memory!");
                            event_loop.exit();
                        }
                        Err(wgpu::SurfaceError::Timeout) => {
                            log::warn!("Surface timeout");
                        }
                    }
                }
            }
            _ => (),
        }
    }
}

// Program state
struct State {
    // General fields needed for WGPU to work
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    window: Arc<Window>,
    render_pipeline: wgpu::RenderPipeline,
    // Buffers & Bindgroups
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    diffuse_bind_group: wgpu::BindGroup,
    // Camera
    camera: Camera,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    camera_controller: CameraController,
}

impl State {
    // Creating some of the wgpu types requires async code
    async fn new(window: Window) -> State {
        let size = window.inner_size();
        let window = Arc::new(window);

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            backends: wgpu::Backends::VULKAN,
            #[cfg(target_arch = "wasm32")]
            backends: wgpu::Backends::GL,
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone()).unwrap();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        // Render device
        let desktop_features =
            wgpu::Features::POLYGON_MODE_POINT | wgpu::Features::POLYGON_MODE_LINE;
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: if cfg!(target_arch = "wasm32") {
                        wgpu::Features::empty()
                    } else {
                        desktop_features
                    },
                    required_limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                    label: None,
                    memory_hints: Default::default(),
                },
                None,
            )
            .await
            .unwrap();

        // Surface configuration
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoVsync, //surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: Vec::new(),
            desired_maximum_frame_latency: 2,
        };

        log::warn!("{} {}", config.width, config.height);

        // Texture
        let tex1_bytes = include_bytes!("res/texture_test_1.png");
        let img: image::DynamicImage = image::load_from_memory(tex1_bytes).unwrap();
        let diffuse_texture =
            Texture::from_image(&device, &queue, &img, Some("diffuse_texture")).unwrap();

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        let diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                },
            ],
            label: Some("diffuse_bind_group"),
        });

        // Camera
        let camera = Camera::new(config.width as f32 / config.height as f32);
        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera);

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("camera_bind_group_layout"),
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        let camera_controller = CameraController::new(0.02);

        // Shader and render pipeline
        let shader = device.create_shader_module(WGSL_CODE);

        // Buffers
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTS),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&texture_bind_group_layout, &camera_bind_group_layout],
                push_constant_ranges: &[],
            });

        let vert_shader_state = wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[Vert::desc()],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        };

        let frag_shader_state = wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: config.format,
                // Set alpha mode so translucency works
                blend: Some(wgpu::BlendState {
                    color: wgpu::BlendComponent {
                        src_factor: wgpu::BlendFactor::SrcAlpha,
                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        operation: wgpu::BlendOperation::Add,
                    },
                    alpha: wgpu::BlendComponent::OVER,
                }),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        };

        let primitive_state = wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        };

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: vert_shader_state,
            fragment: Some(frag_shader_state),
            primitive: primitive_state,
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        // Now create our state struct
        Self {
            surface,
            device,
            queue,
            config,
            size,
            window,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            diffuse_bind_group,
            camera,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            camera_controller,
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn update(&mut self) {
        self.camera_controller.update_camera(&mut self.camera);
        self.camera_uniform.update_view_proj(&self.camera);
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        let clear_color = wgpu::Color {
            r: 0.2,
            g: 0.2,
            b: 0.2,
            a: 1.0,
        };
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(clear_color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // Draw to pipeline
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
            render_pass.set_bind_group(1, &self.camera_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..(INDICES.len() as u32), 0, 0..1);
        }

        // Submit our queue and then render it
        self.queue.submit(iter::once(encoder.finish()));
        output.present();
        Ok(())
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {
    // Create env logger
    #[cfg(target_arch = "wasm32")]
    {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        env_logger::init();
    }

    // Create event loop
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    // Running our app window + wgpu context
    #[allow(unused_mut)]
    let mut app = App::new();

    #[cfg(not(target_arch = "wasm32"))]
    event_loop.run_app(&mut app).unwrap();

    #[cfg(target_arch = "wasm32")]
    {
        use winit::platform::web::EventLoopExtWebSys;
        event_loop.spawn_app(app);
    }
}
