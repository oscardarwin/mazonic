use bevy::{
    pbr::{ExtendedMaterial, MaterialExtension},
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
};

#[derive(Default)]
pub struct ShadersPlugin;

impl Plugin for ShadersPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            MaterialPlugin::<ExtendedMaterial<StandardMaterial, DashedArrowShader>>::default(),
            MaterialPlugin::<ExtendedMaterial<StandardMaterial, PlayerHaloShader>>::default(),
            MaterialPlugin::<ExtendedMaterial<StandardMaterial, GlobalShader>>::default(),
            MaterialPlugin::<ExtendedMaterial<StandardMaterial, MenuSelectionHoverShader>>::default(
            ),
            MaterialPlugin::<ExtendedMaterial<StandardMaterial, PulsingShader>>::default(),
            MaterialPlugin::<ExtendedMaterial<StandardMaterial, PulsingDashedArrowShader>>::default(
            ),
            UiMaterialPlugin::<FlashUiMaterial>::default(),
        ));
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct DashedArrowShader {}

impl MaterialExtension for DashedArrowShader {
    fn fragment_shader() -> ShaderRef {
        "shaders/dashed_line.wgsl".into()
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct PlayerHaloShader {}

impl MaterialExtension for PlayerHaloShader {
    fn fragment_shader() -> ShaderRef {
        "shaders/halo.wgsl".into()
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct GlobalShader {}

impl MaterialExtension for GlobalShader {
    fn fragment_shader() -> ShaderRef {
        "shaders/global.wgsl".into()
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct MenuSelectionHoverShader {}

impl MaterialExtension for MenuSelectionHoverShader {
    fn fragment_shader() -> ShaderRef {
        "shaders/selection_hover.wgsl".into()
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct PulsingShader {}

impl MaterialExtension for PulsingShader {
    fn fragment_shader() -> ShaderRef {
        "shaders/pulsing.wgsl".into()
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct PulsingDashedArrowShader {}

impl MaterialExtension for PulsingDashedArrowShader {
    fn fragment_shader() -> ShaderRef {
        "shaders/pulsing_dashed_line.wgsl".into()
    }
}

#[derive(AsBindGroup, Asset, TypePath, Debug, Clone)]
pub struct FlashUiMaterial {
    #[uniform(0)]
    pub color: Vec4,
    #[texture(1)]
    #[sampler(2)]
    pub color_texture: Handle<Image>,
    #[uniform(3)]
    pub start_uv: Vec2,
    #[uniform(4)]
    pub end_uv: Vec2,
}

impl UiMaterial for FlashUiMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/ui_flash.wgsl".into()
    }
}
