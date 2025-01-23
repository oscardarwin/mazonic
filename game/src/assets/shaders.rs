use bevy::{
    pbr::MaterialExtension,
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
};

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

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct MenuSelectionHoverMaterial {}

impl MaterialExtension for MenuSelectionHoverMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/selection_hover.wgsl".into()
    }
}
