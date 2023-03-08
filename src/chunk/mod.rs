use bevy::prelude::*;
use rand::seq::IteratorRandom;

mod data;
mod loaded;
mod position;
mod storage;

pub use data::CHUNK_EDGE;

pub use data::ChunkData;
pub use loaded::LoadedChunks;
pub use position::ChunkPos;

pub struct ChunkPlugin;

impl Plugin for ChunkPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(LoadedChunks::new());

        app.add_system(periodic_chunk_trim);
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
