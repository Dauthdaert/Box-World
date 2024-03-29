use crate::{
    voxel::{GlobalVoxelPos, Voxel},
    world_generator::NeedsChunkData,
};
use bevy::app::AppExit;
use bevy::prelude::*;
use bevy::tasks::AsyncComputeTaskPool;
use rand::seq::IteratorRandom;

mod data;
mod database;
mod lighting;
mod loaded;
mod position;
mod storage;

pub use data::ChunkData;
pub use lighting::{to_sunlight, to_torchlight};
pub use loaded::{Database, LoadPoint, LoadedChunks};
pub use position::ChunkPos;

#[derive(Event)]
pub struct VoxelAddedEvent {
    pub pos: GlobalVoxelPos,
    pub value: Voxel,
}

impl VoxelAddedEvent {
    pub fn new(pos: GlobalVoxelPos, value: Voxel) -> Self {
        Self { pos, value }
    }
}

#[derive(Event)]
pub struct VoxelRemovedEvent {
    pub pos: GlobalVoxelPos,
}

impl VoxelRemovedEvent {
    pub fn new(pos: GlobalVoxelPos) -> Self {
        Self { pos }
    }
}

pub struct ChunkPlugin;

impl Plugin for ChunkPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(LoadedChunks::new())
            .insert_resource(Database::new())
            .insert_resource(AutosaveTimer::new());

        app.add_systems(Update, (periodic_chunk_trim, autosave_chunks))
            .add_systems(PreUpdate, load_around_load_points);

        app.add_systems(Last, save_chunks_on_close.run_if(on_event::<AppExit>()));

        app.add_event::<VoxelAddedEvent>()
            .add_event::<VoxelRemovedEvent>();
    }
}

fn periodic_chunk_trim(mut chunks: Query<&mut ChunkData>) {
    let mut rng = rand::thread_rng();
    for mut data in chunks
        .iter_mut()
        .filter(|data| !data.is_uniform())
        .choose_multiple(&mut rng, 256)
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
        let load_chunk_pos = ChunkPos::from_global_coords(load_translation);
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
        {
            let _span = info_span!("Unloading chunks").entered();
            let unloaded_chunks = world.unload_outside_range(&load_pos);
            let mut to_save = Vec::new();
            for entity in unloaded_chunks.iter() {
                if let Ok((chunk_pos, chunk_data)) = chunks.get(*entity) {
                    if chunk_data.is_dirty() {
                        to_save.push((*chunk_pos, chunk_data.to_raw()));
                    }
                }
                commands.entity(*entity).despawn_recursive();
            }

            if !to_save.is_empty() {
                let thread_pool = AsyncComputeTaskPool::get();
                let connection_lock = database.get_connection_pool();
                thread_pool
                    .spawn(async move {
                        info!("Saving {} unloaded chunks", to_save.len());
                        database::save_raw_chunks(&connection_lock, to_save);
                    })
                    .detach();
            }
        }

        {
            let _span = info_span!("Loading chunks").entered();
            let loaded_chunks = world.load_inside_range(&load_pos);
            for pos in loaded_chunks.into_iter() {
                if world.get_chunk(pos).is_none() {
                    let entity = commands
                        .spawn((pos, NeedsChunkData, Name::new("Chunk")))
                        .id();
                    world.set(pos, entity);
                }
            }
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
            .take(5000)
        {
            chunk_data.set_dirty(false);
            chunks_cloned.push((*chunk_pos, chunk_data.to_raw()));
        }

        let thread_pool = AsyncComputeTaskPool::get();
        let connection_lock = database.get_connection_pool();
        thread_pool
            .spawn(async move {
                database::save_raw_chunks(&connection_lock, chunks_cloned);
                info!("Done autosaving");
            })
            .detach();
    }
}

fn save_chunks_on_close(
    exit: EventReader<AppExit>,
    database: Res<Database>,
    chunks: Query<(&ChunkPos, &ChunkData)>,
) {
    if !exit.is_empty() {
        info!("Save on close");

        let connection_lock = database.get_connection_pool();
        database::save_raw_chunks(
            &connection_lock,
            chunks
                .iter()
                .filter_map(|(pos, data)| {
                    if data.is_dirty() {
                        Some((*pos, data.to_raw()))
                    } else {
                        None
                    }
                })
                .collect(),
        );
    }
}
