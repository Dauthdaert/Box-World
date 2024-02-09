#import bevy_pbr::{
    mesh_view_bindings::view,
    pbr_types::{STANDARD_MATERIAL_FLAGS_DOUBLE_SIDED_BIT, PbrInput, pbr_input_new},
    pbr_functions as fns,
}
#import bevy_core_pipeline::tonemapping::tone_mapping

struct TerrainTextureMaterial {
    flags: u32,
    alpha_cutoff: f32,
}

@group(1) @binding(0)
var<uniform> material: TerrainTextureMaterial;
@group(1) @binding(1)
var terrain_texture: texture_2d_array<f32>;
@group(1) @binding(2)
var terrain_texture_sampler: sampler;

struct FragmentInput {
    @builtin(front_facing) is_front: bool,
    @builtin(position) frag_coord: vec4<f32>,

    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) color: vec4<f32>,
    @location(4) voxel_indice: u32,
    @location(5) voxel_light: vec2<f32>,
};

@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {
    // Start with a tuned StandardMaterial
    var pbr_input: PbrInput = pbr_input_new();
    pbr_input.material.metallic = 0.0;
    pbr_input.material.reflectance = 0.0;
    pbr_input.material.perceptual_roughness = 0.9;

    pbr_input.material.flags = material.flags;
    pbr_input.material.alpha_cutoff = material.alpha_cutoff;

    // Get color form array texture
    pbr_input.material.base_color = textureSample(terrain_texture, terrain_texture_sampler, in.uv, i32(in.voxel_indice));

    // Apply baked Ambient Occlusion
    pbr_input.material.base_color = pbr_input.material.base_color * in.color;

    // Apply baked lighting
    // FIXME : Make baked lighting not make everything white when combined with light
    let light_intensity: f32 = min(in.voxel_light.x + in.voxel_light.y, 7.5);
    pbr_input.material.base_color = pbr_input.material.base_color * light_intensity;

    // ==== Start PBR Boilerplate ====
    let double_sided = (pbr_input.material.flags & STANDARD_MATERIAL_FLAGS_DOUBLE_SIDED_BIT) != 0u;

    pbr_input.frag_coord = in.frag_coord;
    pbr_input.world_position = in.world_position;
    pbr_input.world_normal = fns::prepare_world_normal(
        in.world_normal,
        double_sided,
        in.is_front,
    );

    pbr_input.is_orthographic = view.projection[3].w == 1.0;

    pbr_input.N = fns::apply_normal_mapping(
        pbr_input.material.flags,
        pbr_input.world_normal,
        double_sided,
        in.is_front,
        in.uv,
        view.mip_bias,
    );
    pbr_input.V = fns::calculate_view(in.world_position, pbr_input.is_orthographic);

    var output_color = fns::apply_pbr_lighting(pbr_input);

    // Apply fog
    //if (bevy_pbr::fog.mode != FOG_MODE_OFF) {
    //    output_color = apply_fog(output_color, in.world_position.xyz, view.world_position.xyz);
    //}

    output_color = tone_mapping(output_color, view.color_grading);
    // ==== End PBR Boilerplate ====

    return output_color;
}
