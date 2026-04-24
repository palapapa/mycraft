use {
    crate::{mesh::VertexAttributes, shader::ShaderId},
    bevy_ecs::system::{SystemParam, SystemParamItem},
    wgpu::{
        BindGroup, BindGroupDescriptor, BindGroupLayout, BindGroupLayoutDescriptor, Device,
        RenderPipelineDescriptor
    }
};

/// A [`Material`] determines how an object should be rendered. It contains the
/// shaders to use and the data to pass to the shaders.
pub trait Material: Sized + Send + Sync {
    /// A [`SystemParam`] that will be passed to [`Material::bind_group`]. You
    /// can use this type to query any value from the
    /// [`World`][bevy_ecs::world::World] to help yourself create the
    /// [`BindGroupDescriptor`].
    type BindGroupCreationParams: SystemParam;

    /// This type is used to uniquely identify a [`Material`] variant and aid in
    /// pipeline caching.
    ///
    /// For example, an implementation might support an optional normal map. If
    /// there are two instances of this [`Material`], one of which uses the
    /// normal map and the other doesn't, then they need two different
    /// [`RenderPipeline`]s because they would need to use different
    /// [`BindGroup`]s, [`BindGroupLayout`]s, and shader variants.
    ///
    /// Therefore, this type is used in [`MaterialPipelineKey::material_key`] to
    /// identify these variants so that different variants can find their
    /// correct pipelines. You should provide a type that contains the
    /// information needed to identify the variants, preferably small and cheap
    /// to hash. If no variants are needed, you can use `()`.
    ///
    /// This type doesn't need to contain information about optional vertex
    /// attributes, because that information is already present as
    /// [`MaterialPipelineKey::vertex_attributes`] and is calculated from
    /// [`Self::wanted_vertex_attributes`].
    type MaterialKey: Send + Sync;

    /// Creates the [`BindGroup`] that will be used to render an object. A
    /// [`BindGroupLayout`] created from the return value of
    /// [`Material::bind_group_layout`] is passed into this function as
    /// `layout`. The actual [`BindGroup`] will be created by the renderer. The
    /// created [`BindGroup`] will be the
    /// [`PER_MATERIAL_BIND_GROUP`][crate::constants::PER_MATERIAL_BIND_GROUP]-th
    /// bind group in the [`RenderPass`]. Use the `params` parameter to help
    /// yourself create the [`BindGroupDescriptor`].
    ///
    /// It's important that this is a pure function, meaning that the same
    /// `material_pipeline_key` value must always produce the same
    /// [`BindGroupDescriptor`]. Otherwise, pipeline caching won't work
    /// correctly.
    fn bind_group<'bgl>(
        &self,
        device: &Device,
        layout: &'bgl BindGroupLayout,
        params: &mut SystemParamItem<'_, '_, Self::BindGroupCreationParams>,
        material_pipeline_key: &MaterialPipelineKey<Self>
    ) -> BindGroupDescriptor<'bgl>;

    /// Creates the [`BindGroupLayoutDescriptor`] that this [`Material`] needs
    /// to render an object. The actual [`BindGroupLayout`] will be created by
    /// the renderer. You can use `material_pipeline_key` to determine the
    /// layout to use.
    ///
    /// It's important that this is a pure function, meaning that the same
    /// `material_pipeline_key` value must always produce the same
    /// [`BindGroupLayoutDescriptor`]. Otherwise, pipeline caching won't work
    /// correctly.
    fn bind_group_layout(
        material_pipeline_key: &MaterialPipelineKey<Self>
    ) -> BindGroupLayoutDescriptor<'_>;

    /// This function gets passed `provided`, the vertex attributes the [`Mesh`]
    /// to be rendered has, and should return a subset of attributes in
    /// `provided` to indicate what attributes the shader will actually use.
    ///
    /// This will not be called if the [`Mesh`] to be rendered doesn't satisfy
    /// [`Self::required_vertex_attributes`].
    ///
    /// This is used to calculate [`MaterialPipelineKey::vertex_attributes`].
    fn wanted_vertex_attributes(provided: VertexAttributes) -> VertexAttributes;

    /// This function should return the smallest set of vertex attributes needed
    /// for the shader used to function properly.
    ///
    /// If the [`Mesh`] to be rendered doesn't satisfy the returned value, the
    /// rendering will be aborted.
    fn required_vertex_attributes() -> VertexAttributes;

    /// The vertex shader to use. You can use `material_pipeline_key` to
    /// determine what shader features to enable.
    ///
    /// It's important that this is a pure function, meaning that the same
    /// `material_pipeline_key` value must always produce the same [`ShaderId`].
    /// Otherwise, pipeline caching won't work correctly.
    fn vertex_shader(material_pipeline_key: &MaterialPipelineKey<Self>) -> ShaderId;

    /// The fragment shader to use. You can use `material_pipeline_key` to
    /// determine what shader features to enable.
    ///
    /// It's important that this is a pure function, meaning that the same
    /// `material_pipeline_key` value must always produce the same [`ShaderId`].
    /// Otherwise, pipeline caching won't work correctly.
    ///
    /// By default, this just calls [`Self::vertex_shader`], which is useful if
    /// both shaders are in the same file.
    fn fragment_shader(material_pipeline_key: &MaterialPipelineKey<Self>) -> ShaderId {
        Self::vertex_shader(material_pipeline_key)
    }

    /// Get the [`Self::MaterialKey`] for this instance. See
    /// [`Material::MaterialKey`] for more detail.
    fn material_key(&self) -> Self::MaterialKey;

    /// This function allows customizing the [`RenderPipelineDescriptor`] that
    /// will be used to create the [`RenderPipeline`] that this [`Material`]
    /// will use to render objects. `pipeline_descriptor` will be preconfigured
    /// according to the other functions of [`Material`] and by the renderer.
    /// You can then further customize the settings.
    ///
    /// It's important that this is a pure function, meaning that the same
    /// `material_pipeline_key` value must always produce the same set of
    /// modifications. Otherwise, pipeline caching won't work correctly.
    ///
    /// If it is impossible to correctly configure the
    /// [`RenderPipelineDescriptor`], such as when the mesh to be rendered
    /// doesn't provide all the required vertex attributes, [`Err`] is returned.
    fn specialize_pipeline(
        pipeline_descriptor: &mut RenderPipelineDescriptor<'_>,
        material_pipeline_key: &MaterialPipelineKey<Self>
    );
}

