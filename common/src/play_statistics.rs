use std::time::Duration;

use bevy::{prelude::*, time::Stopwatch, utils::{HashMap, HashSet}};
use serde::{Deserialize, Serialize};

use crate::game_save::{CurrentPuzzle, DiscoveredMelody, LevelIndex, PuzzleIdentifier};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PuzzleStatistics {
    pub completed: bool,
    pub time_spent: Duration,
    pub discovered_melody: Option<DiscoveredMelody>,
}

#[derive(Resource, Debug, Clone)]
pub struct PlayStatistics(pub HashMap<PuzzleIdentifier, PuzzleStatistics>);

impl PlayStatistics {
    pub fn get_working_level(&self) -> LevelIndex {
        let highest_completed_level_index = self.0.iter()
            .filter_map(|(puzzle_identifier, puzzle_statistics)| match puzzle_identifier { 
                    PuzzleIdentifier::Level(level) if puzzle_statistics.completed => Some(level), 
                    _ => None
            })
            .max();

        match highest_completed_level_index {
            Some(level) => level + 1,
            None => 0,
        }
    }

    pub fn get_melody_room_ids(&self, puzzle_identifier: &PuzzleIdentifier) -> Vec<u64> {
        self.0.get(puzzle_identifier)
            .iter()
            .filter_map(|puzzle_statistics| puzzle_statistics.discovered_melody.clone().map(|melody| melody.room_ids.clone()))
            .flatten()
            .collect()
    }
}

#[derive(Resource, Default)]
pub struct SolveTime {
    pub stopwatch: Stopwatch,
    pub running: bool, 
}

pub fn setup(mut commands: Commands) {
    commands.insert_resource(SolveTime::default());
}

pub fn on_play(
    mut solve_time: ResMut<SolveTime>,
    current_puzzle_query: Query<&CurrentPuzzle>,
    mut play_statistics: ResMut<PlayStatistics>,
) {
    let Ok(CurrentPuzzle(puzzle_identifier)) = current_puzzle_query.get_single() else {
        return;
    };

    let statistics = play_statistics.0.entry(puzzle_identifier.clone()).or_insert(PuzzleStatistics::default());

    solve_time.stopwatch.set_elapsed(statistics.time_spent);
    solve_time.running = !statistics.completed;
}

pub fn on_victory(
    current_puzzle_query: Query<&CurrentPuzzle>,
    mut play_statistics: ResMut<PlayStatistics>,
) {
    let Ok(CurrentPuzzle(puzzle_identifier)) = current_puzzle_query.get_single() else {
        return;
    };

    play_statistics.0.entry(puzzle_identifier.clone()).and_modify(|puzzle_statistics| {
        puzzle_statistics.completed = true
    });
}

pub fn during_play(time: Res<Time>, mut solve_time: ResMut<SolveTime>) {
    if solve_time.running {
        solve_time.stopwatch.tick(time.delta());
    }
}

pub fn exit_play(
    solve_time: Res<SolveTime>,
    current_puzzle_query: Query<&CurrentPuzzle>,
    mut play_statistics: ResMut<PlayStatistics>,
    ) {
    let Ok(CurrentPuzzle(puzzle_identifier)) = current_puzzle_query.get_single() else {
        return;
    };

    play_statistics.0.entry(puzzle_identifier.clone()).and_modify(|puzzle_statistics| {
        if !puzzle_statistics.completed {
            puzzle_statistics.time_spent = solve_time.stopwatch.elapsed();
        }
    });
}

