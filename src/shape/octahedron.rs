use std::{
    cmp::Ordering,
    f32::consts::{FRAC_PI_2, FRAC_PI_3, SQRT_2},
    fmt::Debug,
    hash::{Hash, Hasher},
    ops::Not,
    slice::Iter,
    usize,
};

use bevy::{
    asset::RenderAssetUsages,
    ecs::system::Resource,
    math::{primitives::Cuboid, NormedVectorSpace, Vec3},
    render::mesh::{Indices, Mesh, PrimitiveTopology},
};

use bevy::math::primitives::Tetrahedron as BevyTetrahedron;
use itertools::{iproduct, repeat_n};
use maze_generator::{model::TraversalGraph, traversal_graph_generator::TraversalGraphGenerator};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::shape::platonic_solid::{BorderType, Edge, HasFace, IsRoom, PlatonicSolid};

use super::platonic_mesh_builder::PlatonicMeshBuilder;

const PHI: f32 = 1.618034;

const VERTICES: [[f32; 3]; 6] = [
    [1.0, 0.0, 0.0],
    [-1.0, 0.0, 0.0],
    [0.0, 1.0, 0.0],
    [0.0, -1.0, 0.0],
    [0.0, 0.0, 1.0],
    [0.0, 0.0, -1.0],
];

const FACES: [[usize; 3]; 8] = [
    [0, 2, 4],
    [0, 4, 3],
    [0, 3, 5],
    [0, 5, 2],
    [1, 4, 2],
    [1, 3, 4],
    [1, 5, 3],
    [1, 2, 5],
];

#[derive(Debug, Clone, Hash, Eq, PartialEq, Copy, PartialOrd, Ord)]
pub struct OctahedronFace {
    face_indices: [usize; 3],
}

impl OctahedronFace {
    fn defining_vectors(&self) -> (Vec3, Vec3) {
        let vertices = self.vertices();
        let vec_1 = vertices[1] - vertices[0];
        let vec_2 = vertices[2] - vertices[0];
        (vec_1.normalize(), vec_2.normalize())
    }

    fn vertices(&self) -> [Vec3; 3] {
        self.face_indices
            .map(|index| Vec3::from_array(VERTICES[index]))
    }

    fn is_disconnected_from(&self, other: &OctahedronFace) -> bool {
        false
    }
}

impl HasFace for OctahedronFace {
    fn normal(&self) -> Vec3 {
        let (vec_1, vec_2) = self.defining_vectors();

        vec_1.cross(vec_2).normalize()
    }

    fn border_type(&self, other: &OctahedronFace) -> Option<BorderType> {
        let border_type = if self == other {
            BorderType::SameFace
        } else {
            BorderType::Connected
        };
        Some(border_type)
    }

