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
use model::{BorderType, CubeGenerator, CubeNode};

mod cube_maze_factory;
mod model;
fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            #[cfg(not(target_arch = "wasm32"))]
            WireframePlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                do_input,
                #[cfg(not(target_arch = "wasm32"))]
                toggle_wireframe,
            ),
        )
        .run();
}

/// A marker component for our shapes so we can query them separately from the ground plane
#[derive(Component)]
enum Interactable {
    Node,
    Edge,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let white = Color::srgb_u8(247, 247, 0);
    let beige = Color::srgb_u8(242, 231, 213);
    let green = Color::srgb_u8(109, 152, 134);
    let charcoal = Color::srgb_u8(57, 62, 70);

    let white_material = materials.add(StandardMaterial::from_color(white));
    let beige_material = materials.add(StandardMaterial::from_color(beige));
    let green_material = materials.add(StandardMaterial::from_color(green));
    let charcoal_material = materials.add(StandardMaterial::from_color(charcoal));

    let cube_gen = CubeGenerator::new(3, 2.0);
    let maze = cube_gen.make_maze();

    let connection_height = 0.04;

    for node in maze.graph.nodes() {
        let cylinder = Cylinder::new(0.09, connection_height + 0.005);

        let cuboid_mesh = meshes.add(cylinder);

        let face_normal = node.face.normal();

        let transform = Transform::IDENTITY
            .looking_at(face_normal.any_orthogonal_vector(), face_normal)
            .with_translation(node.position + 0.5 * connection_height * face_normal);

        let material = if *maze.solution.first().unwrap() == node {
            white_material.clone()
        } else if *maze.solution.last().unwrap() == node {
            white_material.clone()
        } else {
            beige_material.clone()
        };

        commands
            .spawn(PbrBundle {
                mesh: cuboid_mesh,
                material,
                transform,
                ..default()
            })
            .insert(Interactable::Node);
    }

    for (source_node, target_node, edge) in maze.graph.all_edges() {
        let mesh = get_connection_mesh(
            source_node,
            target_node,
            cube_gen.distance_between_nodes(),
            connection_height,
        );
        let connecting_mesh = meshes.add(mesh);

        let transform = get_connection_transform(source_node, target_node, connection_height);
        commands
            .spawn(PbrBundle {
                mesh: connecting_mesh,
                material: beige_material.clone(),
                transform,
                ..default()
            })
            .insert(Interactable::Edge);
    }

    let cuboid = meshes.add(Cuboid::from_length(1.5));
    commands.spawn(PbrBundle {
        mesh: cuboid,
        material: green_material.clone(),
        transform: Transform::IDENTITY,
        ..default()
    });

    commands.spawn(Camera3dBundle {
        camera: Camera {
            // clear the whole viewport with the given color
            clear_color: ClearColorConfig::Custom(charcoal),
            ..Default::default()
        },
        transform: Transform::from_xyz(0.0, 0.0, 3.0).looking_at(Vec3::new(0., 0., 0.), Vec3::Y),
        ..default()
    });

    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_xyz(0.0, 0.0, 5.0).looking_at(Vec3::new(0., 0., 0.), Vec3::Y),
        ..default()
    });
}

fn get_connection_mesh(
    from: CubeNode,
    to: CubeNode,
    distance_between_nodes: f32,
    connection_height: f32,
) -> Mesh {
    let border_type = BorderType::get_from_faces(&from.face, &to.face);

    let width = 0.06;
    let node_size = 0.1;

    match border_type {
        BorderType::SameFace => {
            let length = (distance_between_nodes - node_size);
            let half_width = width / 2.0;
            let cube = Cuboid::new(width, connection_height, length).into();
            cube
        }
        BorderType::Connected => {
            create_edge_piece(width, connection_height, distance_between_nodes, node_size)
        }
        _ => panic!["stop"],
    }
}

