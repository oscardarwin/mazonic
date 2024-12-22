use std::{
    cmp::Ordering,
    f32::consts::FRAC_PI_2,
    fmt::Debug,
    hash::{Hash, Hasher},
    ops::Not,
};

use bevy::{
    ecs::system::Resource,
    math::{primitives::Cuboid, Vec3},
};
use itertools::iproduct;
use maze_generator::{model::TraversalGraph, traversal_graph_generator::TraversalGraphGenerator};
use strum_macros::EnumIter;

use crate::shape::platonic_solid::{BorderType, Edge, HasFace, IsRoom, PlatonicSolid};

use super::platonic_mesh_builder::PlatonicMeshBuilder;

#[derive(EnumIter, Debug, Clone, Hash, Eq, PartialEq, Copy, PartialOrd, Ord)]
pub enum CubeFace {
    Front,
    Left,
    Right,
    Up,
    Down,
    Back,
}

impl CubeFace {
    fn defining_vectors(&self) -> (Vec3, Vec3) {
        match self {
            CubeFace::Right => (-Vec3::Y, Vec3::Z),
            CubeFace::Left => (Vec3::Y, Vec3::Z),
            CubeFace::Back => (-Vec3::X, Vec3::Z),
            CubeFace::Front => (Vec3::X, Vec3::Z),
            CubeFace::Up => (-Vec3::X, Vec3::Y),
            CubeFace::Down => (Vec3::X, Vec3::Y),
        }
    }

    fn is_disconnected_from(&self, other: &CubeFace) -> bool {
        match (self, other) {
            (CubeFace::Front, CubeFace::Back) => true,
            (CubeFace::Up, CubeFace::Down) => true,
            (CubeFace::Left, CubeFace::Right) => true,
            (CubeFace::Back, CubeFace::Front) => true,
            (CubeFace::Down, CubeFace::Up) => true,
            (CubeFace::Right, CubeFace::Left) => true,
            _ => false,
        }
    }
}

impl HasFace for CubeFace {
    fn normal(&self) -> Vec3 {
        let (vec_1, vec_2) = self.defining_vectors();

        vec_1.cross(vec_2)
    }

    fn border_type(&self, other: &CubeFace) -> Option<BorderType> {
        self.is_disconnected_from(other).not().then(|| {
            if self == other {
                BorderType::SameFace
            } else {
                BorderType::Connected
            }
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CubeRoom {
    pub position: Vec3,
    pub face_position: (u8, u8),
    pub face: CubeFace,
}

impl IsRoom<CubeFace> for CubeRoom {
    fn position(&self) -> Vec3 {
        self.position
    }

    fn face(&self) -> CubeFace {
        self.face
    }
}

impl Ord for CubeRoom {
    fn cmp(&self, other: &CubeRoom) -> Ordering {
        match self.face.cmp(&other.face) {
            Ordering::Equal => self.face_position.cmp(&other.face_position),
            ordering => ordering,
        }
    }
}

impl PartialOrd for CubeRoom {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Hash for CubeRoom {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.face_position.hash(state);
        self.face.hash(state);
    }
}

impl PartialEq for CubeRoom {
    fn eq(&self, other: &Self) -> bool {
        self.position.distance(other.position) < 0.01
    }
}

impl Eq for CubeRoom {}

#[derive(Resource)]
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
}

impl PlatonicSolid for Cube {
    type Face = CubeFace;
    type Room = CubeRoom;

    fn make_nodes_from_face(&self, face: CubeFace) -> Vec<CubeRoom> {
        let (vec_i, vec_j) = face.defining_vectors();
        let normal = face.normal();

        let nodes_per_edge_float = self.nodes_per_edge as f32;

        iproduct!(0..self.nodes_per_edge, 0..self.nodes_per_edge)
            .map(|(i, j)| {
                let face_x = i as f32;
                let face_y = j as f32;

                let abs_max_face_coord = (nodes_per_edge_float - 1.0) / 2.0;
                let face_coord_x = (face_x - abs_max_face_coord) * vec_i;
                let face_coord_y = (face_y - abs_max_face_coord) * vec_j;

                let face_coord = face_coord_x + face_coord_y + nodes_per_edge_float * normal / 2.0;
                let position = face_coord * self.face_size / nodes_per_edge_float;

                CubeRoom {
                    position,
                    face_position: (i, j),
                    face: face.clone(),
                }
            })
            .collect::<Vec<CubeRoom>>()
    }

    fn generate_traversal_graph(&self, nodes: Vec<CubeRoom>) -> TraversalGraph<CubeRoom, Edge> {
        let traversal_graph_generator = CubeTraversalGraphGenerator {
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
        let mesh = Cuboid::from_length(self.face_size);

        PlatonicMeshBuilder::new(self.distance_between_nodes, FRAC_PI_2, mesh.into())
    }
}

struct CubeTraversalGraphGenerator {
    pub distance_between_nodes: f32,
}

impl TraversalGraphGenerator<CubeRoom, Edge> for CubeTraversalGraphGenerator {
    fn can_connect(&self, from: &CubeRoom, to: &CubeRoom) -> bool {
        let distance = from.position.distance(to.position);

        match from.face.border_type(&to.face) {
            Some(BorderType::SameFace) => distance - 0.1 <= self.distance_between_nodes,
            Some(BorderType::Connected) => distance - 0.1 <= self.distance_between_nodes * 0.8,
            _ => false,
        }
    }
}
