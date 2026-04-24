use {
    crate::{
        components::core::{MaterialComponent, MeshComponent},
        constants::{BIND_GROUP_COUNT, PER_MATERIAL_BIND_GROUP},
        material::{Material, MaterialPipelineKey},
        mesh::Mesh,
        resources::renderer::{RenderPipelineCacheResource, WgpuResource}
    },
    bevy_ecs::{
        query::{Changed, Or},
        system::{Query, Res, ResMut}
    },
    std::any::type_name,
    wgpu::{BindGroupLayout, PipelineLayoutDescriptor}
};

pub fn create_and_cache_pipelines<M: Material>(
    query: Query<
        '_,
        '_,
        (&MaterialComponent<M>, &MeshComponent),
        Or<(Changed<MaterialComponent<M>>, Changed<MeshComponent>)>
    >,
    pipeline_cache: ResMut<'_, RenderPipelineCacheResource<M>>,
    wgpu_reosurce: Res<'_, WgpuResource>
) {
    for (MaterialComponent { material_handle }, MeshComponent { mesh_handle }) in query {
        let device = &wgpu_reosurce.device;
        let material_key = material_handle.read(Material::material_key);
        let provided_vertex_attributes = mesh_handle.read(Mesh::vertex_attributes);
        let wanted_vertex_attributes = M::wanted_vertex_attributes(provided_vertex_attributes);
        let material_pipeline_key = MaterialPipelineKey {
            material_key,
            vertex_attributes: wanted_vertex_attributes
        };
        let material_bind_group_layout =
            device.create_bind_group_layout(&M::bind_group_layout(&material_pipeline_key));
        let mut bind_group_layouts: [Option<&BindGroupLayout>; BIND_GROUP_COUNT] =
            [None; BIND_GROUP_COUNT];
        bind_group_layouts[PER_MATERIAL_BIND_GROUP] = Some(&material_bind_group_layout);
        // bind_group_layouts[PER_VIEW_BIND_GROUP] = ;
        // bind_group_layouts[PER_OBJECT_BIND_GROUP] = ;
        let pipeline_lauout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some(&format!("{}-pipeline-layout", type_name::<M>())),
            #[expect(
                clippy::unwrap_used,
                reason = "We have filled all BIND_GROUP_COUNT slots in bind_group_layouts above."
            )]
            bind_group_layouts: &bind_group_layouts.map(|layout| layout.unwrap()),
            push_constant_ranges: &[]
        });
        // let pipeline_descriptor = RenderPipelineDescriptor {
        //     label: Some(&format!("{}-render-pipeline", type_name::<M>())),
        //     layout: (),
        //     vertex: (),
        //     primitive: (),
        //     depth_stencil: (),
        //     multisample: (),
        //     fragment: (),
        //     multiview: (),
        //     cache: ()
        // };
    }
}
