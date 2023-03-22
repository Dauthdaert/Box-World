use bevy::reflect::TypeUuid;
use serde::{Deserialize, Serialize};

use crate::mesher::VoxelVisibility;

use super::VOXEL_SIZE;

#[allow(dead_code)]
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Serialize, Deserialize, TypeUuid)]
#[uuid = "2f63c7be-0955-40b0-8b5f-845a5f3eba9a"]
pub struct Voxel {
    visibility: VoxelVisibility,
    texture_id: u16,
}

impl Voxel {
    pub const fn size() -> f32 {
        VOXEL_SIZE
    }

    pub fn is_empty(&self) -> bool {
        self.visibility == VoxelVisibility::Empty
    }

    pub fn indice(&self) -> u32 {
        self.texture_id as u32
    }

    pub fn visibility(&self) -> VoxelVisibility {
        self.visibility
    }
}
