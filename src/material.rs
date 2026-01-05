use crate::mesh::*;
use crate::shader::*;
use assets_manager::*;
use strum::*;
use wgpu::*;

/// A [`Material`] determines how an object should be rendered. It contains the
/// shaders to use and the data to pass to the shaders.
pub trait Material {
    /// Creates the [`BindGroup`] that will be used to render an object. The
    /// return value of [`Material::bind_group_layout`] is passed into this
    /// function as `layout`. The returned [`BindGroup`] will be the
    /// [`crate::constants::PER_MATERIAL_BIND_GROUP`]-th bind group in the
    /// [`RenderPass`].
    fn bind_group(&self, device: &Device, layout: &BindGroupLayout) -> BindGroup;

    /// Creates the [`BindGroupLayout`] that [`Material::bind_group`] will use
    /// to create a [`BindGroup`].
    fn bind_group_layout(&self, device: &Device) -> BindGroupLayout;

    /// The vertex shader to use. [`WeslShader`] implements [`Asset`], so you
    /// would typically want to load one using `asset_cache`.
    fn vertex_shader(&self, asset_cache: &AssetCache) -> WeslShader;

    /// If the returned value is [`Some`], it will be the name of the vertex
    /// shader function to use in the shader specified by
    /// [`Material::vertex_shader_path`]; otherwise, the one marked with
    /// `@vertex` will be used. Defaults to [`None`].
    fn vertex_shader_entry_point(&self) -> Option<String> {
        None
    }

    /// The fragment shader to use. [`WeslShader`] implements [`Asset`], so you
    /// would typically want to load one using `asset_cache`.
    fn fragment_shader(&self, asset_cache: &AssetCache) -> WeslShader;

    /// If the returned value is [`Some`], it will be the name of the fragment
    /// shader function to use in the shader specified by
    /// [`Material::fragment_shader_path`]; otherwise, the one marked with
    /// `@fragment` will be used. Defaults to [`None`].
    fn fragment_shader_entry_point(&self) -> Option<String> {
        None
    }

    /// Returns which vertex attributes this [`Material`] requires from a
    /// [`Mesh`] and which `@location()` to map them to in the vertex shader.
    /// For each element in the returned array, its index corresponds to the
    /// discriminant of a [`VertexAttributeKind`] variant, and its value
    /// specifies which `@location()` to map that vertex attribute to in the
    /// vertex shader, or [`None`] if that vertex attribute is not needed by the
    /// vertex shader. For example, if you want to map the vertex positions to
    /// `@location(0)`, you can write:
    /// 
    /// ```
    /// let mut result = [None; VertexAttributeKind::COUNT];
    /// result[VertexAttributeKind::Positions as usize] = Some(0);
    /// return result;
    /// ```
    fn attribute_to_shader_location_mapping(&self) -> [Option<u32>; VertexAttributeKind::COUNT];
}
