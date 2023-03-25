use bevy::prelude::{App, Plugin};

mod data;
mod position;
mod registry;

use bevy_asset_loader::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
pub use data::Voxel;
pub use position::ChunkLocalVoxelPos;
pub use position::GlobalVoxelPos;
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
