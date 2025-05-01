use bevy::{
    asset::Assets, color::Color, ecs::system::{Commands, ResMut}, math::NormedVectorSpace, pbr::{ExtendedMaterial, PbrBundle, StandardMaterial}, prelude::*, render::mesh::Mesh, tasks::{block_on, futures_lite::future}, transform::components::Transform, utils::{HashMap, HashSet}
};
use bevy_rustysynth::{MidiAudio, MidiNote};

use std::{
    collections::VecDeque, f32::consts::FRAC_PI_2, fs::{self, File}, hash::{DefaultHasher, Hash, Hasher}, time::Duration, usize
};

use petgraph::{graphmap::GraphMap, Directed};

use crate::{
    assets::{
        material_handles::FaceMaterialHandles,
        mesh_generators::{
            FaceMeshGenerator, PentagonFaceMeshGenerator, SquareFaceMeshGenerator,
            TriangleFaceMeshGenerator,
        },
        mesh_handles::MeshHandles,
        shaders::GlobalShader,
    }, constants::{SQRT_3, TAN_27}, game_save::CurrentPuzzle, game_settings::{FaceColorPalette, GameSettings}, game_state::{GameState, PlayState}, is_room_junction::is_junction, levels::{GameLevel, PuzzleEntityMarker, Shape}, load_level_asset::{DailyLevelLoadError, LoadedLevels, MazeSaveDataHandle}, maze::{border_type::BorderType, mesh}, player::{Player, PlayerMazeState}, room::{Edge, Face, Room}, sound::{MelodyPuzzleTracker, Note, NoteMapping}, ui::message::MessagePopup
};

use super::{cube, dodecahedron, icosahedron, octahedron, tetrahedron};
use crate::assets::material_handles::MaterialHandles;

use serde::{Deserialize, Serialize};

#[derive(Component)]
pub struct GraphComponent(pub GraphMap<Room, Edge, Directed>);

#[derive(Component)]
pub struct SolutionComponent(pub Vec<Room>);

#[derive(Serialize, Deserialize, Clone)]
pub struct EncryptedMelody {
    pub encrypted_melody_bytes: Vec<u8>,
    pub melody_length: usize,
}

#[derive(Serialize, Deserialize, Asset, TypePath, Clone)]
pub struct MazeLevelData {
    pub shape: Shape,
    pub nodes_per_edge: u8,
    pub graph: GraphMap<Room, Edge, Directed>,
    pub solution: Vec<Room>,
    pub node_id_to_note: HashMap<u64, Note>,
    pub encrypted_melody: Option<EncryptedMelody>,
}
