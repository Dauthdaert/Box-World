use serde::{Deserialize, Serialize};

pub const VOXEL_SIZE: f32 = 2.0;

pub const VOXEL_AIR: Voxel = Voxel::Empty;
pub const VOXEL_BEDROCK: Voxel = Voxel::Opaque(0);
pub const VOXEL_GRASS: Voxel = Voxel::Opaque(1);
pub const VOXEL_STONE: Voxel = Voxel::Opaque(5);

mod position;
pub use position::VoxelPos;

#[allow(dead_code)]
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
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

    pub fn indice(&self) -> u32 {
        match self {
            Voxel::Empty => u32::MAX,
            Voxel::Transparent(val) | Voxel::Opaque(val) => *val as u32,
        }
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
