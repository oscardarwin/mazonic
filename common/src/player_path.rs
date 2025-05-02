use bevy::prelude::*;

use crate::{player::PlayerMazeState, room::Room};

#[derive(Component, Debug, Clone)]
pub struct PlayerPath(pub Vec<Room>);

impl Default for PlayerPath {
    fn default() -> Self {
        Self(vec![])
    }
}

pub fn update(
    mut player_path_query: Query<&mut PlayerPath>,
    player_query: Query<&PlayerMazeState, Changed<PlayerMazeState>>,
) {
    let Ok(PlayerMazeState::Node(current_node)) = player_query.get_single() else {
        return;
    };

    let Ok(mut path) = player_path_query.get_single_mut() else {
        return;
    };

    if path.0.last().filter(|node| *node == current_node).is_some() {
        return;
    } else {
        (*path).0.push(current_node.clone());
    }
}
