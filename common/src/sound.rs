use std::collections::VecDeque;
use std::time::Duration;

use bevy::audio::PlaybackMode;
use bevy::reflect::List;
use bevy::{audio::AddAudioSource, prelude::*, utils::HashMap};
use bevy_rustysynth::{MidiAudio, MidiNote};
use chacha20poly1305::{AeadCore, ChaCha20Poly1305, Key, KeyInit};

use chacha20poly1305::aead::generic_array::typenum::Unsigned;
use chacha20poly1305::aead::generic_array::GenericArray;
use chacha20poly1305::aead::{Aead, Result};
use itertools::Itertools;
use rand::seq::IteratorRandom;
use rand::SeedableRng;
use rand_chacha::{ChaCha20Rng, ChaCha8Rng};
use serde::{Deserialize, Serialize};
use sha2::digest::typenum::Pow;
use sha2::{Digest, Sha256};

use crate::game_save::{CurrentPuzzle, DiscoveredMelody};
use crate::game_systems::SystemHandles;
use crate::maze::mesh::MazeMarker;
use crate::play_statistics::PlayStatistics;
use crate::shape::loader::SolutionComponent;
use crate::ui::message::{MessagePopup, MessagePopupUpperMarker};
use crate::{
    is_room_junction::is_junction, player::PlayerMazeState, room::Room,
    shape::loader::GraphComponent,
};

#[derive(Serialize, Deserialize, Clone, Debug, Hash)]
pub enum NoteValue {
    Semiquaver,
    Quaver,
    Crotchet,
    DottedCrotchet,
    Minim,
    DottedMinim,
    Semibreve,
    SemibreveAndCrotchet,
}

impl NoteValue {
    pub fn as_f32(&self) -> f32 {
        match self {
            NoteValue::Semiquaver => 0.25,
            NoteValue::Quaver => 0.5,
            NoteValue::Crotchet => 1.0,
            NoteValue::DottedCrotchet => 1.5,
            NoteValue::Minim => 2.0,
            NoteValue::DottedMinim => 3.0,
            NoteValue::Semibreve => 4.0,
            NoteValue::SemibreveAndCrotchet => 5.0,
        }

    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Hash)]
pub struct Note {
    pub key: i32,
    pub velocity: i32,
    pub value: NoteValue,
}

#[derive(Hash, Clone, Serialize, Deserialize, Debug)]
pub struct Notes(pub Vec<Note>);

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Melody {
    pub name: String,
    pub notes: Notes,
    pub bpm: f32,
}

impl Default for Melody {
    fn default() -> Self {
        Melody {
            name: "C Major Pentatonic".to_string(),
            notes: Notes(vec![
                Note::crotchet(48),
                Note::crotchet(50),
                Note::crotchet(52),
                Note::crotchet(55),
                Note::crotchet(57),
                Note::crotchet(60),
            ]),
            bpm: 100.0
        }
    }
}

impl Note {
    pub fn new(key: i32, value: NoteValue) -> Self {
        Note {
            key,
            velocity: 100,
            value,
        }
    }

    pub fn semiquaver(key: i32) -> Self {
        Self::new(key, NoteValue::Semiquaver)
    }

    pub fn quaver(key: i32) -> Self {
        Self::new(key, NoteValue::Quaver)
    }

    pub fn crotchet(key: i32) -> Self {
        Self::new(key, NoteValue::Crotchet)
    }

    pub fn dotted_crotchet(key: i32) -> Self {
        Self::new(key, NoteValue::DottedCrotchet)
    }

    pub fn minim(key: i32) -> Self {
        Self::new(key, NoteValue::Minim)
    }

    pub fn dotted_minim(key: i32) -> Self {
        Self::new(key, NoteValue::DottedMinim)
    }

    pub fn semibreve(key: i32) -> Self {
        Self::new(key, NoteValue::Semibreve)
    }

    pub fn semibreve_and_crotchet(key: i32) -> Self {
        Self::new(key, NoteValue::SemibreveAndCrotchet)
    }
}

#[derive(Component)]
pub struct NoteMapping(pub HashMap<u64, (Handle<MidiAudio>, Note)>);

#[derive(Component)]
pub struct MelodyPuzzleTracker {
    pub room_ids: VecDeque<u64>,
    pub encrypted_melody_bytes: Vec<u8>,
}

pub fn play_note(
    mut commands: Commands,
    mut previous_room_local: Local<Option<Room>>,
    mut melody_tracker_query: Query<&mut MelodyPuzzleTracker>,
    graph_component: Query<&GraphComponent>,
    solution_query: Query<&SolutionComponent>,
    player_query: Query<&PlayerMazeState>,
    note_mapping: Query<&NoteMapping>,
    asset_server: Res<AssetServer>,
) {
    let Ok(GraphComponent(graph)) = graph_component.get_single() else {
        return;
    };

    let Ok(PlayerMazeState::Node(room)) = player_query.get_single() else {
        return;
    };

    let play_sound = match *previous_room_local {
        Some(previous_room) => previous_room != *room && is_junction(room, graph),
        None => true,
    };

    *previous_room_local = Some(*room);

    if !play_sound {
        return;
    }

    let Ok(NoteMapping(note_mapping)) = note_mapping.get_single() else {
        return;
    };

    let final_room = solution_query
        .get_single()
        .ok()
        .and_then(|SolutionComponent(solution)| solution.last())
        .unwrap();

    if room != final_room {
        let (note_handle, note) = note_mapping.get(&room.id).unwrap().clone();

        if let Ok(mut melody_tracker) = melody_tracker_query.get_single_mut() {
            if melody_tracker.room_ids.len() == melody_tracker.room_ids.capacity() {
                melody_tracker.room_ids.pop_front();
            }

            melody_tracker.room_ids.push_back(room.id);
        }

        commands.spawn(AudioSourceBundle {
            source: AudioPlayer(note_handle),
            settings: get_playback_settings(1.0)
        });
    } else {
        play_winning_melody(
            commands,
            note_mapping.values().map(|(_, note)| note).collect(),
            asset_server,
        );
    }
}

