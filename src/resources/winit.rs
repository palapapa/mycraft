use std::sync::*;
use bevy_ecs::resource::*;
use winit::window::*;

#[derive(Resource)]
pub struct WinitResource {
    /// <https://www.reddit.com/r/rust/comments/1csjakb/comment/l45os9v>
    pub window: Arc<Window>
}