use bon::bon;
use getset::*;
use glam::*;
use log::*;
use mesh_builder::*;
use strum::*;
use thiserror::*;

/// # Invariants
/// - The length of [`Self::indices`] must be a multiple of 3, including 0.
/// - [`Self::indices`] can never contain indices that index any of the vertex
///   attributes that are not empty out of bounds.
/// - If [`Self::indices`] is empty, it is implied to contain \[0, 1, ..., L -
///   1\] where L is the length of the vertex attribute that is not empty with
///   the least number of elements, or the length of any of them if they all
///   have the same length.
/// - If all vertex attributes are empty, then [`Self::indices`] must also be
///   empty. This is a special state where this [`Mesh`] is considered "empty".
/// 
/// These invariants are checked when the [`Mesh`] is built.
///
/// This `struct` is immutable in order to not allow the user to violate these
/// invariants. It wouldn't be very useful even if it were mutable, because
/// [`Mesh`]es are used by [`bevy_ecs::component::Component`]s in the form of
/// [`crate::asset::AssetHandle`]s, which do not provide a way to mutate the
/// assets they point to.
/// 
/// To create a [`Mesh`], use [`Self::builder`], or convert a `struct` that
/// implements [`AsMeshBuilder`] into one, such as those defined in
/// [`crate::shapes`]. Any vertex attribute that is not set in the builder will
/// default to being empty.
/// 
/// It is possible to build a nonsensical [`Mesh`], like one whose only
/// non-empty vertex attribute is [`Self::colors`]. However, such [`Mesh`] will
/// fail to be rendered by any normal shader anyway, so such situation is not
/// checked here.
#[derive(Getters)]
#[get = "pub"]
pub struct Mesh {
    vertices: Vec<Vec3>,
    indices: Vec<usize>,
    uv: Vec<Vec2>,
    colors: Vec<U8Vec4>
}

#[bon]
impl Mesh {
    #[builder(state_mod(vis = "pub"))]
    pub fn new(
        #[builder(default)]
        vertices: Vec<Vec3>,
        #[builder(default)]
        indices: Vec<usize>,
        #[builder(default)]
        uv: Vec<Vec2>,
        #[builder(default)]
        colors: Vec<U8Vec4>
    ) -> Result<Self, MeshCreationError> {
        if vertices.len() % 3 != 0 {
            let err = MeshCreationError::IndicesLengthNotMultipleOf3 {
                indices_length: vertices.len()
            };
            error!("Mesh creation failed: {err}");
            return Err(err);
        }
        // This will be None if all vertex attributes are empty.
        let shortest_attribute_length_and_name = vec![
            (vertices.len(), "vertices"),
            (uv.len(), "uv"),
            (colors.len(), "colors")]
            .into_iter()
            .filter(|pair| pair.0 != 0)
            .min_by_key(|pair| pair.0);
        match shortest_attribute_length_and_name {
            None => {
                if let indices_length @ 1.. = indices.len() {
                    let err = MeshCreationError::EmptyMeshWithNonemptyIndices { indices_length };
                    error!("Mesh creation failed: {err}");
                    return Err(err);
                }
                // Empty Mesh.
                Ok(Self { vertices, indices, uv, colors })
            }
            Some((shortest_attribute_length, shortest_attribute_name)) => {
                if indices.is_empty() {
                    // Using the implied [0, 1, ..., L - 1] indices.
                    return Ok(Self { vertices, indices, uv, colors });
                }
                #[expect(clippy::unwrap_used, reason = "We have already checked that indices.len() is not 0.")]
                let max_index = indices.iter().copied().max().unwrap();
                if max_index > shortest_attribute_length - 1 {
                    let err = MeshCreationError::IndicesOutOfBounds {
                        out_of_bounds_vertex_attribute_name: shortest_attribute_name,
                        out_of_bounds_vertex_attribute_length: shortest_attribute_length,
                        max_index
                    };
                    error!("Mesh creation failed: {err}");
                    return Err(err)
                }
                // This is the typical situation where you have some vertex
                // attributes and vertices is not empty.
                Ok(Self { vertices, indices, uv, colors })
            }
        }
    }
}

pub trait AsMeshBuilder<BuilderState: State> {
    fn as_mesh_builder(&self) -> MeshBuilder<BuilderState>;
}

#[derive(Debug, Error)]
pub enum MeshCreationError {
    #[error("The length of `indices` must be a multiple of 3, but it is {}.", .indices_length)]
    IndicesLengthNotMultipleOf3 {
        indices_length: usize
    },
    #[error("`indices` contains {}, which indexes the vertex attribute {} out of bounds, whose length is only {}.",
        .max_index,
        .out_of_bounds_vertex_attribute_name,
        .out_of_bounds_vertex_attribute_length)
    ]
    IndicesOutOfBounds {
        /// An attribute name is a `struct` field name, so it is always
        /// `'static`.
        out_of_bounds_vertex_attribute_name: &'static str,
        out_of_bounds_vertex_attribute_length: usize,
        max_index: usize,
    },
    #[error("All vertex attributes are empty, but `indices` is not empty. It's length is {}", .indices_length)]
    EmptyMeshWithNonemptyIndices {
        indices_length: usize
    }
}

/// Used in a [`crate::material::Material`] to specify which vertex attributes
/// it needs and which `@location()` to assign to each of them in the shader.
/// 
/// This `enum` must have the default 0-based discriminants; otherwise, the
/// return value of
/// [`crate::material::Material::attribute_to_shader_location_mapping`] will be
/// indexed out of bounds.
#[derive(EnumCount)]
#[repr(usize)]
pub enum VertexAttributeKind {
    Positions,
    Uv,
    Colors
}
