use pollster::FutureExt;
#[cfg(not(target_arch = "wasm32"))]
use winit::dpi::PhysicalSize;
use winit::{
    application::ApplicationHandler,
    event::*,
    event_loop::ActiveEventLoop,
    keyboard::PhysicalKey,
    window::{WindowAttributes, WindowId},
};

use crate::state::State;

#[cfg(not(target_arch = "wasm32"))]
const SIZE: PhysicalSize<u32> = PhysicalSize::new(512, 512);
#[cfg(not(target_arch = "wasm32"))]
const TITLE: &str = "WGPU Program";

// winit application struct
pub struct App {
    pub state: Option<State>,
}

impl App {
    pub fn new() -> Self {
        Self { state: None }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Base window attributes
        let attrib = win_attrib();
        // Create the window
        let window = event_loop.create_window(attrib).unwrap();

        // WASM canvas element implementation

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

fn win_attrib() -> WindowAttributes {
    #[cfg(target_arch = "wasm32")]
    {
        // I may or may not have copied this from someone else's code
        // at least partially
        // gotta love webassembly and it's eccentricities
        use wgpu::web_sys::HtmlCanvasElement;
        use wgpu::web_sys::wasm_bindgen::JsCast;
        use winit::platform::web::WindowAttributesExtWebSys;
        let canvas = wgpu::web_sys::window()
            .and_then(|win| win.document())
            .and_then(|document| document.get_element_by_id("game"))
            .map(|elem| elem.dyn_into::<HtmlCanvasElement>().unwrap())
            .unwrap();

        WindowAttributes::default().with_canvas(Some(canvas))
    }

    // Configuration specific to desktop
    #[cfg(not(target_arch = "wasm32"))]
    {
        use winit::window::Icon;
        let max_size = PhysicalSize {
            width: SIZE.width * 2,
            height: SIZE.height * 2,
        };

        const ICON_DATA: &[u8] = include_bytes!("res/icon.png");

        let (bytes, w, h) = {
            let img = image::load_from_memory(ICON_DATA).unwrap().to_rgba8();
            let (w, h) = img.dimensions();
            let bytes = img.into_raw();
            (bytes, w, h)
        };

        let win_icon = Icon::from_rgba(bytes, w, h).unwrap();

        // Set stuff that only matters for desktops
        WindowAttributes::default()
            .with_title(TITLE)
            .with_inner_size(SIZE)
            .with_max_inner_size(max_size)
            .with_min_inner_size(SIZE)
            .with_window_icon(Some(win_icon))
    }
}
