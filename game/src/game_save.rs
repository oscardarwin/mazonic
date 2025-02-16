use bevy::prelude::*;
use bevy::utils::{HashMap, HashSet};
use bevy_pkv::PkvStore;
use serde::{Deserialize, Serialize};

use crate::sound::Melody;

type LevelIndex = usize;

#[derive(Component, Debug, Clone)]
pub struct CurrentLevelIndex(pub LevelIndex);

#[derive(Component, Debug, Clone)]
pub struct WorkingLevelIndex(pub LevelIndex);

#[derive(Component, Debug, Clone)]
pub struct PerfectScoreLevelIndices(pub HashSet<LevelIndex>);

#[derive(Component, Debug, Clone)]
pub struct DiscoveredMelodies(pub HashMap<LevelIndex, DiscoveredMelody>);

impl DiscoveredMelodies {
    pub fn get_room_ids_for_level(&self, level_index: LevelIndex) -> HashSet<u64> {
        if let Some(DiscoveredMelody { room_ids, .. }) = self.0.get(&level_index) {
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
    pub current_index: LevelIndex,
    pub completed_index: LevelIndex,
    pub perfect_score_level_indices: HashSet<LevelIndex>,
    pub discovered_melodies: HashMap<LevelIndex, DiscoveredMelody>,
}

impl Default for GameSave {
    fn default() -> Self {
        GameSave {
            current_index: 3,
            completed_index: 19,
            perfect_score_level_indices: HashSet::new(),
            discovered_melodies: HashMap::new(),
        }
    }
}

const SAVE_DATA_KEY: &str = "save_data";

pub fn setup_save_data(mut commands: Commands, pkv_store: Res<PkvStore>) {
    let save_data = match pkv_store.get::<GameSave>(SAVE_DATA_KEY) {
        Ok(game_save) => game_save,
        Err(_) => GameSave::default(),
    };

    commands.spawn((
        CurrentLevelIndex(save_data.current_index),
        WorkingLevelIndex(save_data.completed_index),
        PerfectScoreLevelIndices(save_data.perfect_score_level_indices),
        DiscoveredMelodies(save_data.discovered_melodies),
    ));
}

pub fn update_save_data(
    current_level_index_query: Query<Ref<CurrentLevelIndex>>,
    working_level_index_query: Query<Ref<WorkingLevelIndex>>,
    perfect_score_level_indices_query: Query<Ref<PerfectScoreLevelIndices>>,
    discovered_melodies_query: Query<Ref<DiscoveredMelodies>>,
    mut pkv_store: ResMut<PkvStore>,
) {
    let current_level_index = current_level_index_query.single();
    let working_level_index = working_level_index_query.single();
    let perfect_score_level_indices = perfect_score_level_indices_query.single();
    let discovered_melodies = discovered_melodies_query.single();

    if current_level_index.is_changed()
        || working_level_index.is_changed()
        || perfect_score_level_indices.is_changed()
        || discovered_melodies.is_changed()
    {
        println!("Saving Game");

        let game_save = GameSave {
            current_index: current_level_index.0,
            completed_index: working_level_index.0,
            perfect_score_level_indices: perfect_score_level_indices.0.clone(),
            discovered_melodies: discovered_melodies.0.clone(),
        };

        pkv_store.set(SAVE_DATA_KEY, &game_save);
    }
}
