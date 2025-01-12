#import bevy_pbr::{
    mesh_view_bindings::globals,
    pbr_fragment::pbr_input_from_standard_material,
    forward_io::{VertexOutput, FragmentOutput},
    pbr_functions::{apply_pbr_lighting, alpha_discard, main_pass_post_lighting_processing},
}
#import noisy_bevy::simplex_noise_3d

@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool
) -> FragmentOutput {
    var pbr_input = pbr_input_from_standard_material(in, is_front);

    // alpha discard
    pbr_input.material.base_color = alpha_discard(pbr_input.material, pbr_input.material.base_color);

    var out: FragmentOutput;
    out.color = apply_pbr_lighting(pbr_input);
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
    
    let world_position = vec3(in.world_position.x, in.world_position.y, in.world_position.z);
    let noise_value = simplex_noise_3d(world_position + vec3(5.0 * sin(0.05 * globals.time)));
    let sigmoid_noise_value = 0.5 + (atan(1000 * (noise_value - 0.5)) / 3.1416 / 4.0);

    out.color.a *= noise_value;

    return out;
}
