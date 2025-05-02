use std::collections::VecDeque;
use std::time::Duration;

use bevy::prelude::*;
use bevy::tasks::block_on;
use bevy::tasks::futures_lite::future;
use bevy::tasks::IoTaskPool;
use bevy::tasks::Task;
use bevy::utils::HashMap;
use bevy_rustysynth::MidiAudio;
use bevy_rustysynth::MidiNote;

use crate::game_save::CurrentPuzzle;
use crate::game_save::DailyLevelId;
use crate::game_save::LevelIndex;
use crate::game_save::PuzzleIdentifier;
use crate::game_state::GameState;
use crate::game_state::PuzzleState;
use crate::levels::GameLevel;
use crate::levels::PuzzleEntityMarker;
use crate::shape::loader::EncryptedMelody;
use crate::shape::loader::GraphComponent;
use crate::shape::loader::MazeLevelData;
use crate::shape::loader::SolutionComponent;
use crate::sound::MelodyPuzzleTracker;
use crate::sound::Note;
use crate::sound::NoteMapping;
use crate::ui::message::MessagePopup;
use crate::ui::message::MessagePopupUpperMarker;

#[derive(Debug)]
pub enum DailyLevelLoadError {
    StringParseError(std::io::Error),
    JsonParseError(serde_json::Error),
    HttpError(ureq::Error),
}

#[derive(Component)]
pub enum MazeSaveDataHandle {
    LocalLevel(Handle<MazeLevelData>),
    LoadedRemoteLevel(MazeLevelData),
}

#[derive(Resource, Default)]
pub struct LoadingRemoteLevels(pub HashMap<PuzzleIdentifier, Task<Result<MazeLevelData, DailyLevelLoadError>>>);

#[derive(Resource, Default)]
pub struct LoadedLevels(pub HashMap<PuzzleIdentifier, MazeSaveDataHandle>);

const EASY_DAILY_LEVEL_TAG: &str = "easy";
const HARD_DAILY_LEVEL_TAG: &str = "hard";
const DAILY_LEVELS_URL: &str = "https://raw.githubusercontent.com/oscardarwin/mazonic_levels/main";

pub fn setup(mut commands: Commands) {
    commands.init_resource::<LoadedLevels>();
    commands.init_resource::<LoadingRemoteLevels>();
}

fn start_remote_daily_level_download(daily_level_id: &DailyLevelId, tag: &str) -> Task<Result<MazeLevelData, DailyLevelLoadError>> {
    let thread_pool = IoTaskPool::get();
    let url = format!("{DAILY_LEVELS_URL}/{tag}/{daily_level_id}.json");

    thread_pool.spawn(async move {
        let res = ureq::get(&url).call().map_err(|e| DailyLevelLoadError::HttpError(e))?;
        let body = res.into_string().map_err(|e| DailyLevelLoadError::StringParseError(e))?;
        let parsed: MazeLevelData = serde_json::from_str(&body).map_err(|e| DailyLevelLoadError::JsonParseError(e))?;
        Ok(parsed)
    })
}

fn load_local_level(level_index: LevelIndex, asset_server: &AssetServer) -> Handle<MazeLevelData> {
    let file_path = format!("levels/{}.json", level_index);
    asset_server.load::<MazeLevelData>(file_path)
}

