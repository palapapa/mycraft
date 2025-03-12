mod application_handler;

use std::process::ExitCode;
use application_handler::App;
use env_logger::{Builder, Env};
use log::{info, error};
use winit::event_loop::{ControlFlow, EventLoop};

#[tokio::main]
async fn main() -> ExitCode {
    init_env_logger();
    info!("App started.");
    let event_loop = match EventLoop::new() {
        Ok(event_loop) => event_loop,
        Err(err) => {
            error!("Could not create the event loop. Error: {err}");
            return ExitCode::FAILURE;
        }
    };
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App::default();
    match event_loop.run_app(&mut app) {
        Ok(()) => (),
        Err(err) => {
            error!("The event loop terminated abnormally. Error: {err}");
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
