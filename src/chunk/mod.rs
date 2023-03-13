use rusqlite::params;
use rusqlite::Connection;
use std::io::Cursor;
use zstd::stream::copy_encode;

use crate::world_generator::NeedsChunkData;
use bevy::app::AppExit;
use bevy::prelude::*;
use bevy::tasks::AsyncComputeTaskPool;
use rand::seq::IteratorRandom;

mod data;
mod loaded;
mod position;
mod storage;

pub use data::{ChunkData, CHUNK_EDGE};
pub use loaded::{Database, LoadPoint, LoadedChunks};
pub use position::ChunkPos;

use data::RawChunk;

pub struct ChunkPlugin;

impl Plugin for ChunkPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(LoadedChunks::new())
            .insert_resource(Database::new())
            .insert_resource(AutosaveTimer::new());

        app.add_system(periodic_chunk_trim)
            .add_system(autosave_chunks)
            .add_system(load_around_load_points.in_base_set(CoreSet::PreUpdate));

        app.add_system(
            save_chunks_on_close
                .run_if(on_event::<AppExit>())
                .in_base_set(CoreSet::Last),
        );
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
    database: Res<Database>,
    load_query: Query<(Ref<Transform>, &LoadPoint)>,
    chunks: Query<(&ChunkPos, &ChunkData)>,
) {
    let mut load_pos = Vec::new();
    for (load_transform, load_distance) in load_query.iter() {
        let load_translation = load_transform.translation;
        let load_chunk_pos = ChunkPos::from_global_coords(
            load_translation.x,
            load_translation.y,
            load_translation.z,
        );
        load_pos.push((
            load_chunk_pos,
            load_distance.horizontal,
            load_distance.vertical,
        ));
    }

    if load_query
        .iter()
        .any(|(transform, _distance)| transform.is_changed())
    {
        let thread_pool = AsyncComputeTaskPool::get();
        let unloaded_chunks = world.unload_outside_range(&load_pos);
        for entity in unloaded_chunks.iter() {
            if let Ok((chunk_pos, chunk_data)) = chunks.get(*entity) {
                let connection_lock = database.get_connection();
                let chunk_pos = *chunk_pos;
                let chunk_data = chunk_data.to_raw();
                thread_pool
                    .spawn(async move {
                        save_chunk(&connection_lock.lock().unwrap(), &chunk_pos, &chunk_data);
                    })
                    .detach();
            }
            commands.entity(*entity).despawn_recursive();
        }

        let loaded_chunks = world.load_inside_range(&load_pos);
        for pos in loaded_chunks.into_iter() {
            let entity = commands.spawn((pos, NeedsChunkData)).id();
            world.set(pos, entity);
        }
    }
}

#[derive(Resource)]
struct AutosaveTimer(pub Timer);

impl AutosaveTimer {
    fn new() -> Self {
        Self(Timer::from_seconds(60.0, TimerMode::Repeating))
    }
}

fn autosave_chunks(
    database: Res<Database>,
    time: Res<Time>,
    mut autosave_timer: ResMut<AutosaveTimer>,
    mut chunks: Query<(&ChunkPos, &mut ChunkData)>,
) {
    if autosave_timer.0.tick(time.delta()).just_finished() {
        info!("Starting autosave");

        // Autosave needs to be fast, so we only save a small batch of chunks at a time
        let mut chunks_cloned = Vec::new();
        for (chunk_pos, mut chunk_data) in chunks
            .iter_mut()
            .filter(|(_pos, data)| data.is_dirty())
            .take(20000)
        {
            chunk_data.set_dirty(false);
            chunks_cloned.push((*chunk_pos, chunk_data.to_raw()));
        }

        let thread_pool = AsyncComputeTaskPool::get();
        let connection_lock = database.get_connection();
        thread_pool
            .spawn(async move {
                // Save all chunks in a single transaction
                let connection = connection_lock.lock().unwrap();
                connection.execute("BEGIN;", []).unwrap();
                for (chunk_pos, chunk_data) in chunks_cloned.iter() {
                    save_chunk(&connection, chunk_pos, chunk_data);
                }
                connection.execute("COMMIT;", []).unwrap();

                info!("Done autosaving");
            })
            .detach();
    }
}

fn save_chunks_on_close(
    exit: EventReader<AppExit>,
    database: Res<Database>,
    mut chunks: Query<(&ChunkPos, &mut ChunkData)>,
) {
    if !exit.is_empty() {
        info!("Save on close");

        // Save all chunks in a single transaction
        let connection_lock = database.get_connection();
        let connection = connection_lock.lock().unwrap();
        connection.execute("BEGIN;", []).unwrap();
        for (chunk_pos, mut chunk_data) in chunks.iter_mut().filter(|(_pos, data)| data.is_dirty())
        {
            chunk_data.set_dirty(false);
            save_chunk(&connection, chunk_pos, &chunk_data.to_raw());
        }
        connection.execute("COMMIT;", []).unwrap();
    }
}

fn save_chunk(connection: &Connection, chunk_pos: &ChunkPos, chunk_data: &RawChunk) {
    if let Ok(raw_chunk_bin) = bincode::serialize(chunk_data) {
        let mut final_chunk = Cursor::new(raw_chunk_bin);
        let mut output = Cursor::new(Vec::new());
        copy_encode(&mut final_chunk, &mut output, 0).unwrap();
        connection
            .execute(
                "REPLACE INTO blocks (posx, posy, posz, data) values (?1, ?2, ?3, ?4)",
                params![
                    chunk_pos.x,
                    chunk_pos.y,
                    chunk_pos.z,
                    &output.get_ref().clone(),
                ],
            )
            .unwrap();
    }
}
