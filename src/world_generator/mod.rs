use bevy::{
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task},
};
use futures_lite::future;
use noise::{MultiFractal, NoiseFn, OpenSimplex, RidgedMulti};
use std::io::Cursor;
use zstd::stream::copy_decode;

use crate::{
    chunk::{ChunkData, ChunkPos, Database, LoadedChunks},
    mesher::NeedsMesh,
    voxel::{VoxelPos, VOXEL_AIR, VOXEL_BEDROCK, VOXEL_GRASS, VOXEL_STONE},
};

pub struct GeneratorPlugin;

impl Plugin for GeneratorPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system(enqueue_chunk_generation_tasks);

        app.add_system(handle_done_generation_tasks.in_base_set(CoreSet::PostUpdate));
    }
}

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct NeedsChunkData;

#[derive(Component)]
struct ComputeChunkData(Task<(Entity, ChunkPos, ChunkData)>);

fn enqueue_chunk_generation_tasks(
    mut commands: Commands,
    database: Res<Database>,
    needs_generation: Query<(Entity, &ChunkPos), With<NeedsChunkData>>,
) {
    if needs_generation.is_empty() {
        return;
    }

    let thread_pool = AsyncComputeTaskPool::get();
    let noise: RidgedMulti<OpenSimplex> =
        RidgedMulti::new(RidgedMulti::<OpenSimplex>::DEFAULT_SEED)
            .set_octaves(8)
            .set_frequency(0.25);

    needs_generation
        .iter()
        .take(4096)
        .for_each(|(entity, pos)| {
            let pos = *pos;
            let noise = noise.clone();
            let database = database.get_connection();

            let task = thread_pool.spawn(async move {
                let database = database.lock().unwrap();
                let stmt = database.prepare(
                    "SELECT posx, posy, posz, data FROM blocks WHERE posx=:posx AND posy=:posy AND posz=:posz;",
                );
                if let Ok(mut stmt) = stmt {
                    let chunk_result: Result<Vec<u8>, _> = stmt.query_row(
                        &[(":posx", &pos.x), (":posy", &pos.y), (":posz", &pos.z)],
                        |row| Ok(row.get(3).unwrap()),
                    );
                    if let Ok(chunk_row) = chunk_result {
                        let mut temp_output = Cursor::new(Vec::new());
                        copy_decode(&chunk_row[..], &mut temp_output).unwrap();
                        let final_chunk = bincode::deserialize(temp_output.get_ref()).unwrap();

                        return (entity, pos, ChunkData::from_raw(final_chunk));
                    }
                }

                let mut chunk = ChunkData::default();

                for z in 0..ChunkData::edge() {
                    for y in 0..ChunkData::edge() {
                        for x in 0..ChunkData::edge() {
                            let voxel_pos = VoxelPos::from_chunk_coords(pos, x, y, z);
                            let voxel = if voxel_pos.y <= 20 {
                                if voxel_pos.y < 17 {
                                    // Empty bottom chunk
                                    VOXEL_AIR
                                } else {
                                    // Bedrock
                                    VOXEL_BEDROCK
                                }
                            } else {
                                let noise_val = noise
                                    .get([voxel_pos.x as f64 / 100.0, voxel_pos.z as f64 / 100.0])
                                    * 100.0;
                                if (voxel_pos.y as f64) < 102. + noise_val {
                                    // Stoney peaks
                                    if voxel_pos.y > 150 {
                                        VOXEL_STONE
                                    } else {
                                        // Grass
                                        VOXEL_GRASS
                                    }
                                } else if (voxel_pos.y as f64) < 100. + noise_val {
                                    // Stone
                                    VOXEL_STONE
                                } else {
                                    // Air
                                    VOXEL_AIR
                                }
                            };

                            chunk.set(x, y, z, voxel);
                        }
                    }
                }
                (entity, pos, chunk)
            });
            commands.spawn(ComputeChunkData(task));

            commands.entity(entity).remove::<NeedsChunkData>();
        });
}

fn handle_done_generation_tasks(
    mut commands: Commands,
    world: Res<LoadedChunks>,
    mut generation_tasks: Query<(Entity, &mut ComputeChunkData)>,
) {
    let mut loaded = Vec::new();
    generation_tasks
        .iter_mut()
        .take(4096)
        .for_each(|(task_entity, mut task)| {
            if let Some((entity, pos, data)) = future::block_on(future::poll_once(&mut task.0)) {
                if let Some(mut commands) = commands.get_entity(entity) {
                    commands.insert((data, NeedsMesh));
                    loaded.push(pos);
                }

                commands.entity(task_entity).despawn();
            }
        });

    // Re-mesh all neighbors after loading new chunks to simplify geometry
    for neighbor in world.get_unique_loaded_chunk_neighbors(&loaded) {
        commands.entity(neighbor).insert(NeedsMesh);
    }
}
