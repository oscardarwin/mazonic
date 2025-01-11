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
        // generate a PbrInput struct from the StandardMaterial bindings
    var pbr_input = pbr_input_from_standard_material(in, is_front);

    // we can optionally modify the input before lighting and alpha_discard is applied
    pbr_input.material.base_color.b = pbr_input.material.base_color.r;

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

    //out.color = out.color * 2.0;

    return out;
}
