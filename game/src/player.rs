use std::fmt::Debug;

use crate::{
    game_settings::GameSettings,
    room::SolidRoom,
};
use bevy::prelude::*;

#[derive(Component)]
pub struct Player {
    pub size: f32,
}

#[derive(Component, Debug)]
pub enum PlayerMazeState {
    Node(SolidRoom),
    Edge(SolidRoom, SolidRoom, Vec3),
}

pub fn move_player(
    mut player_query: Query<(&mut Transform, &PlayerMazeState, &Player)>,
    settings: Res<GameSettings>,
) {
    let Ok((mut player_transform, player_maze_state, Player { size })) =
        player_query.get_single_mut()
    else {
        return;
    };

    let target_position = match player_maze_state {
        PlayerMazeState::Node(node) => {
            let height_above_node = settings.player_elevation + size;
            node.position() + height_above_node * node.face().normal()
        }
        PlayerMazeState::Edge(_, _, edge_position) => edge_position.clone(),
    };

    player_transform.translation = player_transform.translation.lerp(target_position, 0.1)
}
