use bevy::prelude::*;

use crate::{player::PlayerMazeState, room::SolidRoom};

#[derive(Resource)]
pub struct PlayerPath(pub Vec<SolidRoom>);

pub fn setup_statistics(mut commands: Commands) {
    commands.insert_resource(PlayerPath(vec![]));
}

pub fn update_player_path(
    player_path_resource: ResMut<PlayerPath>,
    player_query: Query<&PlayerMazeState>,
) {
    match player_query.get_single() {
        Ok(PlayerMazeState::Node(current_node)) => {
            let PlayerPath(path) = player_path_resource.into_inner();

            if path.last().filter(|node| *node == current_node).is_some() {
                return;
            } else {
                (*path).push(current_node.clone());
            }
        }
        _ => {}
    }
}