pub fn wait_until_loaded(
    current_level_index_query: Query<&CurrentPuzzle>,
    mut loaded_levels: ResMut<LoadedLevels>,
    mut loading_remote_levels: ResMut<LoadingRemoteLevels>,
    mut message_popup: Query<&mut MessagePopup, With<MessagePopupUpperMarker>>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    let CurrentPuzzle(puzzle_identifier) = current_level_index_query.single();

    if loaded_levels.0.contains_key(puzzle_identifier) {
        game_state.set(GameState::Puzzle);
        return;
    }

    let task = loading_remote_levels.0.entry(puzzle_identifier.clone()).or_insert_with(||
        match puzzle_identifier {
            PuzzleIdentifier::EasyDaily(id) => start_remote_daily_level_download(&id, EASY_DAILY_LEVEL_TAG),
            PuzzleIdentifier::HardDaily(id) => start_remote_daily_level_download(&id, HARD_DAILY_LEVEL_TAG),
            _ => panic!("Not a remote level")
        }
    );

    let Some(load_result) = block_on(future::poll_once(task)) else {
        return;
    };

    loading_remote_levels.0.remove(puzzle_identifier);
    
    let next_game_state = match load_result {
        Ok(level) => {
            loaded_levels.0.insert(puzzle_identifier.clone(), MazeSaveDataHandle::LoadedRemoteLevel(level));
            GameState::Puzzle
        },
        Err(err) => {
            let message = match err {
                DailyLevelLoadError::JsonParseError(_) => "failed to parse json",
                DailyLevelLoadError::HttpError(_) => "could not fetch level from web",
                DailyLevelLoadError::StringParseError(_) => "failed to parse level data",
            }.to_string();

            message_popup.single_mut().0 = message;

            GameState::Selector
        }
    };

    game_state.set(next_game_state);
}

pub fn spawn_level_data(
    current_level_index_query: Query<&CurrentPuzzle>,
    mut commands: Commands,
    mut play_state: ResMut<NextState<PuzzleState>>,
    mut game_state: ResMut<NextState<GameState>>,
    maze_save_data_assets: Res<Assets<MazeLevelData>>,
    mut loaded_levels: ResMut<LoadedLevels>,
    asset_server: Res<AssetServer>,
) {
    let CurrentPuzzle(puzzle_identifier) = current_level_index_query.single();
    
    println!("Loaded levels: {:?}, trying with pi: {:?}", loaded_levels.0.keys().collect::<Vec<_>>(), puzzle_identifier);

    let maze_save_data_handle = loaded_levels.0.entry(puzzle_identifier.clone()).or_insert_with(||
        match puzzle_identifier {
            PuzzleIdentifier::Level(index) => MazeSaveDataHandle::LocalLevel(load_local_level(*index, &asset_server)),
            _ => panic!("Not a local level")
        }
    );

    let MazeLevelData {
        shape,
        nodes_per_edge,
        face_color_permutation,
        graph,
        solution,
        node_id_to_note,
        encrypted_melody,
    } = match maze_save_data_handle {
        MazeSaveDataHandle::LocalLevel(handle) => match maze_save_data_assets.get(handle) {
            Some(level) => level.clone(),
            None => return,
        },
        MazeSaveDataHandle::LoadedRemoteLevel(level) => level.clone(),
    };

    let note_midi_handle = node_id_to_note
        .into_iter()
        .map(|(node_id, note)| {
            let midi_note = MidiNote {
                key: note.key,
                velocity: note.velocity,
                duration: Duration::from_secs_f32(note.value.as_f32()),
                ..Default::default()
            };
            let audio = MidiAudio::Sequence(vec![midi_note]);
            let audio_handle = asset_server.add::<MidiAudio>(audio);
            (node_id, (audio_handle, note.clone()))
        })
        .collect::<HashMap<u64, (Handle<MidiAudio>, Note)>>();

    if let Some(EncryptedMelody {
        encrypted_melody_bytes,
        melody_length,
    }) = encrypted_melody
    {
        let room_ids = VecDeque::with_capacity(melody_length);
        commands.spawn((
            MelodyPuzzleTracker {
                room_ids,
                encrypted_melody_bytes: encrypted_melody_bytes.clone(),
            },
            PuzzleEntityMarker,
        ));
    }

    commands.spawn((
        PuzzleEntityMarker,
        GameLevel {
            shape,
            nodes_per_edge,
            face_color_permutation,
        },
        GraphComponent(graph),
        SolutionComponent(solution),
        NoteMapping(note_midi_handle),
    ));
    play_state.set(PuzzleState::Playing);
}
