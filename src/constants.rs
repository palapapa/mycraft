use const_format::*;

/// The bind group index (the number inside `@group()` in the shader to use)
/// that contains bindings that only need to be updated once per frame per
/// camera. The view matrix is one such binding.
/// 
/// <https://github.com/gfx-rs/wgpu/wiki/Do's-and-Dont's#do-group-resource-bindings-by-the-change-frequency-start-from-the-lowest>
pub const PER_VIEW_BIND_GROUP: usize = 1;

/// The bind group index (the number inside `@group()` in the shader to use)
/// that the [`BindGroup`][wgpu::BindGroup] returned by
/// [`Material::bind_group`][crate::material::Material::bind_group] will be put.
/// All objects using the same [`Material`][crate::material::Material] will use
/// the same [`BindGroup`][wgpu::BindGroup] at this index.
/// 
/// <https://github.com/gfx-rs/wgpu/wiki/Do's-and-Dont's#do-group-resource-bindings-by-the-change-frequency-start-from-the-lowest>
pub const PER_MATERIAL_BIND_GROUP: usize = 0;

/// The bind group index (the number inside `@group()` in the shader to use)
/// that contains bindings that need to be updated for every object to render.
/// The model matrix is one such binding.
/// 
/// <https://github.com/gfx-rs/wgpu/wiki/Do's-and-Dont's#do-group-resource-bindings-by-the-change-frequency-start-from-the-lowest>
pub const PER_OBJECT_BIND_GROUP: usize = 2;

/// The number of [`BindGroup`][wgpu::BindGroup]s provided by the renderer in a
/// [`RenderPipeline`][`wgpu::RenderPipeline`]. Those are:
/// [`PER_VIEW_BIND_GROUP`], [`PER_MATERIAL_BIND_GROUP`], and
/// [`PER_OBJECT_BIND_GROUP`].
pub const BIND_GROUP_COUNT: usize = [PER_VIEW_BIND_GROUP, PER_MATERIAL_BIND_GROUP, PER_OBJECT_BIND_GROUP].len();

/// An array of WESL shader constants that can be used in shader code. The first
/// elements of the tuples are the names of the constants, and the second ones
/// are the values. This is normally injected into shader code using
/// [`Wesl::add_constants`][wesl::Wesl::add_constants].
pub const SHADER_CONSTANTS: &[(&str, f64)] = &[
    ("PER_MATERIAL_BIND_GROUP", PER_MATERIAL_BIND_GROUP as f64),
    ("PER_VIEW_BIND_GROUP", PER_VIEW_BIND_GROUP as f64),
    ("PER_OBJECT_BIND_GROUP", PER_OBJECT_BIND_GROUP as f64)
];

/// The path to the assets directory. This path is relative to the location of
/// the executable.
pub const ASSETS_PATH: &str = "assets";

/// The path to the directory that stores shaders in the assets directory. This
/// path is relative to the location of the executable.
#[expect(unused_qualifications, reason = "Seems to be a false positive.")]
pub const SHADERS_PATH: &str = formatcp!("{ASSETS_PATH}/shaders");
