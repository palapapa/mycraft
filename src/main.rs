mod application_handler;
mod asset;
mod components;
mod resources;
mod schedules;
mod system_sets;
mod systems;
mod world;
mod camera;
mod egui_renderer;
mod egui_state;
mod extensions;
mod material;
mod mesh;
mod shapes;

use std::error::*;
use application_handler::*;
use env_logger::*;
use log::*;
use winit::event_loop::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    init_env_logger();
    info!("App started.");
    let event_loop = match EventLoop::new() {
        Ok(event_loop) => event_loop,
        Err(err) => {
            error!("Could not create the event loop. Error: {err:#?}");
            return Err(err.into());
        }
    };
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App::new()?;
    match event_loop.run_app(&mut app) {
        Ok(()) => (),
        Err(err) => {
            error!("The event loop terminated abnormally. Error: {err:#?}");
            return Err(err.into());
        }
    }
    info!("Exiting.");
    Ok(())
}

fn init_env_logger() {
    let env = Env::new().filter_or("RUST_LOG", "info");
    Builder::from_env(env).format_timestamp_millis().init();
}
