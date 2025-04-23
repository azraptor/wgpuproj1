use winit::event_loop::{ControlFlow, EventLoop};

// Modules
mod app;
mod camera;
mod model;
mod state;
mod texture;
mod vert;

use crate::app::App;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

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
    #[cfg(not(target_arch = "wasm32"))]
    event_loop.set_control_flow(ControlFlow::Poll);
    #[cfg(target_arch = "wasm32")]
    event_loop.set_control_flow(ControlFlow::Wait); // Removes input lag in webgl

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
