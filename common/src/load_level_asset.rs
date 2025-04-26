use bevy::prelude::*;
use bevy::tasks::IoTaskPool;
use bevy::tasks::Task;
use bevy::utils::HashMap;

use crate::game_save::CurrentPuzzle;
use crate::game_save::DailyLevelId;
use crate::game_save::LevelIndex;
use crate::game_save::PuzzleIdentifier;
use crate::game_state::PlayState;
use crate::levels::LevelData;
use crate::shape::loader::MazeLevelData;

#[derive(Debug)]
pub enum DailyLevelLoadError {
    StringParseError(std::io::Error),
    JsonParseError(serde_json::Error),
    HttpError(ureq::Error),
}

#[derive(Component)]
pub enum MazeSaveDataHandle {
    LocalLevel(Handle<MazeLevelData>),
    RemoteLevel(Task<Result<MazeLevelData, DailyLevelLoadError>>),
}

#[derive(Resource, Default)]
pub struct LoadedLevels(pub HashMap<PuzzleIdentifier, MazeSaveDataHandle>);

const EASY_DAILY_LEVEL_TAG: &str = "easy";
const HARD_DAILY_LEVEL_TAG: &str = "hard";
const DAILY_LEVELS_URL: &str = "https://raw.githubusercontent.com/oscardarwin/mazonic_levels/main";

pub fn setup(mut commands: Commands) {
    commands.init_resource::<LoadedLevels>();
}

fn local_remote_daily_level(daily_level_id: &DailyLevelId, tag: &str) -> MazeSaveDataHandle {
    let thread_pool = IoTaskPool::get();
    let url = format!("{DAILY_LEVELS_URL}/{tag}/{daily_level_id}.json");

    let task = thread_pool.spawn(async move {
        let res = ureq::get(&url).call().map_err(|e| DailyLevelLoadError::HttpError(e))?;
        let body = res.into_string().map_err(|e| DailyLevelLoadError::StringParseError(e))?;
        let parsed: MazeLevelData = serde_json::from_str(&body).map_err(|e| DailyLevelLoadError::JsonParseError(e))?;
        Ok(parsed)
    });
    
    MazeSaveDataHandle::RemoteLevel(task)
}

fn load_local_level(level_index: LevelIndex, asset_server: Res<AssetServer>) -> MazeSaveDataHandle {
    let file_path = format!("levels/{}.json", level_index);
    let maze_save_data_handle = asset_server.load::<MazeLevelData>(file_path);
    MazeSaveDataHandle::LocalLevel(maze_save_data_handle)
}

pub fn load_(
    current_level_index_query: Query<&CurrentPuzzle>,
    mut game_state: ResMut<NextState<PlayState>>,
    asset_server: Res<AssetServer>,
    mut loaded_levels: ResMut<LoadedLevels>,
) {
    let CurrentPuzzle(puzzle_identifier) = current_level_index_query.single();
    
    if loaded_levels.0.contains_key(puzzle_identifier) {
        return;
    }

    let maze_save_data_handle = match puzzle_identifier {
        PuzzleIdentifier::Level(index) => load_local_level(*index, asset_server),
        PuzzleIdentifier::EasyDaily(id) => local_remote_daily_level(&id, EASY_DAILY_LEVEL_TAG),
        PuzzleIdentifier::HardDaily(id) => local_remote_daily_level(&id, HARD_DAILY_LEVEL_TAG),
    };

    loaded_levels.0.insert(puzzle_identifier.clone(), maze_save_data_handle);
}
