use bevy::{audio::AddAudioSource, prelude::*, utils::HashMap};
use bevy_rustysynth::{MidiAudio, MidiNote};

use crate::{
    is_room_junction::is_junction, player::PlayerMazeState, room::SolidRoom,
    shape::loader::GraphComponent, statistics::PlayerPath,
};

// MazeSaveData has a secret length and encrypted song title.

struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        //app.add_audio_source();
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
