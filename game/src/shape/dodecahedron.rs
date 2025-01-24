use std::{
    f32::consts::PI,
    fmt::Debug,
    hash::{DefaultHasher, Hash, Hasher},
    usize,
};

use bevy::{
    asset::RenderAssetUsages,
    ecs::{component::Component, system::Resource},
    math::Vec3,
    render::mesh::{Indices, Mesh, PrimitiveTopology},
};

use itertools::repeat_n;

use crate::{
    constants::{PHI, TAN_27},
    room::{Face, Room},
    shape::shape_loader::ShapeMeshLoader,
};

const DODECAHEDRON_VERTICES: [[f32; 3]; 20] = [
    [1.0 / PHI, PHI, 0.0],
    [PHI, 0.0, 1.0 / PHI],
    [0.0, 1.0 / PHI, PHI],
    [-1.0 / PHI, -PHI, 0.0],
    [-PHI, 0.0, 1.0 / PHI],
    [0.0, -1.0 / PHI, PHI],
    [1.0 / PHI, -PHI, 0.0],
    [PHI, 0.0, -1.0 / PHI],
    [0.0, -1.0 / PHI, -PHI],
    [-1.0 / PHI, PHI, 0.0],
    [-PHI, 0.0, -1.0 / PHI],
    [0.0, 1.0 / PHI, -PHI],
    [1.0, 1.0, 1.0],
    [1.0, -1.0, 1.0],
    [-1.0, -1.0, 1.0],
    [-1.0, 1.0, 1.0],
    [1.0, -1.0, -1.0],
    [1.0, 1.0, -1.0],
    [-1.0, 1.0, -1.0],
    [-1.0, -1.0, -1.0],
];

const DODECAHEDRON_FACES: [[usize; 5]; 12] = [
    [0, 9, 15, 2, 12],
    [0, 17, 11, 18, 9],
    [0, 12, 1, 7, 17],
    [1, 13, 6, 16, 7],
    [1, 12, 2, 5, 13],
    [2, 15, 4, 14, 5],
    [3, 6, 13, 5, 14],
    [3, 19, 8, 16, 6],
    [3, 14, 4, 10, 19],
    [4, 15, 9, 18, 10],
    [7, 16, 8, 11, 17],
    [8, 19, 10, 18, 11],
];

#[derive(Resource, Component, Clone, Debug)]
pub struct Dodecahedron {
    pub distance_between_nodes: f32,
}

impl Dodecahedron {
    pub const fn new() -> Self {
        let distance_between_nodes = TAN_27;
        Self {
            distance_between_nodes,
        }
    }
}

impl ShapeMeshLoader<20, 12, 5> for Dodecahedron {
    const VERTICES: [[f32; 3]; 20] = DODECAHEDRON_VERTICES;
    const FACES: [[usize; 5]; 12] = DODECAHEDRON_FACES;

    fn make_nodes_from_face(&self, face: &Face) -> Vec<Room> {
        let vertex_indices = DODECAHEDRON_FACES[face.id()];
        let vertices = Self::vertices(&vertex_indices).map(|vertex| vertex * PHI / 2.0);

        let face_center = vertices.into_iter().sum::<Vec3>() / 5.0;

        let pairs = [
            (vertices[0], vertices[1]),
            (vertices[1], vertices[2]),
            (vertices[2], vertices[3]),
            (vertices[3], vertices[4]),
            (vertices[4], vertices[0]),
        ];

        let tan_54 = (0.3 * PI).tan();
        let node_from_edge_lerp_factor = self.distance_between_nodes / tan_54;

        let mut hasher = DefaultHasher::new();
        pairs
            .into_iter()
            .map(|(vertex, adjacent)| vertex.lerp(adjacent, 0.5))
            .map(|edge_midpoint| edge_midpoint.lerp(face_center, node_from_edge_lerp_factor))
            .enumerate()
            .map(|(id, position)| {
                (id, face.id).hash(&mut hasher);
                let id = hasher.finish();

                Room {
                    position,
                    face: face.clone(),
                    id,
                }
            })
            .collect::<Vec<Room>>()
    }

    fn get_face_mesh(&self, vertices: [Vec3; 5]) -> Mesh {
        let scaling_factor = PHI / 2.0;
        let uvs = vec![
            [0.0_f32, 0.0],
            [1.0, 0.0],
            [0.0, 1.0],
            [0.5, 1.0],
            [0.0, 0.5],
        ];

        let defining_vector_1 = vertices[1] - vertices[0];
        let defining_vector_2 = vertices[2] - vertices[0];
        let normal = defining_vector_1.cross(defining_vector_2).normalize();
        let normals = repeat_n(normal.to_array(), 5).collect::<Vec<[f32; 3]>>();

        let face_indices = vec![0_u16, 1, 2, 0, 2, 3, 0, 3, 4];

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
