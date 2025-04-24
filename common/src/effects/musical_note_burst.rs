use bevy::prelude::*;
use bevy_hanabi::prelude::*;

use crate::{
    game_save::{CurrentLevel, DiscoveredMelodies},
    game_settings::GameSettings,
    levels::LevelData,
    room::Room,
    shape::loader::GraphComponent,
};

use super::musical_notes::MusicalNoteImageHandles;

#[derive(Component, Debug, Clone)]
pub struct NoteBurstEffectHandles {
    burst_handle: Handle<EffectAsset>,
}

#[derive(Component, Debug, Clone)]
pub struct NoteEffectMarker {
    start_time: f32,
}

const BURST_LIFETIME: f32 = 1.5;

pub fn setup(
    mut effects: ResMut<Assets<EffectAsset>>,
    game_settings: Res<GameSettings>,
    mut commands: Commands,
) {
    let mut gradient = Gradient::new();

    let line_color = &game_settings.palette.line_color.to_linear();

    gradient.add_key(0., line_color.with_alpha(0.0).to_vec4());
    gradient.add_key(0.1, line_color.with_alpha(1.0).to_vec4());
    gradient.add_key(1.0, line_color.with_alpha(0.0).to_vec4());

    let writer = ExprWriter::new();

    let zero_vec = writer.lit(Vec3::ZERO).expr();

    let init_pos = SetPositionSphereModifier {
        center: zero_vec.clone(),
        radius: writer.lit(0.1).expr(),
        dimension: ShapeDimension::Surface,
    };

    let orient = OrientModifier {
        mode: OrientMode::ParallelCameraDepthPlane,
        rotation: None,
    };

    let init_vel = SetVelocitySphereModifier {
        center: zero_vec.clone(),
        speed: writer.lit(0.2).expr(),
    };

    let lifetime = writer.lit(BURST_LIFETIME).expr();
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);
    let init_size = SetAttributeModifier::new(Attribute::SIZE, writer.lit(0.15).expr());

    let render_image = ParticleTextureModifier {
        texture_slot: writer.lit(0_u32).expr(),
        sample_mapping: ImageSampleMapping::Modulate,
    };

    let sphere_radius = writer.lit(1.4).expr();
    let accel = RadialAccelModifier::new(zero_vec.clone(), writer.lit(-0.01).expr());
    let mut module = writer.finish();
    module.add_texture_slot("note");

    let effect = EffectAsset::new(
        64,
        Spawner::new(2.0f32.into(), 0.05f32.into(), BURST_LIFETIME.into()),
        module.clone(),
    )
    .init(init_pos)
    .init(init_size)
    .init(init_vel)
    .init(init_lifetime)
    .update(accel)
    .render(orient)
    .render(render_image)
    .with_simulation_condition(SimulationCondition::Always)
    .render(ColorOverLifetimeModifier { gradient });

    let burst_handle = effects.add(effect.with_name(format!("Note Burst")));

    commands.spawn(NoteBurstEffectHandles { burst_handle });
}

pub fn clear_up_effects(
    effect_entities: Query<(Entity, &NoteEffectMarker)>,
    mut commands: Commands,
    time: Res<Time>,
) {
    let current_time = time.elapsed_secs();

    for (entity, NoteEffectMarker { start_time }) in effect_entities.iter() {
        if current_time - start_time > BURST_LIFETIME {
            commands.entity(entity).despawn_recursive();
        }
    }
}

pub fn spawn(
    rooms_query: Query<(&Room, &Transform)>,
    discovered_melodies: Query<&DiscoveredMelodies>,
    level_index: Query<&CurrentLevel>,
    game_settings: Res<GameSettings>,
    keys: Res<ButtonInput<KeyCode>>,
    musical_note_image_handle_query: Query<&MusicalNoteImageHandles>,
    note_burst_effect_handles_query: Query<&NoteBurstEffectHandles>,
    mut commands: Commands,
    time: Res<Time>,
) {
    let Ok(CurrentLevel(current_level_index)) = level_index.get_single() else {
        return;
    };

    let Ok(discovered_melodies) = discovered_melodies.get_single() else {
        return;
    };

    let Ok(MusicalNoteImageHandles {
        crotchet_handle,
        quaver_handle,
    }) = musical_note_image_handle_query.get_single()
    else {
        return;
    };

    let Ok(NoteBurstEffectHandles { burst_handle }) = note_burst_effect_handles_query.get_single()
    else {
        return;
    };

    let melody_room_ids = discovered_melodies.get_room_ids_for_level(*current_level_index);

    println!("spawning note burst");
    for (room, transform) in rooms_query
        .iter()
        .filter(|(room, _)| melody_room_ids.contains(&room.id))
    {
        let texture_handle = if room.id % 2 == 0 {
            crotchet_handle.clone()
        } else {
            quaver_handle.clone()
        };

        commands
            .spawn(ParticleEffectBundle {
                effect: ParticleEffect::new(burst_handle.clone()),
                transform: transform.clone(),
                ..Default::default()
            })
            .insert(EffectMaterial {
                images: vec![texture_handle],
            })
            .insert(NoteEffectMarker {
                start_time: time.elapsed_secs(),
            });
    }
}
