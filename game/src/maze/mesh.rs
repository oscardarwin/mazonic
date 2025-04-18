use bevy::{
    math::NormedVectorSpace,
    pbr::ExtendedMaterial,
    prelude::*,
    utils::{HashMap, HashSet},
};
use bevy_hanabi::prelude::*;
use rand::{seq::IteratorRandom, SeedableRng};
use rand_chacha::ChaCha8Rng;

use crate::{
    assets::{
        material_handles::MaterialHandles,
        mesh_handles::MeshHandles,
        shaders::{DashedArrowShader, PulsingShader},
    },
    effects::musical_notes::{MusicalNoteEffectHandle, MusicalNoteImageHandles, MusicalNoteMarker},
    game_save::{CurrentLevelIndex, DiscoveredMelodies, DiscoveredMelody},
    game_systems::SystemHandles,
    is_room_junction::is_junction,
    levels::{GameLevel, LevelData, Shape},
    maze::maze_mesh_builder::MazeMeshBuilder,
    room::Room,
    shape::loader::{GraphComponent, SolutionComponent},
};

use super::border_type::BorderType;

const ROOM_HEIGHT: f32 = 0.002;
const SAME_FACE_EDGE_HEIGHT: f32 = 0.001;
const CROSS_FACE_EDGE_HEIGHT: f32 = 0.001;

#[derive(Component, Debug, Clone)]
pub struct MazeMarker;

