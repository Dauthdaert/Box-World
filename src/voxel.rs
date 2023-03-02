pub const VOXEL_SIZE: f32 = 1.0;

#[allow(dead_code)]
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
pub enum Voxel {
    #[default]
    Empty,
    Transparent(u16),
    Opaque(u16),
}

impl block_mesh::Voxel for Voxel {
    fn get_visibility(&self) -> block_mesh::VoxelVisibility {
        match self {
            Voxel::Empty => block_mesh::VoxelVisibility::Empty,
            Voxel::Transparent(_) => block_mesh::VoxelVisibility::Translucent,
            Voxel::Opaque(_) => block_mesh::VoxelVisibility::Opaque,
        }
    }
}

impl block_mesh::MergeVoxel for Voxel {
    type MergeValue = Self;
    type MergeValueFacingNeighbour = Self;

    fn merge_value(&self) -> Self::MergeValue {
        *self
    }

    fn merge_value_facing_neighbour(&self) -> Self::MergeValueFacingNeighbour {
        *self
    }
}
