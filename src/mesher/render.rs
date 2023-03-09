use bevy::{
    asset::LoadState,
    prelude::*,
    reflect::TypeUuid,
    render::{
        mesh::MeshVertexAttribute,
        render_resource::{AsBindGroup, VertexFormat},
    },
};

const SINGLE_TEXTURE_SIZE: f32 = 32.0;

#[derive(Resource, Default)]
pub struct TerrainTexture {
    is_loaded: bool,
    texture_handle: Handle<Image>,
    material_handle: Handle<ArrayTextureMaterial>,
}

impl TerrainTexture {
    pub fn is_loaded(&self) -> bool {
        self.is_loaded
    }

    pub fn material_handle(&self) -> &Handle<ArrayTextureMaterial> {
        &self.material_handle
    }
}

pub const ATTRIBUTE_VOXEL_INDICES: MeshVertexAttribute =
    MeshVertexAttribute::new("VoxelIndices", 987234876, VertexFormat::Uint32);

#[derive(AsBindGroup, Debug, Clone, TypeUuid)]
#[uuid = "8033ab15-49da-4f1f-b2aa-ecda82927520"]
pub struct ArrayTextureMaterial {
    #[texture(0, dimension = "2d_array")]
    #[sampler(1)]
    array_texture: Handle<Image>,
}

impl Material for ArrayTextureMaterial {
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
        texture_handle: asset_server.load("textures/array_texture.png"),
        ..default()
    });
}

pub fn create_terrain_texture_array(
    asset_server: Res<AssetServer>,
    mut terrain_texture: ResMut<TerrainTexture>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<ArrayTextureMaterial>>,
) {
    if asset_server.get_load_state(terrain_texture.texture_handle.clone()) != LoadState::Loaded {
        return;
    }

    terrain_texture.is_loaded = true;
    let image = images.get_mut(&terrain_texture.texture_handle).unwrap();

    // Create new array texture
    let num_layers = (image.size().y / SINGLE_TEXTURE_SIZE) as u32;
    info!("Dectected {} layers in terrain texture.", num_layers);
    image.reinterpret_stacked_2d_as_array(num_layers);

    // Create material
    terrain_texture.material_handle = materials.add(ArrayTextureMaterial {
        array_texture: terrain_texture.texture_handle.clone(),
    });
}
