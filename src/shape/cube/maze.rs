use std::{
    cmp::Ordering,
    fmt::Debug,
    hash::{Hash, Hasher},
    ops::Not,
};

use bevy::{ecs::system::Resource, math::Vec3};
use itertools::iproduct;
use maze_generator::{
    config::Maze,
    model::{Door, TraversalGraph},
    traversal_graph_generator::TraversalGraphGenerator,
};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

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

impl Face for CubeFace {
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

#[derive(Debug, Eq, PartialEq)]
pub enum BorderType {
    SameFace,
    Connected,
}
#[derive(Debug, Clone, Copy)]
pub struct CubeNode {
    pub position: Vec3,
    pub face_position: (u8, u8),
    pub face: CubeFace,
}

impl Room<CubeFace> for CubeNode {
    fn position(&self) -> Vec3 {
        self.position
    }

    fn face(&self) -> CubeFace {
        self.face
    }
}

impl Ord for CubeNode {
    fn cmp(&self, other: &CubeNode) -> Ordering {
        match self.face.cmp(&other.face) {
            Ordering::Equal => self.face_position.cmp(&other.face_position),
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
pub struct CubeEdge;

impl<R> Door<R> for CubeEdge {
    fn is_directed(&self) -> bool {
        false
    }

    fn door_path_weight(&self) -> u16 {
        1
    }

    fn get_all_doors() -> Vec<Self> {
        vec![CubeEdge]
    }
}

pub trait Face: IntoEnumIterator {
    fn normal(&self) -> Vec3;
    fn border_type(&self, other: &Self) -> Option<BorderType>;
}

pub trait Room<F: Face> {
    fn position(&self) -> Vec3;
    fn face(&self) -> F;
}

pub trait PlatonicSolid {
    type MazeFace: Face;
    type MazeRoom: Debug + Clone + Copy + Hash + Eq + Ord + PartialOrd + Room<Self::MazeFace>;

    fn make_nodes_from_face(
        face: Self::MazeFace,
        nodes_per_edge: u8,
        distance_between_nodes: f32,
    ) -> Vec<Self::MazeRoom>;

    fn generate_traversal_graph(
        distance_between_nodes: f32,
        nodes: Vec<Self::MazeRoom>,
    ) -> TraversalGraph<Self::MazeRoom, CubeEdge>;
}

pub struct Cube;

impl PlatonicSolid for Cube {
    type MazeFace = CubeFace;
    type MazeRoom = CubeNode;

    fn make_nodes_from_face(
        face: CubeFace,
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

    fn generate_traversal_graph(
        distance_between_nodes: f32,
        nodes: Vec<CubeNode>,
    ) -> TraversalGraph<CubeNode, CubeEdge> {
        let traversal_graph_generator = CubeTraversalGraphGenerator {
            distance_between_nodes,
        };

        traversal_graph_generator.generate(nodes.clone())
    }
}

#[derive(Resource)]
pub struct CubeMaze<P: PlatonicSolid> {
    pub distance_between_nodes: f32,
    pub maze: Maze<P::MazeRoom, CubeEdge>,
}

impl<P: PlatonicSolid> CubeMaze<P> {
    pub fn build(nodes_per_edge: u8, face_size: f32) -> CubeMaze<P> {
        let distance_between_nodes = face_size / ((1 + nodes_per_edge) as f32);
        let nodes = Self::make_nodes(nodes_per_edge, distance_between_nodes);

        let traversal_graph = P::generate_traversal_graph(distance_between_nodes, nodes.clone());
        let maze = Maze::build(traversal_graph);

        CubeMaze::<P> {
            distance_between_nodes,
            maze,
        }
    }

    fn make_nodes(nodes_per_edge: u8, distance_between_nodes: f32) -> Vec<P::MazeRoom> {
        P::MazeFace::iter()
            .flat_map(|face| P::make_nodes_from_face(face, nodes_per_edge, distance_between_nodes))
            .collect()
    }
}

struct CubeTraversalGraphGenerator {
    pub distance_between_nodes: f32,
}

impl TraversalGraphGenerator<CubeNode, CubeEdge> for CubeTraversalGraphGenerator {
    fn can_connect(&self, from: &CubeNode, to: &CubeNode) -> bool {
        let distance = from.position.distance(to.position);

        match from.face.border_type(&to.face) {
            Some(BorderType::SameFace) => distance - 0.1 <= self.distance_between_nodes,
            Some(BorderType::Connected) => distance - 0.1 <= self.distance_between_nodes * 0.8,
            _ => false,
        }
    }
}
