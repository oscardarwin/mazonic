use bevy::math::Vec3;
use itertools::iproduct;
use maze_generator::model::{ConnectivityGenerator, Door};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(EnumIter, Debug, Clone, Hash, Eq, PartialEq)]
pub enum Face {
    Front,
    Left,
    Right,
    Up,
    Down,
    Back,
}

impl Face {
    fn normal(&self) -> Vec3 {
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

pub enum BorderType {
    SameFace,
    Connected,
    Unconnected,
}

impl BorderType {
    fn get_from_faces(face_1: &Face, face_2: &Face) -> BorderType {
        if face_1 == face_2 {
            BorderType::SameFace
        } else if BorderType::are_unconnacted(face_1, face_2)
            || BorderType::are_unconnacted(face_2, face_1)
        {
            BorderType::Unconnected
        } else {
            BorderType::Connected
        }
    }

    fn are_unconnacted(face_1: &Face, face_2: &Face) -> bool {
        match (face_1, face_2) {
            (Face::Front, Face::Back) => true,
            (Face::Up, Face::Down) => true,
            (Face::Left, Face::Right) => true,
            _ => false,
        }
    }
}

#[derive(Debug)]
pub struct CubeNode {
    pub position: Vec3,
    face: Face,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
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

pub struct CubeGenerator {
    nodes_per_edge: u8,
    face_size: f32,
    distance_between_nodes: f32,
}

impl CubeGenerator {
    pub fn new(nodes_per_edge: u8, face_size: f32) -> CubeGenerator {
        let distance_between_nodes = face_size / ((1 + nodes_per_edge) as f32);

        CubeGenerator {
            nodes_per_edge,
            face_size,
            distance_between_nodes,
        }
    }

    pub fn make_nodes(&self) -> Vec<CubeNode> {
        Face::iter()
            .flat_map(|face| self.make_nodes_from_face(face))
            .collect()
    }

    fn make_nodes_from_face(&self, face: Face) -> Vec<CubeNode> {
        let (vec_i, vec_j) = face.defining_vectors();
        let normal = face.normal();

        iproduct!(0..self.nodes_per_edge, 0..self.nodes_per_edge)
            .map(|(i, j)| (i as f32, j as f32))
            .map(|(i, j)| {
                let face_coord_i = (2.0 * i - (self.nodes_per_edge as f32 - 1.0)) * vec_i;
                let face_coord_j = (2.0 * j - (self.nodes_per_edge as f32 - 1.0)) * vec_j;

                let max_distance_between_nodes = 2.0 * (self.nodes_per_edge as f32);
                let face_coord =
                    face_coord_i + face_coord_j + (self.nodes_per_edge as f32) * normal;
                let position = (self.face_size / max_distance_between_nodes) as f32 * face_coord;

                CubeNode {
                    position,
                    face: face.clone(),
                }
            })
            .collect::<Vec<CubeNode>>()
    }
}

impl ConnectivityGenerator<CubeNode, Edge> for CubeGenerator {
    fn can_connect(&self, from: &CubeNode, to: &CubeNode, with: &Edge) -> bool {
        let border_type = BorderType::get_from_faces(&from.face, &to.face);

        let distance = from.position.distance(to.position);

        match (border_type, distance) {
            (BorderType::Unconnected, _) => false,
            (BorderType::SameFace, distance) if distance - 0.1 <= self.distance_between_nodes => {
                true
            }
            (BorderType::Connected, distance)
                if distance - 0.1 <= self.distance_between_nodes * 1.5 =>
            {
                true
            }
            _ => false,
        }
    }
}
