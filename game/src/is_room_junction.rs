use itertools::Itertools;
use petgraph::{graphmap::GraphMap, Directed, Direction};

use crate::{room::Room, shape::shape_loader::Edge};

pub fn is_junction(room: &Room, graph: &GraphMap<Room, Edge, Directed>) -> bool {
    let incoming_neighbors = graph.neighbors_directed(*room, Direction::Incoming);
    let outgoing_neighbors = graph.neighbors_directed(*room, Direction::Outgoing);

    let neighbors = incoming_neighbors
        .chain(outgoing_neighbors)
        .unique()
        .collect::<Vec<Room>>();

    neighbors.len() != 2 || {
        let first_neighbor_position = room.project_other_to_face(&neighbors[0]);
        let second_neighbor_position = room.project_other_to_face(&neighbors[1]);

        let node_to_first_vec = (room.position() - first_neighbor_position).normalize();
        let node_to_second_vec = (room.position() - second_neighbor_position).normalize();

        let dot_product = node_to_first_vec.dot(node_to_second_vec);
        dot_product > -0.9
    }
}
