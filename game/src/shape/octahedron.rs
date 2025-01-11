use std::{fmt::Debug, usize};

use bevy::{
    ecs::{component::Component, system::Resource},
    math::Vec3,
    render::mesh::Mesh,
};

use crate::{
    room::{SolidFace, SolidRoom},
    shape::shape_loader::ShapeLoader,
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
    face_size: f32,
}

impl Octahedron {
    pub fn new(nodes_per_edge: u8, face_size: f32) -> Self {
        let distance_between_nodes = face_size / (nodes_per_edge as f32 - 1.0 + 3.0_f32.sqrt());

        Self {
            nodes_per_edge,
            distance_between_nodes,
            face_size,
        }
    }
}

impl ShapeLoader<6, 8, 3> for Octahedron {
    const VERTICES: [[f32; 3]; 6] = OCTAHEDRON_VERTICES;
    const FACES: [[usize; 3]; 8] = OCTAHEDRON_FACES;

    fn make_nodes_from_face(&self, face: &SolidFace) -> Vec<SolidRoom> {
        let vertex_indices = OCTAHEDRON_FACES[face.id()];

        let vertices = Self::vertices(&vertex_indices);

        let face_height_from_origin = self.face_size / 6.0_f32.sqrt();

        triangle_face_generator::make_nodes_from_face(
            face,
            vertices,
            self.nodes_per_edge,
            self.distance_between_nodes,
            face_height_from_origin,
        )
    }

    fn get_face_mesh(&self, vertices: [Vec3; 3]) -> Mesh {
        let scaling_factor = self.face_size / 2.0_f32.sqrt();
        triangle_face_generator::get_mesh(vertices, scaling_factor)
    }
}