fn create_edge_piece(width: f32, height: f32, distance_between_nodes: f32, node_size: f32) -> Mesh {
    let half_width = width / 2.0;
    let half_distance_to_node = (distance_between_nodes - node_size) / 2.0;
    let uv_mid_point = height / height + half_distance_to_node;

    let vertices = vec![
        //L +Z face
        [0.0, 0.0, half_width],
        [height, height, half_width],
        [-half_distance_to_node, 0.0, half_width],
        [-half_distance_to_node, height, half_width],
        [0.0, -half_distance_to_node, half_width],
        [height, -half_distance_to_node, half_width],
        //L -Z face
        [0.0, 0.0, -half_width],
        [height, height, -half_width],
        [-half_distance_to_node, 0.0, -half_width],
        [-half_distance_to_node, height, -half_width],
        [0.0, -half_distance_to_node, -half_width],
        [height, -half_distance_to_node, -half_width],
        //X normal face
        [height, height, half_width],
        [height, height, -half_width],
        [height, -half_distance_to_node, -half_width],
        [height, -half_distance_to_node, half_width],
        //Y normal face
        [height, height, half_width],
        [height, height, -half_width],
        [-half_distance_to_node, height, -half_width],
        [-half_distance_to_node, height, half_width],
    ]
    .into_iter()
    .map(|arr| Vec3::from_array(arr))
    .collect::<Vec<Vec3>>();

    let total_corner_piece_length = height + half_distance_to_node;
    let uv_coords = vertices
        .clone()
        .into_iter()
        .map(|vec| (vec + half_distance_to_node) / total_corner_piece_length)
        .map(|vec| Vec2::new(vec.x, vec.y))
        .collect::<Vec<Vec2>>();

    let normals = vertices
        .clone()
        .into_iter()
        .enumerate()
        .map(|(i, _)| match i {
            0..6 => Vec3::Z,
            6..12 => -Vec3::Z,
            12..16 => Vec3::X,
            16..20 => Vec3::Y,
            _ => panic!["stop"],
        })
        .collect::<Vec<Vec3>>();

    // Create a new mesh using a triangle list topology, where each set of 3 vertices composes a triangle.
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uv_coords)
    // Assign normals (everything points outwards)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_indices(Indices::U32(vec![
        0, 1, 3, 0, 5, 1, 0, 3, 2, 0, 4, 5, // +Z face
        6, 9, 7, 6, 7, 11, 6, 8, 9, 6, 11, 10, // -Z face
        12, 14, 13, 12, 15, 14, // X face
        16, 17, 18, 16, 18, 19, // Y face
    ]))
}

fn get_connection_transform(from: CubeNode, to: CubeNode, connection_height: f32) -> Transform {
    let border_type = BorderType::get_from_faces(&from.face, &to.face);
    match border_type {
        BorderType::SameFace => {
            let forward = from.position - to.position;
            let middle = (from.position + to.position) / 2.0;
            Transform::IDENTITY
                .looking_to(forward, from.face.normal())
                .with_translation(middle + from.face.normal() * connection_height / 2.0)
        }
        BorderType::Connected => {
            let forward = from.face.normal().cross(to.face.normal());
            let translation = from.position.abs().max(to.position.abs()) * from.position.signum();
            Transform::IDENTITY
                .looking_to(forward, from.face.normal())
                .with_translation(translation)
        }
        _ => panic!["unknown edge types"],
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn toggle_wireframe(
    mut wireframe_config: ResMut<WireframeConfig>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        wireframe_config.global = !wireframe_config.global;
    }
}

fn do_input(
    mut camera_query: Query<(&mut Transform, &GlobalTransform, &Camera), Without<Interactable>>,
    mut light_query: Query<
        &mut Transform,
        (
            With<DirectionalLight>,
            Without<Interactable>,
            Without<Camera>,
        ),
    >,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    nodes: Query<
        (&Transform, &Handle<Mesh>),
        (
            With<Interactable>,
            Without<Camera>,
            Without<DirectionalLight>,
        ),
    >,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut last_pos: Local<Option<Vec2>>,
) {
    let Ok(window) = primary_window.get_single() else {
        return;
    };
    let window_size = window.size();

    let Some(cursor_position) = window.cursor_position() else {
        // if the cursor is not inside the window, we can't do anything
        return;
    };

    let cursor_position_vec = vec2(cursor_position.x, cursor_position.y);

    let (mut camera_transform, camera_global_transform, camera) = camera_query.single_mut();

    let Some(ray) = camera.viewport_to_world(camera_global_transform, cursor_position) else {
        // if it was impossible to compute for whatever reason; we can't do anything
        return;
    };

    let delta_device_pixels = cursor_position_vec - last_pos.unwrap_or(cursor_position_vec);

    let delta = if mouse_buttons.pressed(MouseButton::Left)
        && !mouse_buttons.just_pressed(MouseButton::Left)
    {
        delta_device_pixels
    } else {
        Vec2::ZERO
    };

    let delta = camera_transform.right() * delta.x + camera_transform.up() * delta.y;
    let axis = delta
        .cross(camera_transform.forward().as_vec3())
        .normalize();

    if axis.norm() > 0.01 {
        // println!("rotate_around: {:?} with delta: {:?}", axis, delta);
        let rotation = Quat::from_axis_angle(axis, delta.norm() / 150.0);

        let mut light_transform = light_query.single_mut();

        light_transform.rotate_around(Vec3::new(0.0, 0.0, 0.0), rotation);
        camera_transform.rotate_around(Vec3::new(0.0, 0.0, 0.0), rotation);
    }
    *last_pos = Some(cursor_position_vec);
}
