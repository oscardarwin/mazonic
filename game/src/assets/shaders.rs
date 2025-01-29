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
