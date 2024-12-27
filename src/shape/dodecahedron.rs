use std::{
    cmp::Ordering,
    f32::consts::{FRAC_PI_2, FRAC_PI_3, PI, SQRT_2},
    fmt::Debug,
    hash::{Hash, Hasher},
    ops::Not,
    slice::Iter,
    usize,
};

use bevy::{
    asset::RenderAssetUsages,
    ecs::system::Resource,
    math::{primitives::Cuboid, NormedVectorSpace, Vec3, VectorSpace},
    render::mesh::{Indices, Mesh, PrimitiveTopology},
};

use bevy::math::primitives::Tetrahedron as BevyTetrahedron;
use itertools::repeat_n;
use maze_generator::{model::TraversalGraph, traversal_graph_generator::TraversalGraphGenerator};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::shape::platonic_solid::{BorderType, Edge, HasFace, IsRoom, PlatonicSolid};

use super::platonic_mesh_builder::PlatonicMeshBuilder;

const PHI: f32 = 1.618034;

const VERTICES: [[f32; 3]; 20] = [
    [1.0 / PHI, PHI, 0.0], //
    [PHI, 0.0, 1.0 / PHI],
    [0.0, 1.0 / PHI, PHI], //
    [-1.0 / PHI, -PHI, 0.0],
    [-PHI, 0.0, 1.0 / PHI],
    [0.0, -1.0 / PHI, PHI],
    [1.0 / PHI, -PHI, 0.0],
    [PHI, 0.0, -1.0 / PHI],
    [0.0, -1.0 / PHI, -PHI],
    [-1.0 / PHI, PHI, 0.0], //
    [-PHI, 0.0, -1.0 / PHI],
    [0.0, 1.0 / PHI, -PHI],
    [1.0, 1.0, 1.0], //
    [1.0, -1.0, 1.0],
    [-1.0, -1.0, 1.0],
    [-1.0, 1.0, 1.0], //
    [1.0, -1.0, -1.0],
    [1.0, 1.0, -1.0],
    [-1.0, 1.0, -1.0],
    [-1.0, -1.0, -1.0],
];

