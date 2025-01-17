use bevy::{
    asset::RenderAssetUsages,
    math::Vec3,
    render::mesh::{Indices, Mesh, PrimitiveTopology},
};
use itertools::{iproduct, repeat_n};

use crate::room::{Face, SolidRoom};
use std::hash::{DefaultHasher, Hash, Hasher};

pub fn make_nodes_from_face(
    face: &Face,
    vertices: [Vec3; 3],
    nodes_per_edge: u8,
    distance_between_nodes: f32,
    face_height_from_origin: f32,
) -> Vec<SolidRoom> {
    let vec_i = (vertices[1] - vertices[0]).normalize();
    let vec_j = (vertices[2] - vertices[0]).normalize();

    let nodes_per_edge_float = nodes_per_edge as f32;

    let max_abs_face_coord = (nodes_per_edge_float - 1.0) / 3.0;
    let mut hasher = DefaultHasher::new();

    iproduct!(0..nodes_per_edge, 0..nodes_per_edge)
        .filter(|(i, j)| i + j <= nodes_per_edge - 1)
        .map(|(i, j)| {
            let face_x = i as f32;
            let face_y = j as f32;

            let face_coord_x = (face_x - max_abs_face_coord) * vec_i;
            let face_coord_y = (face_y - max_abs_face_coord) * vec_j;

            let face_coord = (face_coord_x + face_coord_y) * distance_between_nodes
                + face.normal * face_height_from_origin;
            let position = face_coord;

            (i, j, face.id).hash(&mut hasher);

            let id = hasher.finish();

            SolidRoom {
                position,
                id,
                face: face.clone(),
            }
        })
        .collect::<Vec<SolidRoom>>()
}

pub fn get_mesh(vertices: [Vec3; 3], scaling_factor: f32) -> Mesh {
    let uvs = vec![[0.0_f32, 0.0], [1.0, 0.0], [0.0, 1.0]];
    let defining_vector_1 = vertices[1] - vertices[0];
    let defining_vector_2 = vertices[2] - vertices[0];
    let normal = defining_vector_1.cross(defining_vector_2).normalize();
    let normals = repeat_n(normal.to_array(), 3).collect::<Vec<[f32; 3]>>();

    let face_indices = vec![0_u16, 1, 2];

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vertices.into_iter().collect::<Vec<Vec3>>(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_indices(Indices::U16(face_indices))
    .scaled_by(Vec3::ONE * scaling_factor)
}
