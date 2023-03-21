use bevy::prelude::{App, Plugin};

pub const VOXEL_SIZE: f32 = 2.0;

mod data;
mod position;
mod registry;

use bevy_asset_loader::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
pub use data::Voxel;
pub use position::VoxelPos;
pub use registry::VoxelRegistry;

use crate::states::GameStates;

pub struct VoxelPlugin;

impl Plugin for VoxelPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(RonAssetPlugin::<Voxel>::new(&["voxel.ron"]));

        app.add_collection_to_loading_state::<_, registry::VoxelDataAssets>(
            GameStates::AssetLoading,
        )
        .init_resource_after_loading_state::<_, registry::VoxelRegistry>(GameStates::AssetLoading);
    }
}