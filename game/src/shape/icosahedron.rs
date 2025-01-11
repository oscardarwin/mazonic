use std::{fmt::Debug, usize};

use bevy::{
    ecs::{component::Component, system::Resource},
    math::Vec3,
    render::mesh::Mesh,
};

use crate::{
    constants::PHI,
    room::{SolidFace, SolidRoom},
    shape::shape_loader::ShapeLoader,
};

use super::triangle_face_generator;

const ICOSAHEDRON_VERTICES: [[f32; 3]; 12] = [
    [1.0, PHI, 0.0],
    [1.0, -PHI, 0.0],
    [-1.0, PHI, 0.0],
    [-1.0, -PHI, 0.0],
    [0.0, 1.0, PHI],
    [0.0, 1.0, -PHI],
    [0.0, -1.0, PHI],
    [0.0, -1.0, -PHI],
    [PHI, 0.0, 1.0],
    [-PHI, 0.0, 1.0],
    [PHI, 0.0, -1.0],
    [-PHI, 0.0, -1.0],
];

const ICOSAHEDRON_FACES: [[usize; 3]; 20] = [
    [0, 4, 8],
    [0, 10, 5],
    [0, 8, 10],
    [4, 0, 2],
    [5, 2, 0],
    [8, 4, 6],
    [1, 8, 6],
    [1, 10, 8],
    [1, 7, 10],
    [7, 1, 3],
    [6, 3, 1],
    [2, 9, 4],
    [9, 6, 4],
    [3, 6, 9],
    [3, 9, 11],
    [2, 11, 9],
    [2, 5, 11],
    [11, 5, 7],
    [3, 11, 7],
    [10, 7, 5],
];

#[derive(Resource, Component, Clone, Debug)]
pub struct Icosahedron {
    nodes_per_edge: u8,
    pub distance_between_nodes: f32,
    face_size: f32,
}

impl Icosahedron {
    pub fn new(nodes_per_edge: u8, face_size: f32) -> Self {
        let distance_between_nodes = face_size / (nodes_per_edge as f32 - 1.0 + 3.0_f32.sqrt());

        Self {
            nodes_per_edge,
            distance_between_nodes,
            face_size,
        }
    }
}

impl ShapeLoader<12, 20, 3> for Icosahedron {
    const VERTICES: [[f32; 3]; 12] = ICOSAHEDRON_VERTICES;
    const FACES: [[usize; 3]; 20] = ICOSAHEDRON_FACES;

    fn make_nodes_from_face(&self, face: &SolidFace) -> Vec<SolidRoom> {
        let vertex_indices = ICOSAHEDRON_FACES[face.id()];

        let vertices = Self::vertices(&vertex_indices);

        let face_height_from_origin = self.face_size * PHI.powi(2) / 3.0_f32.sqrt() / 2.0;

        triangle_face_generator::make_nodes_from_face(
            face,
            vertices,
            self.nodes_per_edge,
            self.distance_between_nodes,
            face_height_from_origin,
        )
    }

    fn get_face_mesh(&self, vertices: [Vec3; 3]) -> Mesh {
        let scaling_factor = self.face_size / 2.0;
        triangle_face_generator::get_mesh(vertices, scaling_factor)
    }
}