pub struct BasicMaterial;

impl Material for BasicMaterial {
    type BindGroupCreationParams = ();
    type MaterialKey = ();

    fn bind_group<'bgl>(
        &self,
        device: &Device,
        layout: &'bgl BindGroupLayout,
        params: &mut SystemParamItem<'_, '_, Self::BindGroupCreationParams>,
        material_pipeline_key: &MaterialPipelineKey<Self>
    ) -> BindGroupDescriptor<'bgl> {
        todo!()
    }

    fn bind_group_layout(
        material_pipeline_key: &MaterialPipelineKey<Self>
    ) -> BindGroupLayoutDescriptor<'_> {
        BindGroupLayoutDescriptor {
            label: Some("basic-material-bind-group-layout"),
            entries: &[]
        }
    }

    fn wanted_vertex_attributes(provided: VertexAttributes) -> VertexAttributes {
        todo!()
    }

    fn required_vertex_attributes() -> VertexAttributes {
        todo!()
    }

    fn vertex_shader(material_pipeline_key: &MaterialPipelineKey<Self>) -> ShaderId {
        todo!()
    }

    fn material_key(&self) -> Self::MaterialKey {
        todo!()
    }

    fn specialize_pipeline(
        pipeline_descriptor: &mut RenderPipelineDescriptor<'_>,
        material_pipeline_key: &MaterialPipelineKey<Self>
    ) {
        todo!()
    }
}

/// A key type used to uniquely associate a [`Material`] instance with a
/// [`RenderPipeline`] so that it can be cached.
pub struct MaterialPipelineKey<M: Material> {
    /// See [`Material::MaterialKey`] for more detail.
    pub material_key: M::MaterialKey,

    /// The set of vertex attributes that `M` will use from the [`Mesh`] to be
    /// rendered.
    pub vertex_attributes: VertexAttributes
}
