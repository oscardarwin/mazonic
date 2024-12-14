use std::f32::consts::PI;

#[cfg(not(target_arch = "wasm32"))]
use bevy::pbr::wireframe::{WireframeConfig, WireframePlugin};
use bevy::{
    color::palettes::basic::SILVER,
    math::{vec2, NormedVectorSpace},
    prelude::*,
    render::{
        mesh::{Indices, PrimitiveTopology},
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
    },
    window::PrimaryWindow,
};
use bevy_rapier3d::{
    geometry::Collider,
    pipeline::QueryFilter,
    plugin::{NoUserData, RapierContext, RapierPhysicsPlugin},
};
use camera::PlatonicCamera;
use controller::Controller;
use player::setup_player;
use shape::cube::{
    self,
    maze::{CubeMaze, CubeNode, Edge, Face},
};

mod camera;
mod controller;
mod model;
mod player;
mod shape;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            #[cfg(not(target_arch = "wasm32"))]
            WireframePlugin,
        ))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(Controller::default())
        .add_plugins(PlatonicCamera::default())
        .add_systems(
            Startup,
            (
                load_maze,
                setup_player.after(load_maze),
                cube::spawn.after(load_maze),
            ),
        )
        .run();
}

fn load_maze(mut commands: Commands) {
    let maze = CubeMaze::build(3, 2.0, 0.2);
    commands.insert_resource(maze);
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    cube_maze: Res<CubeMaze>,
) {
    let white = Color::srgb_u8(247, 247, 0);
    let beige = Color::srgb_u8(242, 231, 213);
    let green = Color::srgb_u8(109, 152, 134);
    let charcoal = Color::srgb_u8(57, 62, 70);

    let white_material = materials.add(StandardMaterial::from_color(white));
    let beige_material = materials.add(StandardMaterial::from_color(beige));
    let green_material = materials.add(StandardMaterial::from_color(green));
    let charcoal_material = materials.add(StandardMaterial::from_color(charcoal));
}
