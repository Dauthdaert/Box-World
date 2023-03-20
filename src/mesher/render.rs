use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::{
        mesh::MeshVertexAttribute,
        render_resource::{AsBindGroup, VertexFormat},
    },
};
use bevy_asset_loader::prelude::*;

#[derive(AssetCollection, Resource)]
pub struct TerrainTexture {
    #[asset(path = "textures/terrain_texture.ktx2")]
    terrain_handle: Handle<Image>,
}

#[derive(Resource)]
pub struct TerrainMaterial {
    material_handle: Handle<TerrainTextureMaterial>,
}

impl FromWorld for TerrainMaterial {
    fn from_world(world: &mut World) -> Self {
        let cell = world.cell();
        let terrain_texture = cell
            .get_resource::<TerrainTexture>()
            .expect("Unable to get AssetServer");
        let mut materials = cell
            .get_resource_mut::<Assets<TerrainTextureMaterial>>()
            .expect("Unable to get Assets<TerrainTextureMaterial>");

        info!("Loading TerrainTextureMaterial");

        Self {
            material_handle: materials.add(TerrainTextureMaterial {
                terrain_texture: terrain_texture.terrain_handle.clone_weak(),
            }),
        }
    }
}

impl TerrainMaterial {
    pub fn handle(&self) -> &Handle<TerrainTextureMaterial> {
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
