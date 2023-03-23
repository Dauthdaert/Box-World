#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::mesh_bindings

#import bevy_pbr::pbr_types
#import bevy_pbr::utils
#import bevy_pbr::clustered_forward
#import bevy_pbr::lighting
#import bevy_pbr::shadows
#import bevy_pbr::fog
#import bevy_pbr::pbr_functions
#import bevy_pbr::pbr_ambient

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
    #import bevy_pbr::mesh_vertex_output

    @location(5) voxel_indice: u32
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

    // ==== Start PBR Boilerplate ====
    pbr_input.frag_coord = in.frag_coord;
    pbr_input.world_position = in.world_position;
    pbr_input.world_normal = prepare_world_normal(
        in.world_normal,
        (pbr_input.material.flags & STANDARD_MATERIAL_FLAGS_DOUBLE_SIDED_BIT) != 0u,
        in.is_front,
    );

    pbr_input.is_orthographic = view.projection[3].w == 1.0;

    pbr_input.N = apply_normal_mapping(
        pbr_input.material.flags,
        pbr_input.world_normal,
#ifdef VERTEX_TANGENTS
#ifdef STANDARDMATERIAL_NORMAL_MAP
        in.world_tangent,
#endif
#endif
        in.uv,
    );
    pbr_input.V = calculate_view(in.world_position, pbr_input.is_orthographic);

    var output_color =  pbr(pbr_input);

    // Apply fog
    if (fog.mode != FOG_MODE_OFF) {
        output_color = apply_fog(output_color, in.world_position.xyz, view.world_position.xyz);
    }

    output_color = tone_mapping(output_color);
    // ==== End PBR Boilerplate ====

    return output_color;
}