pub fn spawn(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    level_query: Query<&GameLevel>,
    maze_query: Query<(&GraphComponent, &SolutionComponent)>,
    mesh_handles: Res<MeshHandles>,
    material_handles: Res<MaterialHandles>,
    discovered_melodies_query: Query<&DiscoveredMelodies>,
    current_level_index_query: Query<&CurrentLevelIndex>,
    musical_note_effect_handle: Query<&MusicalNoteEffectHandle>,
    musical_note_image_handle_query: Query<&MusicalNoteImageHandles>,
) {
    let Ok(level) = level_query.get_single() else {
        return;
    };

    let Ok((GraphComponent(graph), SolutionComponent(solution))) = maze_query.get_single() else {
        return;
    };

    let Ok(discovered_melodies) = discovered_melodies_query.get_single() else {
        return;
    };

    let Ok(CurrentLevelIndex(current_level_index)) = current_level_index_query.get_single() else {
        return;
    };

    let Ok(MusicalNoteEffectHandle { effect_handles }) = musical_note_effect_handle.get_single()
    else {
        return;
    };

    let Ok(MusicalNoteImageHandles {
        crotchet_handle,
        quaver_handle,
    }) = musical_note_image_handle_query.get_single()
    else {
        return;
    };

    let discovered_melody_rooms = discovered_melodies.get_room_ids_for_level(*current_level_index);

    let distance_between_nodes = level.node_distance();

    let goal_node = solution.last().unwrap();
    for room in graph.nodes().filter(|room| is_junction(room, &graph)) {
        let is_goal_node = room == *goal_node;
        let is_discovered_melody_room = discovered_melody_rooms.contains(&room.id);

        let transform = Transform::IDENTITY
            .looking_at(
                -room.face().normal(),
                room.face().normal().any_orthogonal_vector(),
            )
            .with_translation(room.position() + room.face().normal() * ROOM_HEIGHT);

        let mut entity_commands = commands.spawn((transform, LevelData, room));

        if is_discovered_melody_room {
            entity_commands.insert(MusicalNoteMarker);

            //let num_effect_handles = effect_handles.len();

            //let crotchet_effect_handle_index = room.id as usize % num_effect_handles;
            //let quaver_effect_handle_index =
            //    (room.id as usize + num_effect_handles / 2) as usize % num_effect_handles;

            //entity_commands.with_children(|parent| {
            //    parent
            //        .spawn(ParticleEffectBundle {
            //            effect: ParticleEffect::new(
            //                effect_handles[crotchet_effect_handle_index].clone(),
            //            ),
            //            transform: Transform::IDENTITY,
            //            ..Default::default()
            //        })
            //        .insert(EffectMaterial {
            //            images: vec![crotchet_handle.clone()],
            //        });

            //    parent
            //        .spawn(ParticleEffectBundle {
            //            effect: ParticleEffect::new(
            //                effect_handles[quaver_effect_handle_index].clone(),
            //            ),
            //            transform: Transform::IDENTITY,
            //            ..Default::default()
            //        })
            //        .insert(EffectMaterial {
            //            images: vec![quaver_handle.clone()],
            //        });
            //});
        }

        let mesh_handle = if room == *goal_node {
            mesh_handles.goal_room.clone()
        } else {
            mesh_handles.junction_room.clone()
        };

        entity_commands.with_children(|parent| {
            let mut child_entity_commands = parent.spawn((
                Mesh3d(mesh_handle),
                Transform::IDENTITY.with_scale(Vec3::splat(distance_between_nodes)),
                MazeMarker,
            ));

            let material_handle = match (is_goal_node, is_discovered_melody_room) {
                (true, _) => child_entity_commands
                    .insert(MeshMaterial3d(material_handles.goal_handle.clone())),
                (false, true) => child_entity_commands.insert(MeshMaterial3d(
                    material_handles.bright_pulsing_line_handle.clone(),
                )),
                (false, false) => child_entity_commands
                    .insert(MeshMaterial3d(material_handles.line_handle.clone())),
            };
        });
    }

    let discovered_melody_room_pairs =
        make_room_pairs_from_discovered_melodies(*current_level_index, &discovered_melodies.0);

    let maze_mesh_builder = match &level.shape {
        Shape::Tetrahedron => MazeMeshBuilder::tetrahedron(),
        Shape::Cube => MazeMeshBuilder::cube(),
        Shape::Octahedron => MazeMeshBuilder::octahedron(),
        Shape::Dodecahedron => MazeMeshBuilder::dodecahedron(),
        Shape::Icosahedron => MazeMeshBuilder::icosahedron(1.0),
    };

    let same_face_edge = meshes.add(maze_mesh_builder.same_face_edge());
    let one_way_same_face_edge = meshes.add(maze_mesh_builder.one_way_same_face_edge());
    let cross_face_edge = meshes.add(maze_mesh_builder.cross_face_edge());
    let one_way_cross_face_edge = meshes.add(maze_mesh_builder.one_way_cross_face_edge());

    for (source_node, target_node, _) in graph.all_edges() {
        let bidirectional = graph.contains_edge(target_node, source_node);

        if bidirectional && source_node.cmp(&target_node).is_lt() {
            continue;
        }

        let Some(border_type) = level.border_type(&source_node.face(), &target_node.face()) else {
            panic!["unknown edge type"];
        };

        let mesh_handle = match (&border_type, bidirectional) {
            (BorderType::SameFace, true) => same_face_edge.clone(),
            (BorderType::SameFace, false) => one_way_same_face_edge.clone(),
            (BorderType::Connected, true) => cross_face_edge.clone(),
            (BorderType::Connected, false) => one_way_cross_face_edge.clone(),
        };

        let transform = get_connection_transform(source_node, target_node, &border_type);

        let is_discovered = discovered_melody_room_pairs
            .contains(&(source_node.id, target_node.id))
            || discovered_melody_room_pairs.contains(&(target_node.id, source_node.id));

        let mut entity_commands = commands
            .spawn((transform.clone(), LevelData))
            .with_children(|parent| {
                let mut entity_commands = parent.spawn((
                    Mesh3d(mesh_handle),
                    Transform::IDENTITY.with_scale(Vec3::splat(distance_between_nodes)),
                    MazeMarker,
                ));

                match (bidirectional, is_discovered) {
                    (false, true) => entity_commands.insert(MeshMaterial3d(
                        material_handles.bright_pulsing_dashed_arrow_handle.clone(),
                    )),
                    (false, false) => entity_commands
                        .insert(MeshMaterial3d(material_handles.dashed_arrow_handle.clone())),
                    (true, true) => entity_commands.insert(MeshMaterial3d(
                        material_handles.bright_pulsing_line_handle.clone(),
                    )),
                    (true, false) => {
                        entity_commands.insert(MeshMaterial3d(material_handles.line_handle.clone()))
                    }
                };
            });
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

pub fn update_on_melody_discovered(
    mut commands: Commands,
    system_handles: Res<SystemHandles>,
    maze_entities_query: Query<Entity, With<MazeMarker>>,
) {
    commands.run_system(system_handles.spawn_maze);
    for maze_entity in maze_entities_query.iter() {
        commands.entity(maze_entity).despawn();
    }
}
