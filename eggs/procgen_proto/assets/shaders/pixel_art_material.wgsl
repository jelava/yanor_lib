#import bevy_pbr::{
    forward_io::{FragmentOutput, Vertex, VertexOutput},
    mesh_functions::{get_world_from_local, mesh_position_local_to_world, mesh_normal_local_to_world, mesh_tangent_local_to_world},
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::{alpha_discard, apply_pbr_lighting, main_pass_post_lighting_processing},
    view_transformations::position_world_to_clip,
};

@group(#{MATERIAL_BIND_GROUP}) @binding(50) var color_texture_array: texture_2d_array<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(51) var color_texture_array_sampler: sampler;

struct CustomVertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    @location(1) @interpolate(flat, either) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) texture_index: u32,
}

struct CustomVertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) @interpolate(flat, either) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) texture_index: u32,
}

@vertex
fn vertex(vertex: CustomVertex) -> CustomVertexOutput {
    let world_from_local = get_world_from_local(vertex.instance_index);

    var output: CustomVertexOutput;
    output.world_position = mesh_position_local_to_world(world_from_local, vec4(vertex.position, 1.0));
    output.position = position_world_to_clip(output.world_position.xyz);
    output.world_normal = mesh_normal_local_to_world(vertex.normal, vertex.instance_index);
    output.uv = vertex.uv;
    output.texture_index = vertex.texture_index;

    return output;
}

@fragment
fn fragment(
    vertex_output: CustomVertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    // make the customized vertex output look like regular bevy vertex output so i can reuse their stuff
    var standard_vertex_output: VertexOutput;
    standard_vertex_output.position = vertex_output.position;
    standard_vertex_output.world_position = vertex_output.world_position;
    standard_vertex_output.world_normal = vertex_output.world_normal;
    standard_vertex_output.uv = vertex_output.uv;

    var pbr_input = pbr_input_from_standard_material(standard_vertex_output, is_front);
    
    pbr_input.material.base_color = textureSample(
        color_texture_array,
        color_texture_array_sampler,
        vertex_output.uv,
        vertex_output.texture_index,
    );

    pbr_input.material.base_color = alpha_discard(pbr_input.material, pbr_input.material.base_color);

    var output: FragmentOutput;
    output.color = apply_pbr_lighting(pbr_input);
    output.color = main_pass_post_lighting_processing(pbr_input, output.color);

    // output.color = vec4(vertex_output.uv, 0.0, 1.0);

    return output;
}
