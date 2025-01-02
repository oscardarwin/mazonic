use bevy::prelude::*;

use crate::{
    player::PlayerMazeState,
    shape::{
        loader::{LevelLoadData, PlatonicLevelData},
        platonic_solid::PlatonicSolid,
    },
};

#[derive(Resource)]
pub struct PlayerPath<P: PlatonicSolid>(pub Vec<P::Room>);

pub fn setup_statistics<P: PlatonicSolid>(mut commands: Commands) {
    commands.insert_resource(PlayerPath::<P>(vec![]));
}

pub fn update_player_path<P: PlatonicSolid>(
    mut player_path_resource: ResMut<PlayerPath<P>>,
    level_data: Res<PlatonicLevelData<P>>,
    player_query: Query<&PlayerMazeState<P>>,
) {
    match player_query.get_single() {
        Ok(PlayerMazeState::Node(current_node)) => {
            let PlayerPath(path) = player_path_resource.into_inner();

            if let Some(previous_node) = path.last().filter(|node| *node == current_node) {
                return;
            } else {
                println!("moved to new node: {:?}", current_node);
                (*path).push(current_node.clone());
            }
        }
        _ => {}
    }
}

enum PlatonicObject<T> {
    Cube(T),
    Tetrahedron(T),
}

impl<T> PlatonicObject<T> {}
