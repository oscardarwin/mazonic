use std::{
    cmp::Ordering,
    hash::{Hash, Hasher},
    ops::Not,
};

use bevy::{ecs::system::Resource, math::Vec3};
use itertools::iproduct;
use maze_generator::{
    config::Maze, model::Door, traversal_graph_generator::TraversalGraphGenerator,
};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(EnumIter, Debug, Clone, Hash, Eq, PartialEq, Copy, PartialOrd, Ord)]
pub enum Face {
    Front,
    Left,
    Right,
    Up,
    Down,
    Back,
}

impl Face {
    pub fn normal(&self) -> Vec3 {
        let (vec_1, vec_2) = self.defining_vectors();

        vec_1.cross(vec_2)
    }

    fn defining_vectors(&self) -> (Vec3, Vec3) {
        match self {
            Face::Right => (-Vec3::Y, Vec3::Z),
            Face::Left => (Vec3::Y, Vec3::Z),
            Face::Back => (-Vec3::X, Vec3::Z),
            Face::Front => (Vec3::X, Vec3::Z),
            Face::Up => (-Vec3::X, Vec3::Y),
            Face::Down => (Vec3::X, Vec3::Y),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum BorderType {
    SameFace,
    Connected,
}

impl BorderType {
    pub fn from_faces(face_1: &Face, face_2: &Face) -> Option<BorderType> {
        BorderType::are_unconnected(face_1, face_2).not().then(|| {
            if face_1 == face_2 {
                BorderType::SameFace
            } else {
                BorderType::Connected
            }
        })
    }

    fn are_unconnected(face_1: &Face, face_2: &Face) -> bool {
        match (face_1, face_2) {
            (Face::Front, Face::Back) => true,
            (Face::Up, Face::Down) => true,
            (Face::Left, Face::Right) => true,
            (Face::Back, Face::Front) => true,
            (Face::Down, Face::Up) => true,
            (Face::Right, Face::Left) => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CubeNode {
    pub position: Vec3,
    pub face_position: (u8, u8),
    pub face: Face,
}

impl CubeNode {
    fn compare_same_face(&self, other: &CubeNode) -> Ordering {
        self.face_position.cmp(&other.face_position)
    }
}

impl Ord for CubeNode {
    fn cmp(&self, other: &CubeNode) -> Ordering {
        match self.face.cmp(&other.face) {
            Ordering::Equal => self.compare_same_face(other),
            ordering => ordering,
        }
    }
}

impl PartialOrd for CubeNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Hash for CubeNode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.face_position.hash(state);
        self.face.hash(state);
    }
}

impl PartialEq for CubeNode {
    fn eq(&self, other: &Self) -> bool {
        self.position.distance(other.position) < 0.01
    }
}

impl Eq for CubeNode {}

#[derive(Debug, Clone, Hash, Eq, PartialEq, PartialOrd, Default)]
pub struct Edge;

impl Door<CubeNode> for Edge {
    fn is_directed(&self) -> bool {
        false
    }

    fn door_path_weight(&self) -> u16 {
        1
    }

    fn get_all_doors() -> Vec<Self> {
        vec![Edge]
    }
}

#[derive(Resource)]
pub struct CubeMaze {
    pub nodes_per_edge: u8,
    pub face_size: f32,
    pub distance_between_nodes: f32,
    pub maze: Maze<CubeNode, Edge>,
}

impl CubeMaze {
    pub fn build(nodes_per_edge: u8, face_size: f32) -> CubeMaze {
        let distance_between_nodes = face_size / ((1 + nodes_per_edge) as f32);
        let nodes = Self::make_nodes(nodes_per_edge, distance_between_nodes);

        let traversal_graph_generator = CubeTraversalGraphGenerator {
            distance_between_nodes,
        };

        let traversal_graph = traversal_graph_generator.generate(nodes.clone());
        let maze = Maze::build(traversal_graph);

        CubeMaze {
            nodes_per_edge,
            face_size,
            distance_between_nodes,
            maze,
        }
    }

    fn make_nodes(nodes_per_edge: u8, distance_between_nodes: f32) -> Vec<CubeNode> {
        Face::iter()
            .flat_map(|face| {
                Self::make_nodes_from_face(face, nodes_per_edge, distance_between_nodes)
            })
            .collect()
    }

    fn make_nodes_from_face(
        face: Face,
        nodes_per_edge: u8,
        distance_between_nodes: f32,
    ) -> Vec<CubeNode> {
        let (vec_i, vec_j) = face.defining_vectors();
        let normal = face.normal();

        let nodes_per_edge_float = nodes_per_edge as f32;

        iproduct!(0..nodes_per_edge, 0..nodes_per_edge)
            .map(|(i, j)| {
                let face_x = i as f32;
                let face_y = j as f32;

                let abs_max_face_coord = nodes_per_edge_float - 1.0;
                let face_coord_x = (2.0 * face_x - abs_max_face_coord) * vec_i;
                let face_coord_y = (2.0 * face_y - abs_max_face_coord) * vec_j;

                let face_coord = face_coord_x + face_coord_y + nodes_per_edge_float * normal;
                let position = face_coord * distance_between_nodes / 2.0;

                CubeNode {
                    position,
                    face_position: (i, j),
                    face: face.clone(),
                }
            })
            .collect::<Vec<CubeNode>>()
    }
}

struct CubeTraversalGraphGenerator {
    pub distance_between_nodes: f32,
}

impl TraversalGraphGenerator<CubeNode, Edge> for CubeTraversalGraphGenerator {
    fn can_connect(&self, from: &CubeNode, to: &CubeNode) -> bool {
        let distance = from.position.distance(to.position);

        match BorderType::from_faces(&from.face, &to.face) {
            Some(BorderType::SameFace) => distance - 0.1 <= self.distance_between_nodes,
            Some(BorderType::Connected) => distance - 0.1 <= self.distance_between_nodes * 0.8,
            _ => false,
        }
    }
}
