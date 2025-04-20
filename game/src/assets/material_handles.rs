use crate::{game_settings::GameSettings, levels::LEVELS};
use bevy::{
    pbr::{ExtendedMaterial, MaterialExtension},
    prelude::*,
};

use super::shaders::{
    DashedArrowShader, GlobalShader, MenuSelectionHoverShader, PlayerHaloShader,
    PulsingDashedArrowShader, PulsingShader,
};

pub struct FaceMaterialHandles {
    pub face_handles: [Handle<ExtendedMaterial<StandardMaterial, GlobalShader>>; 6],
}

impl FaceMaterialHandles {
    fn get_material(
        &self,
        index: usize,
    ) -> Handle<ExtendedMaterial<StandardMaterial, GlobalShader>> {
        self.face_handles[index].clone()
    }

    pub fn tetrahedron(&self) -> [Handle<ExtendedMaterial<StandardMaterial, GlobalShader>>; 4] {
        [
            self.get_material(0),
            self.get_material(1),
            self.get_material(2),
            self.get_material(3),
        ]
    }

    pub fn cube(&self) -> [Handle<ExtendedMaterial<StandardMaterial, GlobalShader>>; 6] {
        [
            self.get_material(0),
            self.get_material(1),
            self.get_material(1),
            self.get_material(2),
            self.get_material(2),
            self.get_material(0),
        ]
    }

    pub fn octahedron(&self) -> [Handle<ExtendedMaterial<StandardMaterial, GlobalShader>>; 8] {
        [
            self.get_material(0),
            self.get_material(1),
            self.get_material(2),
            self.get_material(3),
            self.get_material(2),
            self.get_material(3),
            self.get_material(0),
            self.get_material(1),
        ]
    }

    pub fn dodecahedron(&self) -> [Handle<ExtendedMaterial<StandardMaterial, GlobalShader>>; 12] {
        [
            self.get_material(1),
            self.get_material(3),
            self.get_material(0),
            self.get_material(1),
            self.get_material(2),
            self.get_material(3),
            self.get_material(0),
            self.get_material(3),
            self.get_material(1),
            self.get_material(2),
            self.get_material(2),
            self.get_material(0),
        ]
    }

    pub fn icosahedron(&self) -> [Handle<ExtendedMaterial<StandardMaterial, GlobalShader>>; 20] {
        [
            self.get_material(0),
            self.get_material(1),
            self.get_material(2),
            self.get_material(3),
            self.get_material(4),
            self.get_material(1),
            self.get_material(3),
            self.get_material(4),
            self.get_material(0),
            self.get_material(1),
            self.get_material(2),
            self.get_material(2),
            self.get_material(4),
            self.get_material(0),
            self.get_material(3),
            self.get_material(1),
            self.get_material(0),
            self.get_material(2),
            self.get_material(4),
            self.get_material(3),
        ]
    }
}

pub struct SelectorHandles {
    pub unavailable: Handle<StandardMaterial>,
    pub completed: Handle<StandardMaterial>,
    pub perfect_score: Handle<StandardMaterial>,
    pub melody_found: Handle<StandardMaterial>,
    pub selection_pressed: Handle<ExtendedMaterial<StandardMaterial, MenuSelectionHoverShader>>,
    pub selection_hover: Handle<ExtendedMaterial<StandardMaterial, MenuSelectionHoverShader>>,
    pub level_symbols: Handle<StandardMaterial>,
    pub melody_found_selector_face: Handle<ExtendedMaterial<StandardMaterial, PulsingShader>>,
    pub incomplete_face_colors: [Handle<StandardMaterial>; LEVELS.len()],
}