fn play_winning_melody(
    mut commands: Commands,
    level_notes: Vec<&Note>,
    asset_server: Res<AssetServer>,
) {
    let mut rng = ChaCha20Rng::from_entropy();

    let notes = level_notes
        .into_iter()
        .unique_by(|note| note.key)
        .choose_multiple(&mut rng, 6);

    let num_notes = notes.len();
    let mut midi_notes = notes
        .into_iter()
        .enumerate()
        .map(|(index, note)| {
            let speedup = if index == num_notes - 1 {
                1.0
            } else {
                3.0 * 1.3_f32.powf(index as f32)
            };
            let fast_note_duration = Duration::from_secs_f32(note.value.as_f32() / speedup);

            MidiNote {
                key: note.key,
                velocity: note.velocity,
                duration: fast_note_duration,
                ..Default::default()
            }
        })
        .collect_vec();

    let midi_audio = MidiAudio::Sequence(midi_notes);
    let audio_handle = asset_server.add::<MidiAudio>(midi_audio);
    commands.spawn(AudioSourceBundle {
        source: AudioPlayer(audio_handle),
        settings: get_playback_settings(1.0)
    });
}

fn get_playback_settings(speed: f32) -> PlaybackSettings {
    PlaybackSettings {
        mode: PlaybackMode::Despawn,
        speed,
        ..Default::default()
    }
}

pub fn check_melody_solved(
    melody_tracker_query: Query<&MelodyPuzzleTracker, Changed<MelodyPuzzleTracker>>,
    room_id_note_mapping_query: Query<&NoteMapping>,
    mut play_statistics: ResMut<PlayStatistics>,
    current_level_index_query: Query<&CurrentPuzzle>,
    system_handles: Res<SystemHandles>,
    mut commands: Commands,
    maze_entities_query: Query<Entity, With<MazeMarker>>,
    mut message_popup_query: Query<&mut MessagePopup, With<MessagePopupUpperMarker>>,
) {
    let Ok(melody_tracker) = melody_tracker_query.get_single() else {
        return;
    };

    let Ok(NoteMapping(room_id_note_mapping)) = room_id_note_mapping_query.get_single() else {
        return;
    };

    let notes = Notes(
        melody_tracker
            .room_ids
            .iter()
            .map(|room_id| {
                let (_, note) = room_id_note_mapping.get(room_id).unwrap();
                note.clone()
            })
            .collect_vec(),
    );

    let Some(melody) = try_decrypt_melody(&notes, &melody_tracker.encrypted_melody_bytes) else {
        return;
    };
    
    message_popup_query.single_mut().0 = format!("~ {} ~", melody.name);

    let discovered_melody = DiscoveredMelody {
        melody,
        room_ids: melody_tracker.room_ids.clone().into(),
    };

    let CurrentPuzzle(puzzle_identifier) = current_level_index_query.single();

    play_statistics.0.entry(puzzle_identifier.clone()).and_modify(|play_statistics| play_statistics.discovered_melody = Some(discovered_melody));

    commands.run_system(system_handles.update_on_melody_discovered);
    commands.run_system(system_handles.note_burst);
    commands.run_system(system_handles.play_melody);
}

pub fn play_melody(
    current_level_index_query: Query<&CurrentPuzzle>,
    play_statistics: Res<PlayStatistics>,
    asset_server: ResMut<AssetServer>,
    mut commands: Commands,
) {
    let CurrentPuzzle(puzzle_identifier) = current_level_index_query.single();

    let Some(discovered_melody) = play_statistics.0.get(puzzle_identifier).and_then(|statistics| statistics.discovered_melody.clone()) else {
        return;
    };

    let Notes(notes) = &discovered_melody.melody.notes;

    let seconds_per_note = 60.0 / discovered_melody.melody.bpm;
    let mut midi_notes = notes.iter().map(|note| MidiNote {
            key: note.key,
            velocity: note.velocity,
            duration: Duration::from_secs_f32(note.value.as_f32() * seconds_per_note),
            ..Default::default()
        }).collect_vec();

    let pause_note = MidiNote {
        velocity: 0,
        duration: Duration::from_millis(800),
        ..Default::default()
    };
    midi_notes.insert(0, pause_note);
    let midi_audio = MidiAudio::Sequence(midi_notes);
    let audio_handle = asset_server.add::<MidiAudio>(midi_audio);
    commands.spawn(AudioSourceBundle {
        source: AudioPlayer(audio_handle),
        ..Default::default()
    });
}

fn try_decrypt_melody(notes: &Notes, encrypted_melody: &Vec<u8>) -> Option<Melody> {
    type NonceSize = <ChaCha20Poly1305 as AeadCore>::NonceSize;

    let notes_hash_bytes = hash_melody(notes);

    let key = Key::from_slice(&notes_hash_bytes);
    let cipher = ChaCha20Poly1305::new(key);
    let (nonce, ciphertext) = encrypted_melody.split_at(NonceSize::to_usize());
    let nonce = GenericArray::from_slice(nonce);
    let plaintext = cipher.decrypt(nonce, ciphertext).ok()?;

    serde_json::from_slice(&plaintext).ok()
}

pub fn hash_melody(notes: &Notes) -> [u8; 32] {
    let notes_string = serde_json::to_vec(notes).unwrap();
    let mut hasher = Sha256::new();
    hasher.update(notes_string);
    let hash_result: [u8; 32] = hasher.finalize().into();

    hash_result
}
