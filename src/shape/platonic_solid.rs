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

#[derive(Debug, Eq, PartialEq)]
pub enum BorderType {
    SameFace,
    Connected,
}

pub trait HasFace: IntoEnumIterator {
    fn normal(&self) -> Vec3;
    fn border_type(&self, other: &Self) -> Option<BorderType>;
}

pub trait IsRoom<F: HasFace> {
    fn position(&self) -> Vec3;
    fn face(&self) -> F;
}

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

pub trait PlatonicSolid: Resource + Sized {
    type Face: HasFace;
    type Room: Debug
        + Clone
        + Copy
        + Hash
        + Eq
        + Ord
        + PartialOrd
        + Send
        + Sync
        + IsRoom<Self::Face>;

    fn make_nodes_from_face(&self, face: Self::Face) -> Vec<Self::Room>;

    fn generate_traversal_graph(
        &self,
        nodes: Vec<Self::Room>,
    ) -> TraversalGraph<Self::Room, CubeEdge>;

    fn build_maze(&self) -> Maze<Self::Room, CubeEdge> {
        let nodes = self.make_nodes();
        let traversal_graph = self.generate_traversal_graph(nodes.clone());
        let maze = Maze::build(traversal_graph);

        maze
    }

    fn make_nodes(&self) -> Vec<Self::Room> {
        Self::Face::iter()
            .flat_map(|face| self.make_nodes_from_face(face))
            .collect()
    }

    fn distance_between_nodes(&self) -> f32;
}