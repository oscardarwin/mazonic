use std::collections::VecDeque;
use std::time::Duration;

use bevy::reflect::List;
use bevy::{audio::AddAudioSource, prelude::*, utils::HashMap};
use bevy_rustysynth::{MidiAudio, MidiNote};
use chacha20poly1305::{AeadCore, ChaCha20Poly1305, Key, KeyInit};

use chacha20poly1305::aead::generic_array::typenum::Unsigned;
use chacha20poly1305::aead::generic_array::GenericArray;
use chacha20poly1305::aead::{Aead, Result};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    is_room_junction::is_junction, player::PlayerMazeState, room::Room,
    shape::loader::GraphComponent, statistics::PlayerPath,
};

const CROTCHET_DURATION: f32 = 0.8;

#[derive(Serialize, Deserialize, Clone, Debug, Hash)]
pub struct Note {
    pub key: i32,
    pub velocity: i32,
    pub duration: Duration,
}

#[derive(Hash, Clone, Serialize, Deserialize, Debug)]
pub struct Notes(pub Vec<Note>);

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Melody {
    pub name: String,
    pub notes: Notes,
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
        }
    }
}

impl Note {
    pub fn new(key: i32, duration: Duration) -> Self {
        Note {
            key,
            velocity: 100,
            duration,
        }
    }

    pub fn quaver(key: i32) -> Self {
        Self::new(key, Duration::from_secs_f32(CROTCHET_DURATION * 0.5))
    }

    pub fn crotchet(key: i32) -> Self {
        Self::new(key, Duration::from_secs_f32(CROTCHET_DURATION))
    }

    pub fn minim(key: i32) -> Self {
        Self::new(key, Duration::from_secs_f32(CROTCHET_DURATION * 2.0))
    }

    pub fn dotted_minim(key: i32) -> Self {
        Self::new(key, Duration::from_secs_f32(CROTCHET_DURATION * 3.0))
    }

    pub fn semibreve(key: i32) -> Self {
        Self::new(key, Duration::from_secs_f32(CROTCHET_DURATION * 4.0))
    }
}

impl Into<MidiNote> for Note {
    fn into(self) -> MidiNote {
        MidiNote {
            key: self.key,
            velocity: self.velocity,
            duration: self.duration,
            ..Default::default()
        }
    }
}

#[derive(Component)]
pub struct NoteMapping(pub HashMap<u64, (Handle<MidiAudio>, Note)>);

#[derive(Component)]
pub struct MelodyPuzzleTracker {
    pub notes: VecDeque<Note>,
    pub encrypted_melody_bytes: Vec<u8>,
}

pub fn play_note(
    mut commands: Commands,
    mut last_room_local: Local<Option<Room>>,
    mut melody_tracker_query: Query<&mut MelodyPuzzleTracker>,
    graph_component: Query<&GraphComponent>,
    player_query: Query<&PlayerMazeState>,
    note_mapping: Query<&NoteMapping>,
) {
    let Ok(GraphComponent(graph)) = graph_component.get_single() else {
        return;
    };

    let Ok(PlayerMazeState::Node(room)) = player_query.get_single() else {
        return;
    };

    let last_room = last_room_local.unwrap_or(*room);

    *last_room_local = Some(*room);

    if *room == last_room || !is_junction(&room, &graph) {
        return;
    }

    let Ok(NoteMapping(note_mapping)) = note_mapping.get_single() else {
        return;
    };

    let (note_handle, note) = note_mapping.get(&room.id).unwrap().clone();

    if let Ok(mut melody_tracker) = melody_tracker_query.get_single_mut() {
        if melody_tracker.notes.len() == melody_tracker.notes.capacity() {
            melody_tracker.notes.pop_front();
        }

        melody_tracker.notes.push_back(note.clone());
    }
    commands.spawn(AudioSourceBundle {
        source: AudioPlayer(note_handle),
        ..Default::default()
    });
}

pub fn check_melody_solved(
    melody_tracker_query: Query<&MelodyPuzzleTracker, Changed<MelodyPuzzleTracker>>,
) {
    let Ok(melody_tracker) = melody_tracker_query.get_single() else {
        return;
    };

    let notes = Notes(melody_tracker.notes.iter().cloned().collect_vec());

    let Some(melody) = try_decrypt_melody(&notes, &melody_tracker.encrypted_melody_bytes) else {
        return;
    };

    println!("Solved Melody: {}", melody.name);
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
