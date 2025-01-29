use bevy::prelude::*;

use crate::{
    assets::material_handles::MaterialHandles,
    game_settings::GameSettings,
    is_room_junction::is_junction,
    levels::LevelData,
    player::PlayerMazeState,
    room::Room,
    shape::loader::{GraphComponent, SolutionComponent},
};

#[derive(Resource)]
pub struct NodeArrivalEffectMesh(Handle<Mesh>);

#[derive(Component)]
pub struct NodeArrivalEffectInstance {
    lifetime: f32,
    birth_time: f32,
}

pub fn setup_node_arrival_particle(mut meshes: ResMut<Assets<Mesh>>, mut commands: Commands) {
    let circle = Circle::new(0.1);
    let mesh_handle = meshes.add(Mesh::from(circle));

    commands.insert_resource(NodeArrivalEffectMesh(mesh_handle));
}

pub fn spawn_node_arrival_particles(
    mut commands: Commands,
    node_arrival_mesh: Res<NodeArrivalEffectMesh>,
    player_maze_state: Query<&PlayerMazeState>,
    graph_component: Query<&GraphComponent>,
    solution_component_query: Query<(&SolutionComponent)>,
    mut last_room_local: Local<Option<Room>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    game_materials: Res<MaterialHandles>,
    settings: Res<GameSettings>,
    time: Res<Time>,
) {
    let Ok(GraphComponent(graph)) = graph_component.get_single() else {
        return;
    };

    let Ok(PlayerMazeState::Node(room)) = player_maze_state.get_single() else {
        return;
    };

    let Ok(SolutionComponent(rooms)) = solution_component_query.get_single() else {
        return;
    };

    let last_room = last_room_local.unwrap_or(*room);

    *last_room_local = Some(*room);

    if *room == last_room || !is_junction(&room, &graph) {
        return;
    }

    let is_goal_node = rooms.last().unwrap() == room;

    let effect_color = if is_goal_node {
        settings.palette.player_color.clone().with_alpha(0.99)
    } else {
        settings.palette.line_color.clone().with_alpha(0.99)
    };

    let material_handle = materials.add(effect_color);

    let NodeArrivalEffectMesh(mesh_handle) = node_arrival_mesh.into_inner();

    let position = room.position();
    let normal = room.face().normal();
    let forward_direction = normal.any_orthogonal_vector();

    commands
        .spawn(PbrBundle {
            mesh: Mesh3d(mesh_handle.clone()),
            material: MeshMaterial3d(material_handle.clone()),
            transform: Transform::IDENTITY
                .looking_to(-normal, forward_direction)
                .with_translation(position + normal * 0.02),

            ..default()
        })
        .insert(LevelData)
        .insert(NodeArrivalEffectInstance {
            lifetime: 1.,
            birth_time: time.elapsed_secs(),
        });
}

pub fn update_node_arrival_particles(
    mut node_arrival_particles: Query<(
        Entity,
        &mut Transform,
        &NodeArrivalEffectInstance,
        &MeshMaterial3d<StandardMaterial>,
    )>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
    time: Res<Time>,
) {
    for (
        entity,
        mut transform,
        NodeArrivalEffectInstance {
            lifetime,
            birth_time,
        },
        MeshMaterial3d::<StandardMaterial>(material_handle),
    ) in node_arrival_particles.iter_mut()
    {
        let age = time.elapsed_secs() - birth_time;
        if age > *lifetime {
            materials.remove(material_handle);
            commands.entity(entity).despawn();
            return;
        }

        let decay_factor = (-age * 3.0).exp();
        transform.scale = Vec3::ONE * (1.0 - decay_factor) * 3.5;

        let Some(material) = materials.get_mut(material_handle) else {
            return;
        };

        material.base_color.set_alpha(decay_factor);
    }
}
