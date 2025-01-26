use crate::game_settings::GameSettings;
use bevy::{pbr::ExtendedMaterial, prelude::*};

use super::shaders::{
    DashedArrowMaterial, MenuSelectionHoverMaterial, PlayerHaloMaterial, ShapeFaceMaterial,
};

pub struct FaceMaterialHandles {
    pub materials: [Handle<ExtendedMaterial<StandardMaterial, ShapeFaceMaterial>>; 6],
}

impl FaceMaterialHandles {
    fn get_material(
        &self,
        index: usize,
    ) -> Handle<ExtendedMaterial<StandardMaterial, ShapeFaceMaterial>> {
        self.materials[index].clone()
    }

    pub fn tetrahedron(
        &self,
    ) -> [Handle<ExtendedMaterial<StandardMaterial, ShapeFaceMaterial>>; 4] {
        [
            self.get_material(0),
            self.get_material(1),
            self.get_material(2),
            self.get_material(3),
        ]
    }

    pub fn cube(&self) -> [Handle<ExtendedMaterial<StandardMaterial, ShapeFaceMaterial>>; 6] {
        [
            self.get_material(0),
            self.get_material(1),
            self.get_material(1),
            self.get_material(2),
            self.get_material(2),
            self.get_material(0),
        ]
    }

    pub fn octahedron(&self) -> [Handle<ExtendedMaterial<StandardMaterial, ShapeFaceMaterial>>; 8] {
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

    pub fn dodecahedron(
        &self,
    ) -> [Handle<ExtendedMaterial<StandardMaterial, ShapeFaceMaterial>>; 12] {
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

    pub fn icosahedron(
        &self,
    ) -> [Handle<ExtendedMaterial<StandardMaterial, ShapeFaceMaterial>>; 20] {
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
    pub selection_pressed: Handle<ExtendedMaterial<StandardMaterial, MenuSelectionHoverMaterial>>,
    pub selection_hover: Handle<ExtendedMaterial<StandardMaterial, MenuSelectionHoverMaterial>>,
}

#[derive(Resource)]
pub struct MaterialHandles {
    pub player_halo_material: Handle<ExtendedMaterial<StandardMaterial, PlayerHaloMaterial>>,
    pub player_material: Handle<StandardMaterial>,
    pub line_material: Handle<StandardMaterial>,
    pub dashed_arrow_material: Handle<ExtendedMaterial<StandardMaterial, DashedArrowMaterial>>,
    pub bright_dashed_arrow_material:
        Handle<ExtendedMaterial<StandardMaterial, DashedArrowMaterial>>,
    pub face_materials: FaceMaterialHandles,
    pub selector_handles: SelectorMaterialHandles,
}

pub fn setup_materials(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut dashed_arrow_materials: ResMut<
        Assets<ExtendedMaterial<StandardMaterial, DashedArrowMaterial>>,
    >,
    mut player_halo_materials: ResMut<
        Assets<ExtendedMaterial<StandardMaterial, PlayerHaloMaterial>>,
    >,
    mut menu_selection_hover_materials: ResMut<
        Assets<ExtendedMaterial<StandardMaterial, MenuSelectionHoverMaterial>>,
    >,
    mut shape_face_materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, ShapeFaceMaterial>>>,
    asset_server: Res<AssetServer>,
    game_settings: Res<GameSettings>,
) {
    let bright_player_color = player_color.to_linear().to_vec3() * 2.0;
    let player_halo_material = player_halo_materials.add(ExtendedMaterial {
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
        extension: PlayerHaloMaterial {},
    });
    let player_color = &game_settings.palette.player_color;
    let player_material = materials.add(StandardMaterial {
        base_color: *player_color,
        emissive: (*player_color).into(),
        reflectance: 0.1,
        ..Default::default()
    });

    let line_color = &game_settings.palette.line_color;
    let line_color_vec = line_color.to_linear().to_vec3();
    let line_material_handle = materials.add(*line_color);

    let dashed_arrow_material = dashed_arrow_materials.add(ExtendedMaterial {
        base: StandardMaterial {
            base_color: *line_color,
            alpha_mode: AlphaMode::Blend,
            ..Default::default()
        },
        extension: DashedArrowMaterial {},
    });

    let bright_dashed_arrow_material = dashed_arrow_materials.add(ExtendedMaterial {
        base: StandardMaterial {
            base_color: *line_color,
            alpha_mode: AlphaMode::Blend,
            emissive: LinearRgba::from_vec3(line_color_vec * 20.0),
            ..Default::default()
        },
        extension: DashedArrowMaterial {},
    });

    let face_materials = game_settings.palette.face_colors.colors.map(|color| {
        shape_face_materials.add(ExtendedMaterial {
            base: StandardMaterial {
                base_color: color,
                reflectance: 0.0,
                perceptual_roughness: 1.0,
                ..Default::default()
            },
            extension: ShapeFaceMaterial {},
        })
    });

    let selection_hover = menu_selection_hover_materials.add(ExtendedMaterial {
        base: StandardMaterial {
            base_color: line_color.with_alpha(0.75),
            emissive: LinearRgba::from_vec3(line_color_vec * 20.0),
            alpha_mode: AlphaMode::Blend,
            ..Default::default()
        },
        extension: MenuSelectionHoverMaterial {},
    });
    let selection_pressed = menu_selection_hover_materials.add(ExtendedMaterial {
        base: StandardMaterial {
            base_color: line_color.with_alpha(0.9),
            emissive: LinearRgba::from_vec3(line_color_vec * 50.0),
            alpha_mode: AlphaMode::Blend,
            ..Default::default()
        },
        extension: MenuSelectionHoverMaterial {},
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
        player_halo_material,
        player_material,
        line_material: line_material_handle,
        dashed_arrow_material,
        bright_dashed_arrow_material,
        face_materials: FaceMaterialHandles {
            materials: face_materials,
        },
        selector_handles,
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
