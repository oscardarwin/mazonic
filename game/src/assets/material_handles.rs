use crate::game_settings::GameSettings;
use bevy::{
    pbr::{ExtendedMaterial, MaterialExtension},
    prelude::*,
};

use super::shaders::{
    DashedArrowShader, GlobalShader, MenuSelectionHoverShader, PlayerHaloShader, PulsingShader,
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

pub struct SelectorMaterialHandles {
    pub unavailable: Handle<StandardMaterial>,
    pub completed: Handle<StandardMaterial>,
    pub perfect_score: Handle<StandardMaterial>,
    pub melody_found: Handle<StandardMaterial>,
    pub selection_pressed: Handle<ExtendedMaterial<StandardMaterial, MenuSelectionHoverShader>>,
    pub selection_hover: Handle<ExtendedMaterial<StandardMaterial, MenuSelectionHoverShader>>,
}

#[derive(Resource)]
pub struct MaterialHandles {
    pub player_halo_handle: Handle<ExtendedMaterial<StandardMaterial, PlayerHaloShader>>,
    pub player_handle: Handle<StandardMaterial>,
    pub line_handle: Handle<StandardMaterial>,
    pub bright_line_handle: Handle<StandardMaterial>,
    pub dashed_arrow_handle: Handle<ExtendedMaterial<StandardMaterial, DashedArrowShader>>,
    pub bright_dashed_arrow_handle: Handle<ExtendedMaterial<StandardMaterial, DashedArrowShader>>,
    pub face_handles: FaceMaterialHandles,
    pub selector_handles: SelectorMaterialHandles,
    pub goal_handle: Handle<ExtendedMaterial<StandardMaterial, PulsingShader>>,
}

pub fn setup_materials(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut dashed_arrow_materials: ResMut<
        Assets<ExtendedMaterial<StandardMaterial, DashedArrowShader>>,
    >,
    mut player_halo_materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, PlayerHaloShader>>>,
    mut pulsing_materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, PulsingShader>>>,
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
            ..Default::default()
        },
        extension: PulsingShader {},
    });

    let player_color = &game_settings.palette.player_color;
    let bright_player_color = player_color.to_linear().to_vec3() * 2.0;
    let player_halo_handle = player_halo_materials.add(ExtendedMaterial {
        base: StandardMaterial {
            base_color: game_settings.palette.player_color,
            emissive: LinearRgba::from_vec3(bright_player_color),
            alpha_mode: AlphaMode::Blend,
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
        base_color: *player_color,
        emissive: (*player_color).into(),
        reflectance: 0.1,
        ..Default::default()
    });

    let line_color = &game_settings.palette.line_color;
    let line_color_vec = line_color.to_linear().to_vec3();
    let line_handle = materials.add(*line_color);

    let bright_line = StandardMaterial {
        base_color: *line_color,
        alpha_mode: AlphaMode::Blend,
        emissive: LinearRgba::from_vec3(line_color_vec * 20.0),
        ..Default::default()
    };

    let bright_line_handle = materials.add(bright_line.clone());

    let dashed_arrow_handle = dashed_arrow_materials.add(ExtendedMaterial {
        base: StandardMaterial {
            base_color: *line_color,
            alpha_mode: AlphaMode::Blend,
            ..Default::default()
        },
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
            alpha_mode: AlphaMode::Blend,
            ..Default::default()
        },
        extension: MenuSelectionHoverShader {},
    });
    let selection_pressed = menu_selection_hover_materials.add(ExtendedMaterial {
        base: StandardMaterial {
            base_color: line_color.with_alpha(0.9),
            emissive: LinearRgba::from_vec3(line_color_vec * 50.0),
            alpha_mode: AlphaMode::Blend,
            ..Default::default()
        },
        extension: MenuSelectionHoverShader {},
    });
    let face_colors = &game_settings.palette.face_colors.colors;
    let selector_handles = SelectorMaterialHandles {
        unavailable: materials.add(get_face_material_from_color(face_colors[4])),
        completed: materials.add(get_face_material_from_color(face_colors[2])),
        perfect_score: materials.add(get_face_material_from_color(face_colors[1])),
        melody_found: materials.add(game_settings.palette.player_color),
        selection_pressed,
        selection_hover,
    };

    commands.insert_resource(MaterialHandles {
        player_halo_handle,
        player_handle,
        line_handle,
        bright_line_handle,
        dashed_arrow_handle,
        bright_dashed_arrow_handle,
        face_handles: FaceMaterialHandles { face_handles },
        selector_handles,
        goal_handle,
    })
}

fn get_face_material_from_color(color: Color) -> StandardMaterial {
    StandardMaterial {
        base_color: color,
        reflectance: 0.0,
        perceptual_roughness: 1.0,
        ..Default::default()
    }
}
