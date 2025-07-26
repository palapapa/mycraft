mod application_handler;
mod egui_renderer;
mod shaders;
mod ui_renderer;
mod ui_state;

use std::process::*;
use application_handler::*;
use env_logger::*;
use log::*;
use winit::event_loop::*;
use crate::ui_renderer::*;

#[tokio::main]
async fn main() -> ExitCode {
    init_env_logger();
    info!("App started.");
    let event_loop = match EventLoop::new() {
        Ok(event_loop) => event_loop,
        Err(err) => {
            error!("Could not create the event loop. Error: {err:#?}");
            return ExitCode::FAILURE;
        }
    };
    event_loop.set_control_flow(ControlFlow::Poll);
    let ui_renderers: Vec<Box<dyn UiRenderer>> = vec![Box::new(DefaultUiRenderer)];
    let mut app = App::new(ui_renderers);
    match event_loop.run_app(&mut app) {
        Ok(()) => (),
        Err(err) => {
            error!("The event loop terminated abnormally. Error: {err:#?}");
            return ExitCode::FAILURE;
        }
    }
    info!("Exiting.");
    ExitCode::SUCCESS
}

fn init_env_logger() {
    let env = Env::new().filter_or("RUST_LOG", "info");
    Builder::from_env(env).format_timestamp_millis().init();
}
