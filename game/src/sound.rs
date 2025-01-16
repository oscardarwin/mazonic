use std::time::Duration;

use bevy::{audio::AddAudioSource, prelude::*, utils::HashMap};
use bevy_rustysynth::{MidiAudio, MidiNote};
use serde::{Deserialize, Serialize};

use crate::{
    is_room_junction::is_junction, player::PlayerMazeState, room::SolidRoom,
    shape::loader::GraphComponent, statistics::PlayerPath,
};

const CROTCHET_DURATION: f32 = 0.8;

#[derive(Serialize, Deserialize, Clone)]
pub struct Note {
    pub key: i32,
    pub velocity: i32,
    pub duration: Duration,
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
pub struct NoteMapping(pub HashMap<u64, Handle<MidiAudio>>);

pub fn play_note(
    mut commands: Commands,
    mut last_room_local: Local<Option<SolidRoom>>,

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

    let note = note_mapping.get(&room.id).unwrap().clone();
    commands.spawn(AudioSourceBundle {
        source: AudioPlayer(note),
        ..Default::default()
    });
}
