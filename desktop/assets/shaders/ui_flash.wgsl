#import bevy_ui::ui_vertex_output::UiVertexOutput
#import bevy_render::globals::Globals

@group(0) @binding(1) var<uniform> globals: Globals;

@group(1) @binding(0) var<uniform> color: vec4<f32>;
@group(1) @binding(1) var material_color_texture: texture_2d<f32>;
@group(1) @binding(2) var material_color_sampler: sampler;
@group(1) @binding(3) var<uniform> start_uv: vec2<f32>;
@group(1) @binding(4) var<uniform> end_uv: vec2<f32>;

@fragment
fn fragment(
  in: UiVertexOutput
) -> @location(0) vec4<f32> {
    let image_uv = start_uv + (end_uv - start_uv) * in.uv;

    let flash_sine = sin(image_uv.x - image_uv.y - 1.4 * globals.time);

    let flash_color = select(vec4(1.0, 1.0, 1.0, color.a), color, flash_sine < 0.998);

    return textureSample(material_color_texture, material_color_sampler, image_uv) * flash_color;
} 