const FACES: [[usize; 5]; 12] = [
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

#[derive(Debug, Clone, Hash, Eq, PartialEq, Copy, PartialOrd, Ord)]
pub struct DodecahedronFace {
    face_indices: [usize; 5],
}

impl DodecahedronFace {
    fn defining_vectors(&self) -> (Vec3, Vec3) {
        let vertices = self.vertices();
        let vec_1 = vertices[1] - vertices[0];
        let vec_2 = vertices[2] - vertices[0];
        (vec_1.normalize(), vec_2.normalize())
    }

    fn vertices(&self) -> [Vec3; 5] {
        self.face_indices
            .map(|index| Vec3::from_array(VERTICES[index]))
            .map(|vertex| vertex * PHI / 2.0)
    }

    fn is_disconnected_from(&self, other: &DodecahedronFace) -> bool {
        false
    }
}

impl HasFace for DodecahedronFace {
    fn normal(&self) -> Vec3 {
        let (vec_1, vec_2) = self.defining_vectors();

        vec_1.cross(vec_2).normalize()
    }

    fn border_type(&self, other: &DodecahedronFace) -> Option<BorderType> {
        let border_type = if self == other {
            BorderType::SameFace
        } else {
            BorderType::Connected
        };
        Some(border_type)
    }

    fn all_faces() -> Vec<DodecahedronFace> {
        FACES
            .map(|face_indices| DodecahedronFace { face_indices })
            .into_iter()
            .collect::<Vec<_>>()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DodecahedronRoom {
    pub position: Vec3,
    pub face_position: usize,
    pub face: DodecahedronFace,
}

impl IsRoom<DodecahedronFace> for DodecahedronRoom {
    fn position(&self) -> Vec3 {
        self.position
    }

    fn face(&self) -> DodecahedronFace {
        self.face
    }
}

impl Ord for DodecahedronRoom {
    fn cmp(&self, other: &DodecahedronRoom) -> Ordering {
        match self.face.cmp(&other.face) {
            Ordering::Equal => self.face_position.cmp(&other.face_position),
            ordering => ordering,
        }
    }
}

impl PartialOrd for DodecahedronRoom {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Hash for DodecahedronRoom {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.face_position.hash(state);
        self.face.hash(state);
    }
}

impl PartialEq for DodecahedronRoom {
    fn eq(&self, other: &Self) -> bool {
        self.position.distance(other.position) < 0.01
    }
}

impl Eq for DodecahedronRoom {}

#[derive(Resource, Clone)]
pub struct Dodecahedron {
    distance_between_nodes: f32,
    face_size: f32,
    node_from_edge_lerp_factor: f32,
}

impl Dodecahedron {
    pub fn new(face_size: f32) -> Self {
        let tan_27 = (0.15 * PI).tan();
        let distance_between_nodes = face_size * tan_27;

        let tan_54 = (0.3 * PI).tan();
        let node_from_edge_lerp_factor = tan_27 / tan_54;
        Self {
            distance_between_nodes,
            face_size,
            node_from_edge_lerp_factor,
        }
    }

    fn get_mesh(&self) -> Mesh {
        let scaling_factor = self.face_size * PHI / 2.0;

        let vertices = FACES
            .iter()
            .map(|face_indices| face_indices.iter().map(|i| VERTICES[*i]))
            .flatten()
            .collect::<Vec<[f32; 3]>>();

        let uvs = FACES
            .iter()
            .map(|face_indices| {
                vec![
                    [0.0_f32, 0.0],
                    [1.0, 0.0],
                    [0.0, 1.0],
                    [0.5, 1.0],
                    [0.0, 0.5],
                ]
            })
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
                repeat_n(normal.to_array(), 5)
            })
            .flatten()
            .collect::<Vec<[f32; 3]>>();

        let pentagonal_face_triangle_indices = [0, 1, 2, 0, 2, 3, 0, 3, 4];

        let face_indices = (0..FACES.len())
            .flat_map(|face_index| {
                pentagonal_face_triangle_indices
                    .into_iter()
                    .map(move |pentagon_index| (pentagon_index + face_index * 5) as u16)
            })
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

impl PlatonicSolid for Dodecahedron {
    type Face = DodecahedronFace;
    type Room = DodecahedronRoom;

    fn make_nodes_from_face(&self, face: &DodecahedronFace) -> Vec<DodecahedronRoom> {
        let face_height_from_origin = self.face_size * PHI.powi(2) / (3.0 - PHI).sqrt() / 2.0;
        let face_center = face.vertices().into_iter().sum::<Vec3>() / 5.0;

        let vertices = face.vertices();

        let pairs = [
            (vertices[0], vertices[1]),
            (vertices[1], vertices[2]),
            (vertices[2], vertices[3]),
            (vertices[3], vertices[4]),
            (vertices[4], vertices[0]),
        ];

        pairs
            .into_iter()
            .map(|(vertex, adjacent)| vertex.lerp(adjacent, 0.5))
            .map(|edge_midpoint| edge_midpoint.lerp(face_center, self.node_from_edge_lerp_factor))
            .enumerate()
            .map(|(face_position, position)| DodecahedronRoom {
                position,
                face_position,
                face: face.clone(),
            })
            .collect::<Vec<DodecahedronRoom>>()
    }

    fn generate_traversal_graph(
        &self,
        nodes: Vec<DodecahedronRoom>,
    ) -> TraversalGraph<DodecahedronRoom, Edge> {
        let traversal_graph_generator = IcosahedronTraversalGraphGenerator {
            distance_between_nodes: self.distance_between_nodes,
        };

        let traversal_graph = traversal_graph_generator.generate(nodes.clone());

        println!(
            "Produced traversal graph with {:?} nodes and {:?} edges",
            traversal_graph.nodes().count(),
            traversal_graph.all_edges().count()
        );

        traversal_graph
    }

    fn get_mesh_builder(&self) -> PlatonicMeshBuilder {
        let mesh = self.get_mesh();
        let dihedral_angle = (-5.0_f32.sqrt() / 5.0).acos();
        PlatonicMeshBuilder::new(self.distance_between_nodes, dihedral_angle, mesh)
    }
}

struct IcosahedronTraversalGraphGenerator {
    pub distance_between_nodes: f32,
}

impl TraversalGraphGenerator<DodecahedronRoom, Edge> for IcosahedronTraversalGraphGenerator {
    fn can_connect(&self, from: &DodecahedronRoom, to: &DodecahedronRoom) -> bool {
        let distance = from.position.distance(to.position);

        match from.face.border_type(&to.face) {
            Some(BorderType::SameFace) => distance - 0.1 <= self.distance_between_nodes,
            Some(BorderType::Connected) => {
                let cosine_of_dihedral_angle = -5.0_f32.sqrt() / 5.0;
                let connected_edge_factor = ((1.0 - cosine_of_dihedral_angle) / 2.0).sqrt();
                distance - 0.1 <= self.distance_between_nodes * connected_edge_factor
            }
            _ => false,
        }
    }
}
