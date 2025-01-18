use std::{fmt::Debug, usize};

use bevy::{
    ecs::{component::Component, system::Resource},
    math::Vec3,
    render::mesh::Mesh,
};

use crate::{
    constants::SQRT_3,
    room::{Face, SolidRoom},
    shape::shape_loader::ShapeMeshLoader,
};

use super::triangle_face_generator;

const OCTAHEDRON_VERTICES: [[f32; 3]; 6] = [
    [1.0, 0.0, 0.0],
    [-1.0, 0.0, 0.0],
    [0.0, 1.0, 0.0],
    [0.0, -1.0, 0.0],
    [0.0, 0.0, 1.0],
    [0.0, 0.0, -1.0],
];

const OCTAHEDRON_FACES: [[usize; 3]; 8] = [
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
}

impl ShapeMeshLoader<6, 8, 3> for Octahedron {
    const VERTICES: [[f32; 3]; 6] = OCTAHEDRON_VERTICES;
    const FACES: [[usize; 3]; 8] = OCTAHEDRON_FACES;

    fn make_nodes_from_face(&self, face: &Face) -> Vec<SolidRoom> {
        let vertex_indices = OCTAHEDRON_FACES[face.id()];

        let vertices = Self::vertices(&vertex_indices);

        let face_height_from_origin = 1.0 / 6.0_f32.sqrt();

        triangle_face_generator::make_nodes_from_face(
            face,
            vertices,
            self.nodes_per_edge,
            self.distance_between_nodes,
            face_height_from_origin,
        )
    }

    fn get_face_mesh(&self, vertices: [Vec3; 3]) -> Mesh {
        let scaling_factor = 1.0 / 2.0_f32.sqrt();
        triangle_face_generator::get_mesh(vertices, scaling_factor)
    }
}
