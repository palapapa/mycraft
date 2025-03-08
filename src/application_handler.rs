use log::{error, info};
use winit::application::ApplicationHandler;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};
use winit::event::WindowEvent::{self, *};

/// The default implementation for [`ApplicationHandler`].
#[derive(Default)]
pub struct App {
    window: Option<Window>,
    has_window_been_initialized: bool
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        info!("App resumed.");
        if self.has_window_been_initialized {
            return;
        }
        let window_attributes = Window::default_attributes().with_title("Mycraft");
        self.window  = match event_loop.create_window(window_attributes) {
            Ok(window) => {
                self.has_window_been_initialized = true;
                Some(window)
            },
            Err(err) => {
                error!("Could not create the window. Error: {err}");
                return;
            }
        };
        if let Some(window) = self.window.as_ref() {
            info!("The window with ID {} has been created.", u64::from(window.id()));
        }
        else {
            unreachable!(); // If the window creation failed, this function would have returned.
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            CloseRequested => {
                info!("The window with ID {} is exiting.", u64::from(window_id));
                event_loop.exit();
            }
            _ => {}
        }
    }
}