#[derive(Resource)]
pub struct MaterialHandles {
    pub player_halo_handle: Handle<ExtendedMaterial<StandardMaterial, PlayerHaloShader>>,
    pub player_handle: Handle<StandardMaterial>,
    pub line_handle: Handle<StandardMaterial>,
    pub bright_pulsing_line_handle: Handle<ExtendedMaterial<StandardMaterial, PulsingShader>>,
    pub dashed_arrow_handle: Handle<ExtendedMaterial<StandardMaterial, DashedArrowShader>>,
    pub bright_dashed_arrow_handle: Handle<ExtendedMaterial<StandardMaterial, DashedArrowShader>>,
    pub bright_pulsing_dashed_arrow_handle:
        Handle<ExtendedMaterial<StandardMaterial, PulsingDashedArrowShader>>,
    pub face_handles: FaceMaterialHandles,
    pub selector: SelectorHandles,
    pub goal_handle: Handle<ExtendedMaterial<StandardMaterial, PulsingShader>>,
}

pub const ALPHA_MODE: AlphaMode = AlphaMode::AlphaToCoverage;

pub fn setup_materials(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut dashed_arrow_materials: ResMut<
        Assets<ExtendedMaterial<StandardMaterial, DashedArrowShader>>,
    >,
    mut player_halo_materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, PlayerHaloShader>>>,
    mut pulsing_materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, PulsingShader>>>,
    mut pulsing_dashed_materials: ResMut<
        Assets<ExtendedMaterial<StandardMaterial, PulsingDashedArrowShader>>,
    >,
    mut menu_selection_hover_materials: ResMut<
        Assets<ExtendedMaterial<StandardMaterial, MenuSelectionHoverShader>>,
    >,
    mut shape_face_materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, GlobalShader>>>,
    asset_server: Res<AssetServer>,
    game_settings: Res<GameSettings>,
) {
    let goal_handle = pulsing_materials.add(ExtendedMaterial {
        base: StandardMaterial {
            base_color: game_settings.palette.player_color,
            alpha_mode: ALPHA_MODE,
            ..Default::default()
        },
        extension: PulsingShader {},
    });

    let player_color = &game_settings.palette.player_color.to_linear();
    let player_halo_handle = player_halo_materials.add(ExtendedMaterial {
        base: StandardMaterial {
            base_color: Color::LinearRgba(*player_color),
            emissive: LinearRgba::from_vec3(player_color.to_vec3() * 2.0),
            alpha_mode: ALPHA_MODE,
            diffuse_transmission: 1.0,
            thickness: 0.17,
            metallic: 0.2,
            fog_enabled: true,
            double_sided: true,
            ..Default::default()
        },
        extension: PlayerHaloShader {},
    });
    let player_handle = materials.add(StandardMaterial {
        base_color: Color::LinearRgba(*player_color),
        emissive: LinearRgba::from_vec3(player_color.to_vec3() * 1.5),
        alpha_mode: ALPHA_MODE,
        reflectance: 0.1,
        ..Default::default()
    });

    let line_color = &game_settings.palette.line_color;
    let line_color_vec = line_color.to_linear().to_vec3();

    let line_material = StandardMaterial {
        base_color: *line_color,
        alpha_mode: ALPHA_MODE,
        ..Default::default()
    };

    let bright_line = StandardMaterial {
        base_color: *line_color,
        alpha_mode: ALPHA_MODE,
        emissive: LinearRgba::from_vec3(line_color_vec * 20.0),
        ..Default::default()
    };

    let bright_pulsing_line_handle = pulsing_materials.add(ExtendedMaterial {
        base: bright_line.clone(),
        extension: PulsingShader {},
    });

    let bright_pulsing_dashed_arrow_handle = pulsing_dashed_materials.add(ExtendedMaterial {
        base: bright_line.clone(),
        extension: PulsingDashedArrowShader {},
    });

    let dashed_arrow_handle = dashed_arrow_materials.add(ExtendedMaterial {
        base: line_material.clone(),
        extension: DashedArrowShader {},
    });

    let bright_dashed_arrow_handle = dashed_arrow_materials.add(ExtendedMaterial {
        base: bright_line.clone(),
        extension: DashedArrowShader {},
    });

    let face_handles = game_settings.palette.face_colors.colors.map(|color| {
        shape_face_materials.add(ExtendedMaterial {
            base: StandardMaterial {
                base_color: color,
                reflectance: 0.0,
                alpha_mode: ALPHA_MODE,
                perceptual_roughness: 1.0,
                ..Default::default()
            },
            extension: GlobalShader {},
        })
    });

    let selection_hover = menu_selection_hover_materials.add(ExtendedMaterial {
        base: StandardMaterial {
            base_color: line_color.with_alpha(0.75),
            emissive: LinearRgba::from_vec3(line_color_vec * 20.0),
            alpha_mode: ALPHA_MODE,
            ..Default::default()
        },
        extension: MenuSelectionHoverShader {},
    });
    let selection_pressed = menu_selection_hover_materials.add(ExtendedMaterial {
        base: StandardMaterial {
            base_color: line_color.with_alpha(0.9),
            emissive: LinearRgba::from_vec3(line_color_vec * 50.0),
            alpha_mode: ALPHA_MODE,
            ..Default::default()
        },
        extension: MenuSelectionHoverShader {},
    });
    let face_colors = &game_settings.palette.face_colors.colors;

    let level_symbol_sprite_sheet = asset_server.load("sprites/symbols_sprite_sheet.png");
    let level_symbols = materials.add(StandardMaterial {
        base_color_texture: Some(level_symbol_sprite_sheet.clone()),
        base_color: game_settings.palette.line_color,
        alpha_mode: ALPHA_MODE,
        emissive: LinearRgba::from_vec3(line_color_vec * 10.0),
        ..Default::default()
    });

    let melody_found_selector_face = pulsing_materials.add(ExtendedMaterial {
        base: StandardMaterial {
            base_color: game_settings.palette.player_color,
            base_color_texture: Some(level_symbol_sprite_sheet.clone()),
            emissive: LinearRgba::from_vec3(player_color.to_vec3() * 2.0),
            alpha_mode: ALPHA_MODE,
            ..Default::default()
        },
        extension: PulsingShader {},
    });

    let ready_easy_color = &game_settings.palette.face_colors.colors[0];
    let ready_hard_color = &game_settings.palette.face_colors.colors[3];

    let incomplete_face_colors = core::array::from_fn(|level_index| {
        let material =
            get_ready_selector_face_colors(level_index, ready_easy_color, ready_hard_color);
        materials.add(material)
    });

    let selector_handles = SelectorHandles {
        unavailable: materials.add(get_face_material_from_color(face_colors[4])),
        completed: materials.add(get_face_material_from_color(face_colors[2])),
        perfect_score: materials.add(get_face_material_from_color(face_colors[1])),
        melody_found: materials.add(get_face_material_from_color(
            game_settings.palette.player_color,
        )),
        selection_pressed,
        selection_hover,
        level_symbols,
        melody_found_selector_face,
        incomplete_face_colors,
    };

    commands.insert_resource(MaterialHandles {
        player_halo_handle,
        player_handle,
        line_handle: materials.add(line_material),
        bright_pulsing_line_handle,
        dashed_arrow_handle,
        bright_dashed_arrow_handle,
        bright_pulsing_dashed_arrow_handle,
        face_handles: FaceMaterialHandles { face_handles },
        selector: selector_handles,
        goal_handle,
    })
}

fn get_face_material_from_color(color: Color) -> StandardMaterial {
    StandardMaterial {
        base_color: color,
        reflectance: 0.0,
        perceptual_roughness: 1.0,
        alpha_mode: ALPHA_MODE,
        ..Default::default()
    }
}

fn get_ready_selector_face_colors(
    level_index: usize,
    ready_easy_color: &Color,
    ready_hard_color: &Color,
) -> StandardMaterial {
    let mix_factor = (level_index as f32) / (LEVELS.len() as f32);
    let color = ready_easy_color.mix(ready_hard_color, mix_factor);
    StandardMaterial {
        base_color: color,
        alpha_mode: ALPHA_MODE,
        ..Default::default()
    }
}
