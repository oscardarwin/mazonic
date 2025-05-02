use bevy::prelude::*;
use bevy_hanabi::prelude::*;
use itertools::repeat_n;

use crate::{game_save::{CurrentPuzzle, DiscoveredMelodies}, game_settings::GameSettings};

#[derive(Component, Debug, Clone)]
pub enum MusicalNoteEffectColor {
    Player,
    Line,
}

#[derive(Component, Debug, Clone)]
pub struct MusicalNoteEffectHandle {
    pub player_color_effect_handles: Vec<Handle<EffectAsset>>,
    pub line_color_effect_handles: Vec<Handle<EffectAsset>>,
}

#[derive(Component, Debug, Clone)]
pub struct MusicalNoteImageHandles {
    pub crotchet_handle: Handle<Image>,
    pub quaver_handle: Handle<Image>,
}

#[derive(Component, Debug, Clone)]
pub struct MusicalNoteMarker(pub usize, pub MusicalNoteEffectColor);

pub fn setup(
    mut effects: ResMut<Assets<EffectAsset>>,
    mut assets: Res<AssetServer>,
    mut commands: Commands,
    game_settings: Res<GameSettings>,
) {
    let crotchet_handle = assets.load("sprites/crotchet.png");
    let quaver_handle = assets.load("sprites/quaver.png");

    let line_color_effect_handles = create_note_effects_for_color(&mut effects, game_settings.palette.line_color);
    let player_color_effect_handles = create_note_effects_for_color(&mut effects, game_settings.palette.player_color);
    
    commands.spawn(MusicalNoteEffectHandle { player_color_effect_handles, line_color_effect_handles });
    commands.spawn(MusicalNoteImageHandles {
        crotchet_handle,
        quaver_handle,
    });
}

fn create_note_effects_for_color(mut effects: &mut Assets<EffectAsset>, color: Color) -> Vec<Handle<EffectAsset>> {
    let num_effects = 8;
    (0..num_effects)
        .map(|index| {
            let effect = create_note_effect(color, num_effects, index);

            effects.add(effect.with_name(format!("Note {index}")))
        })
        .collect()
}

fn create_note_effect(
    color: Color,
    num_effects: usize,
    effect_index: usize,
) -> EffectAsset {
    let mut gradient = Gradient::new();
    let end_color = color
        .to_linear()
        .with_alpha(0.9)
        .to_vec4();
    let start_color = color
        .to_linear()
        .with_alpha(0.0)
        .to_vec4();

    let float_num_effects = num_effects as f32;
    let float_effect_index = effect_index as f32;
    let start_time = float_effect_index / float_num_effects;
    let end_time = (float_effect_index + 1.0) / float_num_effects;
    let middle_time = start_time + 0.7 * (end_time - start_time);

    gradient.add_key(start_time, start_color.clone());
    gradient.add_key(middle_time, end_color);
    gradient.add_key(end_time, start_color);

    let writer = ExprWriter::new();

    let init_pos = SetPositionCircleModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        radius: writer.lit(0.02).expr(),
        axis: writer.lit(Vec3::Y).expr(),
        dimension: ShapeDimension::Volume,
    };

    let orient = OrientModifier {
        mode: OrientMode::ParallelCameraDepthPlane,
        rotation: None,
    };

    let init_vel = SetVelocityTangentModifier {
        axis: writer.lit(Vec3::Y).expr(),
        origin: writer.lit(Vec3::ZERO).expr(),
        speed: writer.lit(0.008).expr(),
    };

    let lifetime = writer.lit(4.0 * float_num_effects).expr();
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);
    let init_size = SetAttributeModifier::new(Attribute::SIZE, writer.lit(0.1).expr());

    let render_image = ParticleTextureModifier {
        texture_slot: writer.lit(0_u32).expr(),
        sample_mapping: ImageSampleMapping::Modulate,
    };

    let accel = RadialAccelModifier::new(writer.lit(Vec3::ZERO).expr(), writer.lit(-0.0001).expr());

    let mut module = writer.finish();
    module.add_texture_slot("note");

    EffectAsset::new(64, Spawner::rate(CpuValue::Uniform((0.08, 0.15))), module)
        .init(init_pos)
        .init(init_size)
        .init(init_vel)
        .init(init_lifetime)
        .update(accel)
        .render(orient)
        .render(render_image)
        .with_simulation_condition(SimulationCondition::Always)
        .render(ColorOverLifetimeModifier { gradient })
}

pub fn spawn(
    mut commands: Commands,
    musical_note_effect_handle: Query<&MusicalNoteEffectHandle>,
    musical_note_image_handle_query: Query<&MusicalNoteImageHandles>,
    musical_note_marker_query: Query<(Entity, &Transform, &MusicalNoteMarker), Added<MusicalNoteMarker>>,
) {
    if musical_note_marker_query.is_empty() {
        return;
    }

    let Ok(MusicalNoteEffectHandle { player_color_effect_handles, line_color_effect_handles }) = musical_note_effect_handle.get_single()
    else {
        return;
    };
    let num_effect_handles = player_color_effect_handles.len();

    let Ok(MusicalNoteImageHandles {
        crotchet_handle,
        quaver_handle,
    }) = musical_note_image_handle_query.get_single()
    else {
        return;
    };

    for (entity, transform, MusicalNoteMarker(melody_index, color)) in musical_note_marker_query.iter() {
        let mut entity_commands = commands.entity(entity);
        let index = entity.index() as usize;

        let crotchet_effect_handle_index = melody_index % num_effect_handles;
        let quaver_effect_handle_index = (melody_index + num_effect_handles / 2) % num_effect_handles;
        
        let effect_handles = match color {
            MusicalNoteEffectColor::Player => &player_color_effect_handles,
            MusicalNoteEffectColor::Line => &line_color_effect_handles,
        };

        let particle_effect_entity = entity_commands.with_children(|parent| {
            parent
                .spawn(ParticleEffectBundle {
                    effect: ParticleEffect::new(
                        effect_handles[crotchet_effect_handle_index].clone(),
                    ),
                    transform: Transform::IDENTITY,
                    ..Default::default()
                })
                .insert(EffectMaterial {
                    images: vec![crotchet_handle.clone()],
                });

            parent
                .spawn(ParticleEffectBundle {
                    effect: ParticleEffect::new(effect_handles[quaver_effect_handle_index].clone()),
                    transform: Transform::IDENTITY,
                    ..Default::default()
                })
                .insert(EffectMaterial {
                    images: vec![quaver_handle.clone()],
                });
        });
    }
}
