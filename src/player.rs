use std::fmt::Debug;

use crate::{
    game_settings::GameSettings,
    shape::{
        cube::Cube,
        loader::{load_maze, PlatonicLevelData},
        platonic_solid::{HasFace, IsRoom, PlatonicSolid},
        tetrahedron::Tetrahedron,
    },
};
use bevy::prelude::*;
use bevy_rapier3d::geometry::Collider;

#[derive(Component)]
pub struct Player;

#[derive(Component, Debug)]
pub enum PlayerMazeState<P: PlatonicSolid> {
    Node(P::Room),
    Edge(P::Room, P::Room, Vec3),
}

#[derive(Default)]
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, move_player::<Cube>);
    }
}

fn move_player<P: PlatonicSolid>(
    mut player_query: Query<(&mut Transform, &PlayerMazeState<P>)>,
    settings: Res<GameSettings>,
) {
    let (mut player_transform, player_maze_state) = player_query.single_mut();

    let target_position = match player_maze_state {
        PlayerMazeState::<P>::Node(node) => {
            node.position() + settings.player_elevation * node.face().normal()
        }
        PlayerMazeState::<P>::Edge(_, _, edge_position) => edge_position.clone(),
    };

    player_transform.translation = player_transform.translation.lerp(target_position, 0.1)
}
