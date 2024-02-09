#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::mesh_bindings mesh
#import bevy_pbr::mesh_vertex_output MeshVertexOutput
#import bevy_render::instance_index::get_instance_index

// NOTE: Bindings must come before functions that use them!
#import bevy_pbr::mesh_functions::mesh_normal_local_to_world;
#import bevy_pbr::mesh_functions::mesh_position_local_to_world;
#import bevy_pbr::mesh_functions::get_model_matrix;
#import bevy_pbr::view_transformations::position_world_to_clip;

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) color: vec4<f32>,
    @location(4) voxel_indice: u32,
    @location(5) voxel_light: vec2<f32>
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,

    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) color: vec4<f32>,
    @location(4) voxel_indice: u32,
    @location(5) voxel_light: vec2<f32>,
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;

    var model = get_model_matrix(vertex.instance_index);

    out.world_normal = mesh_normal_local_to_world(vertex.normal, get_instance_index(vertex.instance_index));

    out.world_position = mesh_position_local_to_world(model, vec4<f32>(vertex.position, 1.0));
    out.position = position_world_to_clip(out.world_position.xyz);
    out.uv = vertex.uv;

    out.color = vertex.color;
    out.voxel_indice = vertex.voxel_indice;
    out.voxel_light = vertex.voxel_light;

    return out;
}
