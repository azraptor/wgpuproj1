use pollster::FutureExt;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::*,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::PhysicalKey,
    window::{WindowAttributes, WindowId},
};

// Modules
mod camera;
mod state;
mod texture;
mod vert;

use crate::state::State;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

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

fn main() {
    pollster::block_on(run());
}
