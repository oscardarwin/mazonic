use std::{f32::consts::SQRT_2, fmt::Debug};

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

const TETRAHEDRON_VERTICES: [[f32; 3]; 4] = [
    [1.0, 1.0, 1.0],
    [-1.0, 1.0, -1.0],
    [-1.0, -1.0, 1.0],
    [1.0, -1.0, -1.0],
];

const TETRAHEDRON_FACES: [[usize; 3]; 4] = [[3, 2, 1], [0, 2, 3], [3, 1, 0], [0, 1, 2]];

#[derive(Resource, Component, Clone, Debug)]
pub struct Tetrahedron {
    nodes_per_edge: u8,
    pub distance_between_nodes: f32,
}

impl Tetrahedron {
    pub const fn new(nodes_per_edge: u8) -> Self {
        let distance_between_nodes = 1.0 / (nodes_per_edge as f32 - 1.0 + SQRT_3);

        Self {
            nodes_per_edge,
            distance_between_nodes,
        }
    }
}

impl ShapeMeshLoader<4, 4, 3> for Tetrahedron {
    const VERTICES: [[f32; 3]; 4] = TETRAHEDRON_VERTICES;
    const FACES: [[usize; 3]; 4] = TETRAHEDRON_FACES;

    fn make_nodes_from_face(&self, face: &Face) -> Vec<SolidRoom> {
        let vertex_indices = TETRAHEDRON_FACES[face.id()];

        let vertices = Self::vertices(&vertex_indices);

        let face_height_from_origin = 3.0_f32.sqrt() / 6.0 / SQRT_2;
        triangle_face_generator::make_nodes_from_face(
            face,
            vertices,
            self.nodes_per_edge,
            self.distance_between_nodes,
            face_height_from_origin,
        )
    }

    fn get_face_mesh(&self, vertices: [Vec3; 3]) -> Mesh {
        let scaling_factor = 1.0 / SQRT_2 / 2.0;
        triangle_face_generator::get_mesh(vertices, scaling_factor)
    }
}
