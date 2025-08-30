use crate::mesh::{*, mesh_builder::*};
use glam::*;
use std::borrow::*;

pub struct Cuboid {
    /// The size of the cuboid in the order of X, Y, and Z. When any of them is
    /// negative, the cuboid gets mirrored along that axis.
    pub dimensions: Vec3
}

impl<T: Borrow<Cuboid>> AsMeshBuilder<SetUv<SetIndices<SetVertices>>> for T {
    /// Creates a [`MeshBuilder`] that has [`MeshBuilder::vertices`] and
    /// [`MeshBuilder::uv`] set with 24 vertices and UVs of the cuboid, 4 for
    /// each face, and [`MeshBuilder::indices`] set with 36 indices, 6 for each
    /// face. The UVs are arranged such that the +X, -X, +Z, -Z faces have (0,
    /// 0) as the UV in the bottom left corner, and (1, 1) in the upper right
    /// corner when the cuboid is viewed from the side. The faces are in the
    /// order of +X, -X, +Y, -Y, +Z, -Z.
    /// 
    /// Refer to `diagrams/cuboid.png` for a diagram of the mesh being built.
    fn as_mesh_builder(&self) -> MeshBuilder<SetUv<SetIndices<SetVertices>>> {
        let max = self.borrow().dimensions * 0.5;
        let min = -max;
        let vertices = vec![
            // +X
            Vec3::from_array([max.x, min.y, min.z]), // 0
            Vec3::from_array([max.x, min.y, max.z]), // 1
            Vec3::from_array([max.x, max.y, max.z]), // 2
            Vec3::from_array([max.x, max.y, min.z]), // 3
            // -X
            Vec3::from_array([min.x, min.y, max.z]), // 4
            Vec3::from_array([min.x, min.y, min.z]), // 5
            Vec3::from_array([min.x, max.y, min.z]), // 6
            Vec3::from_array([min.x, max.y, max.z]), // 7
            // +Y
            Vec3::from_array([min.x, max.y, min.z]), // 8
            Vec3::from_array([max.x, max.y, min.z]), // 9
            Vec3::from_array([max.x, max.y, max.z]), // 10
            Vec3::from_array([min.x, max.y, max.z]), // 11
            // -Y
            Vec3::from_array([min.x, min.y, max.z]), // 12
            Vec3::from_array([max.x, min.y, max.z]), // 13
            Vec3::from_array([max.x, min.y, min.z]), // 14
            Vec3::from_array([min.x, min.y, min.z]), // 15
            // +Z
            Vec3::from_array([max.x, min.y, max.z]), // 16
            Vec3::from_array([min.x, min.y, max.z]), // 17
            Vec3::from_array([min.x, max.y, max.z]), // 18
            Vec3::from_array([max.x, max.y, max.z]), // 19
            // -Z
            Vec3::from_array([min.x, min.y, min.z]), // 20
            Vec3::from_array([max.x, min.y, min.z]), // 21
            Vec3::from_array([max.x, max.y, min.z]), // 22
            Vec3::from_array([min.x, max.y, min.z]), // 23
        ];
        let indices = vec![
            0, 1, 2, 2, 3, 0,
            4, 5, 6, 6, 7, 4,
            8, 9, 10, 10, 11, 8,
            12, 13, 14, 14, 15, 12,
            16, 17, 18, 18, 19, 16,
            20, 21, 22, 22, 23, 20
        ];
        let uv: Vec<Vec2> = vec![
            Vec2::from_array([0.0, 0.0]),
            Vec2::from_array([1.0, 0.0]),
            Vec2::from_array([1.0, 1.0]),
            Vec2::from_array([0.0, 1.0])
        ].into_iter().cycle().take(24).collect();
        Mesh::builder().vertices(vertices).indices(indices).uv(uv)
    }
}
