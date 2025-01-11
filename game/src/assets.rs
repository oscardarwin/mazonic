use crate::game_settings::GameSettings;
use bevy::{
    pbr::{ExtendedMaterial, MaterialExtension},
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
};

pub struct FaceMaterialHandles {
    pub materials: [Handle<StandardMaterial>; 6],
}

impl FaceMaterialHandles {
    fn get_material(&self, index: usize) -> Handle<StandardMaterial> {
        self.materials[index].clone()
    }

    pub fn tetrahedron(&self) -> [Handle<StandardMaterial>; 4] {
        [
            self.get_material(0),
            self.get_material(1),
            self.get_material(2),
            self.get_material(3),
        ]
    }

    pub fn cube(&self) -> [Handle<StandardMaterial>; 6] {
        [
            self.get_material(0),
            self.get_material(1),
            self.get_material(1),
            self.get_material(2),
            self.get_material(2),
            self.get_material(0),
        ]
    }

    pub fn octahedron(&self) -> [Handle<StandardMaterial>; 8] {
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

    pub fn dodecahedron(&self) -> [Handle<StandardMaterial>; 12] {
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

    pub fn icosahedron(&self) -> [Handle<StandardMaterial>; 20] {
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

#[derive(Resource)]
pub struct GameAssetHandles {
    pub player_material: Handle<StandardMaterial>,
    pub line_material: Handle<StandardMaterial>,
    pub dashed_arrow_material: Handle<ExtendedMaterial<StandardMaterial, DashedArrowMaterial>>,
    pub face_materials: FaceMaterialHandles,
}

pub fn setup_game_assets(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut dashed_arrow_materials: ResMut<
        Assets<ExtendedMaterial<StandardMaterial, DashedArrowMaterial>>,
    >,
    asset_server: Res<AssetServer>,
    game_settings: Res<GameSettings>,
) {
    let player_material = materials.add(StandardMaterial::from_color(
        game_settings.palette.player_color,
    ));
    let line_material = materials.add(StandardMaterial::from_color(
        game_settings.palette.line_color,
    ));

    let dashed_arrow_material = dashed_arrow_materials.add(ExtendedMaterial {
        base: StandardMaterial {
            base_color: game_settings.palette.line_color,
            alpha_mode: AlphaMode::Blend,
            ..Default::default()
        },
        extension: DashedArrowMaterial {},
    });

    let face_materials = game_settings
        .palette
        .face_colors
        .colors
        .map(|color| materials.add(StandardMaterial::from_color(color)));

    commands.insert_resource(GameAssetHandles {
        player_material,
        line_material,
        dashed_arrow_material,
        face_materials: FaceMaterialHandles {
            materials: face_materials,
        },
    })
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct DashedArrowMaterial {}

impl MaterialExtension for DashedArrowMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/dashed_line.wgsl".into()
    }
}
