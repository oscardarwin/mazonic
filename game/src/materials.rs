use crate::game_settings::GameSettings;
use bevy::{
    pbr::{ExtendedMaterial, MaterialExtension},
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
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
    pub selection_pressed: Handle<StandardMaterial>,
    pub selection_hover: Handle<StandardMaterial>,
}

#[derive(Resource)]
pub struct GameMaterialHandles {
    pub player_halo_material: Handle<ExtendedMaterial<StandardMaterial, PlayerHaloMaterial>>,
    pub player_material: Handle<StandardMaterial>,
    pub line_material: Handle<StandardMaterial>,
    pub dashed_arrow_material: Handle<ExtendedMaterial<StandardMaterial, DashedArrowMaterial>>,
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
    mut shape_face_materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, ShapeFaceMaterial>>>,
    asset_server: Res<AssetServer>,
    game_settings: Res<GameSettings>,
) {
    let player_color = &game_settings.palette.player_color;
    let player_material = materials.add(StandardMaterial {
        base_color: *player_color,
        emissive: (*player_color).into(),
        reflectance: 0.1,
        ..Default::default()
    });
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

    let face_materials = game_settings.palette.face_colors.colors.map(|color| {
        shape_face_materials.add(ExtendedMaterial {
            base: StandardMaterial {
                base_color: color,
                reflectance: 0.0,
                ..Default::default()
            },
            extension: ShapeFaceMaterial {},
        })
    });

    let bright_player_color = player_color.to_linear().to_vec3() * 2.0;
    let player_halo_material = player_halo_materials.add(ExtendedMaterial {
        base: StandardMaterial {
            base_color: game_settings.palette.player_color,
            emissive: LinearRgba::from_vec3(bright_player_color),
            alpha_mode: AlphaMode::Blend,
            diffuse_transmission: 1.0,
            //attenuation_distance: 10.0,
            thickness: 0.17,
            metallic: 0.2,
            fog_enabled: true,
            double_sided: true,
            ..Default::default()
        },
        extension: PlayerHaloMaterial {},
    });

    let selection_hover = materials.add(StandardMaterial {
        base_color: Color::WHITE.with_alpha(0.2),
        alpha_mode: AlphaMode::Blend,
        ..Default::default()
    });
    let selection_click = materials.add(StandardMaterial {
        base_color: Color::WHITE.with_alpha(0.9),
        alpha_mode: AlphaMode::Blend,
        ..Default::default()
    });

    let selector_handles = SelectorMaterialHandles {
        unavailable: materials.add(game_settings.palette.face_colors.colors[4]),
        completed: materials.add(game_settings.palette.face_colors.colors[2]),
        perfect_score: materials.add(game_settings.palette.face_colors.colors[1]),
        melody_found: materials.add(game_settings.palette.player_color),
        selection_pressed: selection_click,
        selection_hover,
    };

    // This could just be an entity with lots of little components.
    commands.insert_resource(GameMaterialHandles {
        player_halo_material,
        player_material,
        line_material,
        dashed_arrow_material,
        face_materials: FaceMaterialHandles {
            materials: face_materials,
        },
        selector_handles,
    })
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct DashedArrowMaterial {}

impl MaterialExtension for DashedArrowMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/dashed_line.wgsl".into()
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct PlayerHaloMaterial {}

impl MaterialExtension for PlayerHaloMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/halo.wgsl".into()
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct ShapeFaceMaterial {}

impl MaterialExtension for ShapeFaceMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/face_material.wgsl".into()
    }
}
