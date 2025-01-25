use std::{f32::consts::SQRT_2, fmt::Debug, usize};

use bevy::{
    ecs::{component::Component, system::Resource},
    math::Vec3,
    render::mesh::Mesh,
};

use crate::{
    constants::SQRT_3,
    room::{Face, Room},
};

use super::{shape_loader::face_indices_to_vertices, triangle_face_generator};

const VERTEX_SCALING_FACTOR: f32 = SQRT_2 / 2.0;

const OCTAHEDRON_VERTICES: [Vec3; 6] = [
    Vec3::new(1.0, 0.0, 0.0),
    Vec3::new(-1.0, 0.0, 0.0),
    Vec3::new(0.0, 1.0, 0.0),
    Vec3::new(0.0, -1.0, 0.0),
    Vec3::new(0.0, 0.0, 1.0),
    Vec3::new(0.0, 0.0, -1.0),
];

pub const OCTAHEDRON_FACES: [[usize; 3]; 8] = [
    [0, 2, 4],
    [0, 4, 3],
    [0, 3, 5],
    [0, 5, 2],
    [1, 4, 2],
    [1, 3, 4],
    [1, 5, 3],
    [1, 2, 5],
];

#[derive(Resource, Component, Clone, Debug)]
pub struct Octahedron {
    nodes_per_edge: u8,
    pub distance_between_nodes: f32,
}

impl Octahedron {
    pub const fn new(nodes_per_edge: u8) -> Self {
        let distance_between_nodes = 1.0 / (nodes_per_edge as f32 - 1.0 + SQRT_3);

        Self {
            nodes_per_edge,
            distance_between_nodes,
        }
    }

    pub fn get_faces() -> [[Vec3; 3]; 8] {
        face_indices_to_vertices(OCTAHEDRON_FACES, &OCTAHEDRON_VERTICES)
            .map(|face_vertices| face_vertices.map(|vertex| vertex * VERTEX_SCALING_FACTOR))
    }
}
