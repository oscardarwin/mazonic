use std::path::PathBuf;

use bevy::prelude::*;
use bevy::utils::{HashMap, HashSet};
use bevy_pkv::PkvStore;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::play_statistics::{PlayStatistics, PuzzleStatistics};
use crate::sound::Melody;

pub type LevelIndex = usize;
pub type DailyLevelId = String;

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub enum PuzzleIdentifier {
    Level(LevelIndex),
    EasyDaily(DailyLevelId),
    HardDaily(DailyLevelId),
}

#[derive(Component, Debug, Clone)]
pub struct CurrentPuzzle(pub PuzzleIdentifier);

#[derive(Component, Debug, Clone)]
pub struct WorkingLevelIndex(pub LevelIndex);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredMelody {
    pub melody: Melody,
    pub room_ids: Vec<u64>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GameSave {
    pub current_index: PuzzleIdentifier,
    pub play_statistics: HashMap<PuzzleIdentifier, PuzzleStatistics>,
}

impl GameSave {
    pub fn completed(highest_level: usize) -> Self {
        let play_statistics = (0..highest_level).map(|level_index| (PuzzleIdentifier::Level(level_index), PuzzleStatistics::completed())).collect();
        Self {
            current_index: PuzzleIdentifier::Level(0),
            play_statistics,
        }
    }
}

#[derive(Resource, Clone)]
pub struct SaveLocation(pub PathBuf);

impl Default for GameSave {
    fn default() -> Self {
        GameSave {
            current_index: PuzzleIdentifier::Level(0),
            play_statistics: HashMap::new(),
        }
    }
}

const SAVE_DATA_KEY: &str = "save_data";

pub fn setup(mut commands: Commands, save_location: Option<Res<SaveLocation>>) {
    let pkv_store = match save_location {
        None => PkvStore::new("hallayus", "mazonic"),
        Some(save_location) => PkvStore::new_in_dir(save_location.0.clone()),
    };

    let save_data = match pkv_store.get::<GameSave>(SAVE_DATA_KEY) {
        Ok(game_save) => GameSave::completed(5),
        Err(_) => GameSave::default(),
    };

    let play_statistics = PlayStatistics(save_data.play_statistics);

    commands.spawn((
        CurrentPuzzle(save_data.current_index),
        WorkingLevelIndex(play_statistics.get_working_level()),
    ));

    commands.insert_resource(play_statistics);
    commands.insert_resource(pkv_store);
}

pub fn update(
    current_level_index_query: Query<Ref<CurrentPuzzle>>,
    working_level_index_query: Query<Ref<WorkingLevelIndex>>,
    play_statistics: Res<PlayStatistics>,
    mut pkv_store: ResMut<PkvStore>,
) {
    let current_level_index = current_level_index_query.single();
    

    if current_level_index.is_changed()
        || play_statistics.is_changed()
    {
        println!("Saving Game");

        let game_save = GameSave {
            current_index: current_level_index.0.clone(),
            play_statistics: play_statistics.0.clone(),
        };

        pkv_store.set(SAVE_DATA_KEY, &game_save);
    }
}

pub fn update_working_level(
    mut working_level_index_query: Query<&mut WorkingLevelIndex>,
    play_statistics: Res<PlayStatistics>,
) {
    if play_statistics.is_changed() {
        let level_index = play_statistics.get_working_level();

        working_level_index_query.single_mut().0 = level_index;
    }
}
