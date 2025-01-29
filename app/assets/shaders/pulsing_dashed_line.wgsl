#import bevy_pbr::{
    mesh_view_bindings::globals,
    pbr_fragment::pbr_input_from_standard_material,
    forward_io::{VertexOutput, FragmentOutput},
    pbr_functions::{apply_pbr_lighting, alpha_discard, main_pass_post_lighting_processing},
}

@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool
) -> FragmentOutput {
    var pbr_input = pbr_input_from_standard_material(in, is_front);
  
    let cos_wave = cos(1.6 * globals.time) + 1.0;
    let emissive_factor = pow(0.7 * cos_wave, 4.0);
    pbr_input.material.emissive.x = emissive_factor * pbr_input.material.base_color.x;
    pbr_input.material.emissive.y = emissive_factor * pbr_input.material.base_color.y;
    pbr_input.material.emissive.z = emissive_factor * pbr_input.material.base_color.z;

    // alpha discard
    pbr_input.material.base_color = alpha_discard(pbr_input.material, pbr_input.material.base_color);

    var out: FragmentOutput;
    // apply lighting
    out.color = apply_pbr_lighting(pbr_input);

    // apply in-shader post processing (fog, alpha-premultiply, and also tonemapping, debanding if the camera is non-hdr)
    // note this does not include fullscreen postprocessing effects like bloom.
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
    
    let sine_sample = sin(40.0 * in.uv.y - globals.time);
    let arrow_chunk = floor(1.5 * (sine_sample + 1.0));
    let in_arrow_head = floor(in.uv.y + 0.5);
    out.color = out.color * max(min(arrow_chunk, 1.0), in_arrow_head);

    return out;
}
