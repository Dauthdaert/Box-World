use std::sync::Arc;

use bevy::{prelude::*, utils::HashMap};
use bevy_asset_loader::prelude::*;

use super::Voxel;

#[derive(AssetCollection, Resource)]
pub struct VoxelDataAssets {
    #[asset(path = "data/blocks", collection(typed, mapped))]
    data_mapped: HashMap<String, Handle<Voxel>>,
}

#[derive(Resource, Clone)]
pub struct VoxelRegistry {
    correspondance: Arc<HashMap<String, Voxel>>,
}

impl FromWorld for VoxelRegistry {
    fn from_world(world: &mut World) -> Self {
        let voxels = world
            .get_resource::<Assets<Voxel>>()
            .expect("Failed to get Assets<Voxel>");
        let voxel_assets = world
            .get_resource::<VoxelDataAssets>()
            .expect("Failed to get VoxelDataAssets");

        let correspondance: HashMap<String, Voxel> = voxel_assets
            .data_mapped
            .iter()
            .map(|(key, value)| {
                let mut key = key.as_str();
                key = key.split('.').collect::<Vec<_>>()[0];
                key = key.split('\\').last().unwrap();
                key = key.split('/').last().unwrap();
                (key.to_string(), *voxels.get(value).unwrap())
            })
            .collect();

        dbg!(&correspondance);

        Self {
            correspondance: Arc::new(correspondance),
        }
    }
}

impl VoxelRegistry {
    pub fn get_voxel(&self, name: &str) -> Voxel {
        if let Some(voxel) = self.correspondance.get(name) {
            *voxel
        } else {
            panic!(
                "Failed to acess Voxel with name {}. Name doesn't exist",
                name
            );
        }
    }
}
