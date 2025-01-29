use bevy::{
    math::NormedVectorSpace,
    prelude::*,
    utils::{HashMap, HashSet},
};

use crate::{
    assets::material_handles::MaterialHandles,
    game_save::{CurrentLevelIndex, DiscoveredMelodies, DiscoveredMelody},
    is_room_junction::is_junction,
    levels::{GameLevel, LevelData},
    maze::maze_mesh_builder::MazeMeshBuilder,
    room::Room,
    shape::loader::{GraphComponent, SolutionComponent},
};

use super::border_type::BorderType;

const ROOM_HEIGHT: f32 = 0.002;
const SAME_FACE_EDGE_HEIGHT: f32 = 0.001;
const CROSS_FACE_EDGE_HEIGHT: f32 = 0.0005;

pub fn spawn(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: Res<Assets<StandardMaterial>>,
    level_query: Query<(&MazeMeshBuilder, &GameLevel)>,
    maze_query: Query<(&GraphComponent, &SolutionComponent)>,
    material_handles: Res<MaterialHandles>,
    discovered_melodies_query: Query<&DiscoveredMelodies>,
    current_level_index_query: Query<&CurrentLevelIndex>,
) {
    let Ok((mesh_builder, level)) = level_query.get_single() else {
        return;
    };

    let Ok((GraphComponent(graph), SolutionComponent(solution))) = maze_query.get_single() else {
        return;
    };

    let Ok(DiscoveredMelodies(discovered_melodies)) = discovered_melodies_query.get_single() else {
        return;
    };

    let Ok(CurrentLevelIndex(current_level_index)) = current_level_index_query.get_single() else {
        return;
    };

    let discovered_melody_rooms =
        make_discovered_melody_room_set(*current_level_index, discovered_melodies);

    let room_mesh_handle = meshes.add(mesh_builder.intersection_room_mesh());
    let goal_mesh_handle = meshes.add(mesh_builder.goal_mesh());

    let goal_node = solution.last().unwrap();
    for node in graph.nodes().filter(|room| is_junction(room, &graph)) {
        let is_goal_node = node == *goal_node;
        let is_discovered_melody_room = discovered_melody_rooms.contains(&node.id);

        let transform = Transform::IDENTITY
            .looking_at(
                -node.face().normal(),
                node.face().normal().any_orthogonal_vector(),
            )
            .with_translation(node.position() + node.face().normal() * ROOM_HEIGHT);

        let mesh_handle = if node == *goal_node {
            goal_mesh_handle.clone()
        } else {
            room_mesh_handle.clone()
        };

        let mut entity_commands = commands.spawn((Mesh3d(mesh_handle), transform, LevelData));
        let material_handle = match (is_goal_node, is_discovered_melody_room) {
            (true, _) => {
                entity_commands.insert(MeshMaterial3d(material_handles.goal_handle.clone()))
            }
            (false, true) => {
                entity_commands.insert(MeshMaterial3d(material_handles.bright_line_handle.clone()))
            }
            (false, false) => {
                entity_commands.insert(MeshMaterial3d(material_handles.line_handle.clone()))
            }
        };
    }

    let discovered_melody_room_pairs =
        make_room_pairs_from_discovered_melodies(*current_level_index, discovered_melodies);

    let edge_mesh = meshes.add(mesh_builder.edge());
    let one_way_edge_mesh = meshes.add(mesh_builder.one_way_edge());
    let cross_face_edge_mesh = meshes.add(mesh_builder.cross_face_edge());
    let cross_face_one_way_edge_mesh = meshes.add(mesh_builder.cross_face_one_way_edge());

    for (source_node, target_node, _) in graph.all_edges() {
        let bidirectional = graph.contains_edge(target_node, source_node);

        if bidirectional && source_node.cmp(&target_node).is_lt() {
            continue;
        }

        let Some(border_type) = level.border_type(&source_node.face(), &target_node.face()) else {
            panic!["unknown edge type"];
        };

        let mesh_handle = match (&border_type, bidirectional) {
            (BorderType::SameFace, true) => edge_mesh.clone(),
            (BorderType::SameFace, false) => one_way_edge_mesh.clone(),
            (BorderType::Connected, true) => cross_face_edge_mesh.clone(),
            (BorderType::Connected, false) => cross_face_one_way_edge_mesh.clone(),
        };

        let transform = get_connection_transform(source_node, target_node, &border_type);

        let is_discovered = discovered_melody_room_pairs
            .contains(&(source_node.id, target_node.id))
            || discovered_melody_room_pairs.contains(&(target_node.id, source_node.id));

        let mut entity_commands =
            commands.spawn((Mesh3d(mesh_handle), transform.clone(), LevelData));

        match (bidirectional, is_discovered) {
            (false, true) => entity_commands.insert(MeshMaterial3d(
                material_handles.bright_dashed_arrow_handle.clone(),
            )),
            (false, false) => {
                entity_commands.insert(MeshMaterial3d(material_handles.dashed_arrow_handle.clone()))
            }
            (true, true) => {
                entity_commands.insert(MeshMaterial3d(material_handles.bright_line_handle.clone()))
            }
            (true, false) => {
                entity_commands.insert(MeshMaterial3d(material_handles.line_handle.clone()))
            }
        };
    }
}

fn get_connection_transform(from: Room, to: Room, border_type: &BorderType) -> Transform {
    match border_type {
        BorderType::SameFace => {
            let forward = from.position() - to.position();
            Transform::IDENTITY
                .looking_to(forward, from.face().normal())
                .with_translation(from.position() + from.face().normal() * SAME_FACE_EDGE_HEIGHT)
        }
        BorderType::Connected => get_cross_face_edge_transform(
            from.position(),
            from.face().normal(),
            to.position(),
            to.face().normal(),
        ),
    }
}

pub fn get_cross_face_edge_transform(
    from_position: Vec3,
    from_normal: Vec3,
    to_position: Vec3,
    to_normal: Vec3,
) -> Transform {
    let half_angle = from_normal.angle_between(to_normal) / 2.0;

    let average_normal = from_normal.lerp(to_normal, 0.5).normalize();

    let edge_vec = to_position - from_position;

    let intersection_point =
        from_position + (edge_vec + edge_vec.norm() * half_angle.tan() * average_normal) / 2.0;

    Transform::IDENTITY
        .looking_to(intersection_point - to_position, to_normal)
        .with_translation(intersection_point + average_normal * CROSS_FACE_EDGE_HEIGHT)
}

pub fn make_discovered_melody_room_set(
    current_level_index: usize,
    discovered_melodies: &HashMap<usize, DiscoveredMelody>,
) -> HashSet<u64> {
    if let Some(DiscoveredMelody { room_ids, .. }) = discovered_melodies.get(&current_level_index) {
        room_ids.iter().cloned().collect()
    } else {
        HashSet::new()
    }
}

pub fn make_room_pairs_from_discovered_melodies(
    current_level_index: usize,
    discovered_melodies: &HashMap<usize, DiscoveredMelody>,
) -> HashSet<(u64, u64)> {
    let mut room_pairs = HashSet::new();

    let Some(DiscoveredMelody { room_ids, .. }) = discovered_melodies.get(&current_level_index)
    else {
        return room_pairs;
    };

    for (from, to) in room_ids.iter().zip(room_ids.iter().skip(1)) {
        room_pairs.insert((*from, *to));
    }

    room_pairs
}
