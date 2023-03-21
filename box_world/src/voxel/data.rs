use bevy::reflect::TypeUuid;
use serde::{Deserialize, Serialize};

use super::VOXEL_SIZE;

#[allow(dead_code)]
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Serialize, Deserialize, TypeUuid)]
#[uuid = "2f63c7be-0955-40b0-8b5f-845a5f3eba9a"]
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
