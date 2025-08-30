pub enum ProjectionMode {
    Orthographic(OrthographicProjectionConfig),
    Perspective(PerspectiveProjectionConfig)
}

pub struct OrthographicProjectionConfig {
    pub width: f32,
    pub height: f32,
    pub near_clipping_plane_distance: f32,
    pub far_clipping_plane_distance: f32
}

pub struct PerspectiveProjectionConfig {
    /// The horizontal FOV in radians.
    pub horizontal_fov: f32,
    pub near_clipping_plane_distance: f32,
    pub far_clipping_plane_distance: f32
}
