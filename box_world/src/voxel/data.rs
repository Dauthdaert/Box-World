use bevy::{
    asset::Asset,
    reflect::{TypePath, TypeUuid},
};
use serde::{Deserialize, Serialize};

use crate::mesher::VoxelVisibility;

#[allow(dead_code)]
#[derive(
    Debug, Copy, Clone, Default, PartialEq, Eq, Serialize, Deserialize, TypeUuid, Asset, TypePath,
)]
#[uuid = "2f63c7be-0955-40b0-8b5f-845a5f3eba9a"]
pub struct Voxel {
    visibility: VoxelVisibility,
    texture_id: u16,
    emissiveness: u8,
}

impl Voxel {
    pub fn is_empty(&self) -> bool {
        self.visibility == VoxelVisibility::Empty
    }

    pub fn is_opaque(&self) -> bool {
        self.visibility == VoxelVisibility::Opaque
    }

    pub fn indice(&self) -> u32 {
        self.texture_id as u32
    }

    pub fn visibility(&self) -> VoxelVisibility {
        self.visibility
    }

    pub fn emissiveness(&self) -> u8 {
        self.emissiveness
    }
}