    fn all_faces() -> Vec<OctahedronFace> {
        FACES
            .map(|face_indices| OctahedronFace { face_indices })
            .into_iter()
            .collect::<Vec<_>>()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct OctahedronRoom {
    pub position: Vec3,
    pub face_position: (u8, u8),
    pub face: OctahedronFace,
}

impl IsRoom<OctahedronFace> for OctahedronRoom {
    fn position(&self) -> Vec3 {
        self.position
    }

    fn face(&self) -> OctahedronFace {
        self.face
    }
}

impl Ord for OctahedronRoom {
    fn cmp(&self, other: &OctahedronRoom) -> Ordering {
        match self.face.cmp(&other.face) {
            Ordering::Equal => self.face_position.cmp(&other.face_position),
            ordering => ordering,
        }
    }
}

impl PartialOrd for OctahedronRoom {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Hash for OctahedronRoom {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.face_position.hash(state);
        self.face.hash(state);
    }
}

impl PartialEq for OctahedronRoom {
    fn eq(&self, other: &Self) -> bool {
        self.position.distance(other.position) < 0.01
    }
}

impl Eq for OctahedronRoom {}

#[derive(Resource, Clone)]
pub struct Octahedron {
    nodes_per_edge: u8,
    distance_between_nodes: f32,
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

    fn get_mesh(&self) -> Mesh {
        let scaling_factor = self.face_size / 2.0_f32.sqrt();

        let vertices = FACES
            .iter()
            .map(|face_indices| face_indices.iter().map(|i| VERTICES[*i]))
            .flatten()
            .collect::<Vec<[f32; 3]>>();

        let uvs = FACES
            .iter()
            .map(|face_indices| vec![[0.0_f32, 0.0], [1.0, 0.0], [0.0, 1.0]])
            .flatten()
            .collect::<Vec<[f32; 2]>>();

        let normals = FACES
            .iter()
            .map(|face_indices| {
                let vertices = face_indices
                    .iter()
                    .map(|i| VERTICES[*i])
                    .map(Vec3::from_array)
                    .collect::<Vec<Vec3>>();
                let defining_vector_1 = vertices[1] - vertices[0];
                let defining_vector_2 = vertices[2] - vertices[0];
                let normal = defining_vector_1.cross(defining_vector_2).normalize();
                repeat_n(normal.to_array(), 3)
            })
            .flatten()
            .collect::<Vec<[f32; 3]>>();

        let face_indices = (0..FACES.len() * 3)
            .map(|index| index as u16)
            .collect::<Vec<u16>>();

        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::RENDER_WORLD,
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_indices(Indices::U16(face_indices))
        .scaled_by(Vec3::ONE * scaling_factor)
    }
}

impl PlatonicSolid for Octahedron {
    type Face = OctahedronFace;
    type Room = OctahedronRoom;

    fn make_nodes_from_face(&self, face: &OctahedronFace) -> Vec<OctahedronRoom> {
        let (vec_i, vec_j) = face.defining_vectors();
        let normal = face.normal();

        let nodes_per_edge_float = self.nodes_per_edge as f32;
        let face_height_from_origin = self.face_size / 6.0_f32.sqrt();

        let max_abs_face_coord = (nodes_per_edge_float - 1.0) / 3.0;

        iproduct!(0..self.nodes_per_edge, 0..self.nodes_per_edge)
            .filter(|(i, j)| i + j <= self.nodes_per_edge - 1)
            .map(|(i, j)| {
                let face_x = i as f32;
                let face_y = j as f32;

                let face_coord_x = (face_x - max_abs_face_coord) * vec_i;
                let face_coord_y = (face_y - max_abs_face_coord) * vec_j;

                let face_coord = (face_coord_x + face_coord_y) * self.distance_between_nodes
                    + normal * face_height_from_origin;
                let position = face_coord;

                OctahedronRoom {
                    position,
                    face_position: (i, j),
                    face: face.clone(),
                }
            })
            .collect::<Vec<OctahedronRoom>>()
    }

    fn generate_traversal_graph(
        &self,
        nodes: Vec<OctahedronRoom>,
    ) -> TraversalGraph<OctahedronRoom, Edge> {
        let traversal_graph_generator = OctahedronTraversalGraphGenerator {
            distance_between_nodes: self.distance_between_nodes,
        };

        let traversal_graph = traversal_graph_generator.generate(nodes.clone());

        println!(
            "Produced traversal graph with {:?} edges",
            traversal_graph.all_edges().count()
        );

        traversal_graph
    }

    fn get_mesh_builder(&self) -> PlatonicMeshBuilder {
        let mesh = self.get_mesh();
        let dihedral_angle = (-1.0_f32 / 3.0).acos();
        PlatonicMeshBuilder::new(self.distance_between_nodes, dihedral_angle, mesh)
    }
}

struct OctahedronTraversalGraphGenerator {
    pub distance_between_nodes: f32,
}

impl TraversalGraphGenerator<OctahedronRoom, Edge> for OctahedronTraversalGraphGenerator {
    fn can_connect(&self, from: &OctahedronRoom, to: &OctahedronRoom) -> bool {
        let distance = from.position.distance(to.position);

        match from.face.border_type(&to.face) {
            Some(BorderType::SameFace) => distance - 0.1 <= self.distance_between_nodes,
            Some(BorderType::Connected) => {
                let cosine_of_dihedral_angle = -1.0_f32 / 3.0;
                let connected_edge_factor = ((1.0 - cosine_of_dihedral_angle) / 2.0).sqrt();
                distance - 0.1 <= self.distance_between_nodes * connected_edge_factor
            }
            _ => false,
        }
    }
}
