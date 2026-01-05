use const_format::*;

/// The bind group index (the number inside `@group()` in the shader to use)
/// that contains bindings that only need to be updated once per frame per
/// camera. The view matrix is one such binding.
pub const PER_VIEW_BIND_GROUP: i32 = 0;

/// The bind group index (the number inside `@group()` in the shader to use)
/// that the [`wgpu::BindGroup`] returned by
/// [`crate::material::Material::bind_group`] will be put. All objects using the
/// same [`crate::material::Material`] will use the same [`wgpu::BindGroup`] at
/// this index.
pub const PER_MATERIAL_BIND_GROUP: i32 = 1;

/// The bind group index (the number inside `@group()` in the shader to use)
/// that contains bindings that need to be updated for every object to render.
/// The model matrix is one such binding.
pub const PER_OBJECT_BIND_GROUP: i32 = 2;

/// The path to the assets directory. This path is relative to the location of
/// the executable.
pub const ASSETS_PATH: &str = "assets";

/// The path to the directory that stores shaders in the assets directory. This
/// path is relative to the location of the executable.
#[expect(unused_qualifications, reason = "Seems to be a false positive.")]
pub const SHADERS_PATH: &str = formatcp!("{ASSETS_PATH}/shaders");
