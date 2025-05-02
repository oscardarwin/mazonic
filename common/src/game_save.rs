use std::path::PathBuf;

use bevy::prelude::*;
use bevy::utils::{HashMap, HashSet};
use bevy_pkv::PkvStore;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

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

#[derive(Component, Debug, Clone)]
pub struct CompletedEasyDailies(pub HashSet<DailyLevelId>);

#[derive(Component, Debug, Clone)]
pub struct CompletedHardDailies(pub HashSet<DailyLevelId>);

#[derive(Component, Debug, Clone)]
pub struct DiscoveredMelodies(pub HashMap<PuzzleIdentifier, DiscoveredMelody>);


impl DiscoveredMelodies {
    pub fn get_room_ids_for_level(&self, puzzle_identifier: &PuzzleIdentifier) -> HashSet<u64> {
        if let Some(DiscoveredMelody { room_ids, .. }) = self.0.get(puzzle_identifier) {
            room_ids.iter().cloned().collect()
        } else {
            HashSet::new()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredMelody {
    pub melody: Melody,
    pub room_ids: Vec<u64>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GameSave {
    pub current_index: PuzzleIdentifier,
    pub completed_index: LevelIndex,
    pub completed_easy_dailies: HashSet<DailyLevelId>,
    pub completed_hard_dailies: HashSet<DailyLevelId>,
    pub discovered_melodies: HashMap<PuzzleIdentifier, DiscoveredMelody>,
}

#[derive(Resource, Clone)]
pub struct SaveLocation(pub PathBuf);

impl Default for GameSave {
    fn default() -> Self {
        GameSave {
            current_index: PuzzleIdentifier::Level(0),
            completed_index: 0,
            completed_easy_dailies: HashSet::new(),
            completed_hard_dailies: HashSet::new(),
            discovered_melodies: HashMap::new(),
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
        Ok(game_save) => game_save,
        Err(_) => GameSave::default(),
    };

    commands.spawn((
        CurrentPuzzle(save_data.current_index),
        WorkingLevelIndex(save_data.completed_index),
        DiscoveredMelodies(save_data.discovered_melodies),
        CompletedEasyDailies(save_data.completed_easy_dailies),
        CompletedHardDailies(save_data.completed_hard_dailies),
    ));

    commands.insert_resource(pkv_store);
}

pub fn update(
    current_level_index_query: Query<Ref<CurrentPuzzle>>,
    working_level_index_query: Query<Ref<WorkingLevelIndex>>,
    discovered_melodies_query: Query<Ref<DiscoveredMelodies>>,
    completed_easy_dailies_query: Query<Ref<CompletedEasyDailies>>,
    completed_hard_dailies_query: Query<Ref<CompletedHardDailies>>,
    mut pkv_store: ResMut<PkvStore>,
) {
    let current_level_index = current_level_index_query.single();
    let working_level_index = working_level_index_query.single();
    let completed_easy_dailies = completed_easy_dailies_query.single();
    let completed_hard_dailies = completed_hard_dailies_query.single();
    let discovered_melodies = discovered_melodies_query.single();
    

    if current_level_index.is_changed()
        || working_level_index.is_changed()
        || discovered_melodies.is_changed()
        || completed_easy_dailies.is_changed()
        || completed_hard_dailies.is_changed()
    {
        println!("Saving Game");

        let game_save = GameSave {
            current_index: current_level_index.0.clone(),
            completed_index: working_level_index.0,
            completed_easy_dailies: completed_easy_dailies.0.clone(),
            completed_hard_dailies: completed_hard_dailies.0.clone(),
            discovered_melodies: discovered_melodies.0.clone(),
        };

        pkv_store.set(SAVE_DATA_KEY, &game_save);
    }
}
