#[cfg(not(target_arch = "wasm32"))]
use bevy::pbr::wireframe::WireframePlugin;
use bevy::prelude::*;
use bevy_rapier3d::plugin::{NoUserData, RapierPhysicsPlugin};
use bevy_vector_shapes::ShapePlugin;
use camera::PlatonicCamera;
use controller::Controller;
use game_settings::GameSettingsPlugin;
use maze_generator::config::Maze;
use player::PlayerPlugin;
use shape::cube::{
    self,
    maze::{Cube, CubeEdge, CubeMaze, PlatonicSolid},
};

mod camera;
mod controller;
mod game_settings;
mod player;
mod shape;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            #[cfg(not(target_arch = "wasm32"))]
            WireframePlugin,
        ))
        .add_plugins(GameSettingsPlugin::default())
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(Controller::default())
        .add_plugins(PlatonicCamera::default())
        .add_plugins(PlayerPlugin::default())
        .add_plugins(ShapePlugin::default())
        .add_systems(Startup, (load_maze, cube::spawn::<Cube>.after(load_maze)))
        .run();
}

#[derive(Resource)]
pub struct Level<P: PlatonicSolid> {
    pub platonic_solid: P,
    pub maze: Maze<P::Room, CubeEdge>,
}

fn load_maze(mut commands: Commands) {
    let platonic_solid = Cube::new(3, 2.0);
    let CubeMaze(maze) = platonic_solid.build();
    commands.insert_resource(Level::<Cube> {
        maze,
        platonic_solid,
    });
}
