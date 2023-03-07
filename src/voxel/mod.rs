const VOXEL_SIZE: f32 = 4.0;

mod position;
pub use position::VoxelPos;

#[allow(dead_code)]
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
pub enum Voxel {
    #[default]
    Empty,
    Transparent(u16),
    Opaque(u16),
}

impl Voxel {
    pub const fn size() -> f32 {
        VOXEL_SIZE
    }

    pub fn visibility(&self) -> crate::mesher::VoxelVisibility {
        use crate::mesher::VoxelVisibility;
        match self {
            Voxel::Empty => VoxelVisibility::Empty,
            Voxel::Transparent(_) => VoxelVisibility::Transparent,
            Voxel::Opaque(_) => VoxelVisibility::Opaque,
        }
    }
}
