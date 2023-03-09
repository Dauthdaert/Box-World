use crate::mesher::NeedsMesh;
use crate::world_generator::NeedsChunkData;
use crate::{HORIZONTAL_VIEW_DISTANCE, VERTICAL_VIEW_DISTANCE};
use bevy::prelude::*;
use rand::seq::IteratorRandom;

mod data;
mod loaded;
mod position;
mod storage;

pub use data::CHUNK_EDGE;

pub use data::ChunkData;
pub use loaded::LoadPoint;
pub use loaded::LoadedChunks;
pub use position::ChunkPos;

pub struct ChunkPlugin;

impl Plugin for ChunkPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(LoadedChunks::new());

        app.add_system(periodic_chunk_trim)
            .add_system(load_around_load_points.in_base_set(CoreSet::PreUpdate));
    }
}

fn periodic_chunk_trim(mut chunks: Query<&mut ChunkData>) {
    let mut rng = rand::thread_rng();
    for mut data in chunks
        .iter_mut()
        .filter(|data| !data.is_uniform())
        .choose_multiple(&mut rng, 2)
    {
        data.trim();
    }
}

fn load_around_load_points(
    mut commands: Commands,
    mut world: ResMut<LoadedChunks>,
    load_query: Query<&Transform, With<loaded::LoadPoint>>,
) {
    let mut load_pos = Vec::new();
    for camera_transform in load_query.iter() {
        let load_translation = camera_transform.translation;
        let load_chunk_pos = ChunkPos::from_global_coords(
            load_translation.x,
            load_translation.y,
            load_translation.z,
        );
        load_pos.push(load_chunk_pos);
    }

    let unloaded_chunks = world.unload_outside_range(&load_pos, HORIZONTAL_VIEW_DISTANCE);
    for entity in unloaded_chunks.iter() {
        commands.entity(*entity).despawn_recursive();
    }

    let loaded_chunks =
        world.load_inside_range(&load_pos, HORIZONTAL_VIEW_DISTANCE, VERTICAL_VIEW_DISTANCE);
    for (pos, chunk) in loaded_chunks.into_iter() {
        let entity = if let Some(chunk) = chunk {
            commands.spawn((pos, chunk, NeedsMesh)).id()
        } else {
            commands.spawn((pos, NeedsChunkData)).id()
        };
        world.set(pos, entity);
    }
}
