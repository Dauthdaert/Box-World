use bevy::{
    pbr::StandardMaterialFlags,
    prelude::*,
    render::{mesh::MeshVertexAttribute, render_resource::*},
};
use bevy_asset_loader::prelude::*;

#[derive(AssetCollection, Resource)]
pub struct TerrainTexture {
    #[asset(path = "textures/terrain_texture.ktx2")]
    terrain_handle: Handle<Image>,
}

#[derive(Resource)]
pub struct TerrainMaterial {
    opaque_material: Handle<TerrainTextureMaterial>,
    transparent_material: Handle<TerrainTextureMaterial>,
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
            opaque_material: materials.add(TerrainTextureMaterial {
                terrain_texture: terrain_texture.terrain_handle.clone_weak(),
                ..default()
            }),
            transparent_material: materials.add(TerrainTextureMaterial {
                terrain_texture: terrain_texture.terrain_handle.clone_weak(),
                alpha_mode: AlphaMode::Blend,
                ..default()
            }),
        }
    }
}

impl TerrainMaterial {
    pub fn opaque(&self) -> &Handle<TerrainTextureMaterial> {
        &self.opaque_material
    }

    pub fn transparent(&self) -> &Handle<TerrainTextureMaterial> {
        &self.transparent_material
    }
}

#[derive(Clone, Default, ShaderType)]
pub struct TerrainTextureMaterialUniform {
    pub flags: u32,
    pub alpha_cutoff: f32,
}

impl From<&TerrainTextureMaterial> for TerrainTextureMaterialUniform {
    fn from(value: &TerrainTextureMaterial) -> Self {
        let mut flags = StandardMaterialFlags::NONE;

        if value.double_sided {
            flags |= StandardMaterialFlags::DOUBLE_SIDED;
        }

        let mut alpha_cutoff = 0.5;
        match value.alpha_mode {
            AlphaMode::Opaque => flags |= StandardMaterialFlags::ALPHA_MODE_OPAQUE,
            AlphaMode::Mask(c) => {
                alpha_cutoff = c;
                flags |= StandardMaterialFlags::ALPHA_MODE_MASK;
            }
            AlphaMode::Blend => flags |= StandardMaterialFlags::ALPHA_MODE_BLEND,
            AlphaMode::Premultiplied => flags |= StandardMaterialFlags::ALPHA_MODE_PREMULTIPLIED,
            AlphaMode::Add => flags |= StandardMaterialFlags::ALPHA_MODE_ADD,
            AlphaMode::Multiply => flags |= StandardMaterialFlags::ALPHA_MODE_MULTIPLY,
        };

        Self {
            flags: flags.bits(),
            alpha_cutoff,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct TerrainTextureMaterialKey {
    cull_mode: Option<Face>,
}

impl From<&TerrainTextureMaterial> for TerrainTextureMaterialKey {
    fn from(value: &TerrainTextureMaterial) -> Self {
        Self {
            cull_mode: value.cull_mode,
        }
    }
}

pub const ATTRIBUTE_VOXEL_INDICES: MeshVertexAttribute =
    MeshVertexAttribute::new("VoxelIndices", 987234876, VertexFormat::Uint32);
pub const ATTRIBUTE_VOXEL_LIGHTS: MeshVertexAttribute =
    MeshVertexAttribute::new("VoxelLights", 187234876, VertexFormat::Float32x2);

#[derive(AsBindGroup, Debug, Clone, Asset, TypePath)]
#[bind_group_data(TerrainTextureMaterialKey)]
#[uniform(0, TerrainTextureMaterialUniform)]
pub struct TerrainTextureMaterial {
    #[texture(1, dimension = "2d_array")]
    #[sampler(2)]
    terrain_texture: Handle<Image>,
    alpha_mode: AlphaMode,

    // FIXME: Back faces often have different lighting from front faces.
    // Really ugly result.
    cull_mode: Option<Face>,
    double_sided: bool,
}

impl Default for TerrainTextureMaterial {
    fn default() -> Self {
        Self {
            terrain_texture: Handle::default(),
            alpha_mode: AlphaMode::Opaque,
            cull_mode: Some(Face::Back),
            double_sided: false,
        }
    }
}

impl Material for TerrainTextureMaterial {
    fn vertex_shader() -> bevy::render::render_resource::ShaderRef {
        "shaders/chunk_vertex.wgsl".into()
    }

    fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
        "shaders/chunk_frag.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        self.alpha_mode
    }

    fn specialize(
        _pipeline: &bevy::pbr::MaterialPipeline<Self>,
        descriptor: &mut bevy::render::render_resource::RenderPipelineDescriptor,
        layout: &bevy::render::mesh::MeshVertexBufferLayout,
        key: bevy::pbr::MaterialPipelineKey<Self>,
    ) -> Result<(), bevy::render::render_resource::SpecializedMeshPipelineError> {
        descriptor.primitive.cull_mode = key.bind_group_data.cull_mode;

        let vertex_layout = layout.get_layout(&[
            Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
            Mesh::ATTRIBUTE_NORMAL.at_shader_location(1),
            Mesh::ATTRIBUTE_UV_0.at_shader_location(2),
            Mesh::ATTRIBUTE_COLOR.at_shader_location(3),
            ATTRIBUTE_VOXEL_INDICES.at_shader_location(4),
            ATTRIBUTE_VOXEL_LIGHTS.at_shader_location(5),
        ])?;
        descriptor.vertex.buffers = vec![vertex_layout];
        Ok(())
    }
}
