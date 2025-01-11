use std::{
    fmt::Debug,
    hash::{DefaultHasher, Hash, Hasher},
};

use bevy::{
    asset::RenderAssetUsages,
    ecs::{component::Component, system::Resource},
    math::Vec3,
    render::mesh::{Indices, Mesh, PrimitiveTopology},
};
use itertools::{iproduct, repeat_n};

use crate::{
    room::{SolidFace, SolidRoom},
    shape::shape_loader::ShapeLoader,
};

const CUBE_VERTICES: [[f32; 3]; 8] = [
    [-1.0, -1.0, -1.0],
    [-1.0, -1.0, 1.0],
    [-1.0, 1.0, -1.0],
    [-1.0, 1.0, 1.0],
    [1.0, -1.0, -1.0],
    [1.0, -1.0, 1.0],
    [1.0, 1.0, -1.0],
    [1.0, 1.0, 1.0],
];

const CUBE_FACES: [[usize; 4]; 6] = [
    [0, 2, 6, 4],
    [0, 1, 3, 2],
    [6, 7, 5, 4],
    [2, 3, 7, 6],
    [4, 5, 1, 0],
    [5, 7, 3, 1],
];

#[derive(Resource, Component, Clone, Debug)]
pub struct Cube {
    nodes_per_edge: u8,
    pub distance_between_nodes: f32,
    face_size: f32,
}

impl Cube {
    pub fn new(nodes_per_edge: u8, face_size: f32) -> Self {
        let distance_between_nodes = face_size / (nodes_per_edge as f32);
        Self {
            nodes_per_edge,
            distance_between_nodes,
            face_size,
        }
    }

    fn defining_vectors(vertex_indices: &[usize; 4]) -> (Vec3, Vec3) {
        let vertices = Self::vertices(vertex_indices);
        let vec_1 = vertices[1] - vertices[0];
        let vec_2 = vertices[3] - vertices[0];
        (vec_1.normalize(), vec_2.normalize())
    }
}

impl ShapeLoader<8, 6, 4> for Cube {
    const VERTICES: [[f32; 3]; 8] = CUBE_VERTICES;
    const FACES: [[usize; 4]; 6] = CUBE_FACES;

    fn make_nodes_from_face(&self, face: &SolidFace) -> Vec<SolidRoom> {
        let vertex_indices = CUBE_FACES[face.id()];

        let (vec_i, vec_j) = Self::defining_vectors(&vertex_indices);
        let normal = face.normal();

        let nodes_per_edge_float = self.nodes_per_edge as f32;

        let mut hasher = DefaultHasher::new();

        iproduct!(0..self.nodes_per_edge, 0..self.nodes_per_edge)
            .map(|(i, j)| {
                let face_x = i as f32;
                let face_y = j as f32;

                let abs_max_face_coord = (nodes_per_edge_float - 1.0) / 2.0;
                let face_coord_x = (face_x - abs_max_face_coord) * vec_i;
                let face_coord_y = (face_y - abs_max_face_coord) * vec_j;

                let face_coord = face_coord_x + face_coord_y + nodes_per_edge_float * normal / 2.0;
                let position = face_coord * self.face_size / nodes_per_edge_float;
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

    fn get_face_mesh(&self, vertices: [Vec3; 4]) -> Mesh {
        let scaling_factor = self.face_size / 2.0;

        let uvs = vec![[0.0_f32, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];

        let defining_vector_1 = vertices[1] - vertices[0];
        let defining_vector_2 = vertices[2] - vertices[0];
        let normal = defining_vector_1.cross(defining_vector_2).normalize();
        let normals = repeat_n(normal.to_array(), 4).collect::<Vec<[f32; 3]>>();

        let face_indices = vec![0_u16, 1, 2, 0, 2, 3];

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
}
