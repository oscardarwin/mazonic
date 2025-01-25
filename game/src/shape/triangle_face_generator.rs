use bevy::{
    asset::RenderAssetUsages,
    math::Vec3,
    render::mesh::{Indices, Mesh, PrimitiveTopology},
};
use itertools::{iproduct, repeat_n};

use crate::room::{Face, Room};
use std::hash::{DefaultHasher, Hash, Hasher};

pub fn make_nodes_from_face(
    face: &Face,
    vertices: [Vec3; 3],
    nodes_per_edge: u8,
    distance_between_nodes: f32,
    face_height_from_origin: f32,
) -> Vec<Room> {
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

            Room {
                position,
                id,
                face: face.clone(),
            }
        })
        .collect::<Vec<Room>>()
}
