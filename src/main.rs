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
mod constants;
mod shader;

use std::error::*;
use std::env::*;
use application_handler::*;
use env_logger::*;
use log::*;
use winit::event_loop::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    init_env_logger();
    info!("App started.");
    let current_exe_dir = match current_exe() {
        #[expect(clippy::unwrap_used, reason = "current_exe returns the absolute path to the executable file, so there must be a parent directory.")]
        Ok(current_exe_path) => current_exe_path.parent().unwrap().to_path_buf(),
        Err(err) => {
            error!("Could not get the path to the executable. Error: {err:#?}");
            return Err(err.into());
        }
    };
    if let Err(err) = set_current_dir(current_exe_dir) {
        error!("Could not set the current working directory to the directory of the executable. Error: {err:#?}");
        return Err(err.into());
    }
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
