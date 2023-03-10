use bevy::{
    asset::LoadState,
    prelude::*,
    reflect::TypeUuid,
    render::{
        mesh::MeshVertexAttribute,
        render_resource::{AsBindGroup, VertexFormat},
    },
};

#[derive(Resource, Default)]
pub struct TerrainTexture {
    is_loaded: bool,
    texture_handle: Handle<Image>,
    material_handle: Handle<TerrainTextureMaterial>,
}

impl TerrainTexture {
    pub fn is_loaded(&self) -> bool {
        self.is_loaded
    }

    pub fn material_handle(&self) -> &Handle<TerrainTextureMaterial> {
        &self.material_handle
    }
}

pub const ATTRIBUTE_VOXEL_INDICES: MeshVertexAttribute =
    MeshVertexAttribute::new("VoxelIndices", 987234876, VertexFormat::Uint32);

#[derive(AsBindGroup, Debug, Clone, TypeUuid)]
#[uuid = "8033ab15-49da-4f1f-b2aa-ecda82927520"]
pub struct TerrainTextureMaterial {
    #[texture(0, dimension = "2d_array")]
    #[sampler(1)]
    terrain_texture: Handle<Image>,
}

impl Material for TerrainTextureMaterial {
    fn vertex_shader() -> bevy::render::render_resource::ShaderRef {
        "shaders/chunk_vertex.wgsl".into()
    }

    fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
        "shaders/chunk_frag.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Opaque
    }

    fn specialize(
        _pipeline: &bevy::pbr::MaterialPipeline<Self>,
        descriptor: &mut bevy::render::render_resource::RenderPipelineDescriptor,
        layout: &bevy::render::mesh::MeshVertexBufferLayout,
        _key: bevy::pbr::MaterialPipelineKey<Self>,
    ) -> Result<(), bevy::render::render_resource::SpecializedMeshPipelineError> {
        let vertex_layout = layout.get_layout(&[
            Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
            Mesh::ATTRIBUTE_NORMAL.at_shader_location(1),
            Mesh::ATTRIBUTE_UV_0.at_shader_location(2),
            // Pos 3: TANGENT
            Mesh::ATTRIBUTE_COLOR.at_shader_location(4),
            // Pos 5: JOINT_INDEX
            // Pos 6: JOINT_HEIGHT
            ATTRIBUTE_VOXEL_INDICES.at_shader_location(7),
        ])?;
        descriptor.vertex.buffers = vec![vertex_layout];
        Ok(())
    }
}

pub fn load_terrain_texture(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(TerrainTexture {
        is_loaded: false,
        texture_handle: asset_server.load("textures/terrain_texture.ktx2"),
        ..default()
    });
}

pub fn create_terrain_texture_material(
    asset_server: Res<AssetServer>,
    mut terrain_texture: ResMut<TerrainTexture>,
    mut materials: ResMut<Assets<TerrainTextureMaterial>>,
) {
    if asset_server.get_load_state(terrain_texture.texture_handle.clone()) != LoadState::Loaded {
        return;
    }

    terrain_texture.is_loaded = true;

    // Create material
    terrain_texture.material_handle = materials.add(TerrainTextureMaterial {
        terrain_texture: terrain_texture.texture_handle.clone(),
    });
}
