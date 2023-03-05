use bevy::{
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task},
};

use futures_lite::future;
use noise::{MultiFractal, NoiseFn, OpenSimplex, RidgedMulti};

use crate::{
    chunk::{ChunkData, ChunkPos},
    mesher::NeedsMesh,
    voxel::{Voxel, VoxelPos},
};

pub struct GeneratorPlugin;

impl Plugin for GeneratorPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system(enqueue_chunk_generation);

        app.add_system_to_stage(CoreStage::PostUpdate, handle_generation);
    }
}

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct NeedsChunkData;

#[derive(Component)]
struct ComputeChunkData(Task<(Entity, ChunkPos, ChunkData)>);

fn enqueue_chunk_generation(
    mut commands: Commands,
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

    for (entity, pos) in needs_generation.iter() {
        let pos = *pos;
        let noise = noise.clone();

        let task = thread_pool.spawn(async move {
            let mut chunk = ChunkData::default();

            for z in 0..ChunkData::edge() {
                for y in 0..ChunkData::edge() {
                    for x in 0..ChunkData::edge() {
                        let voxel_pos = VoxelPos::from_chunk_coords(pos, x, y, z);
                        let voxel = if voxel_pos.y <= 3 {
                            // Bedrock
                            Voxel::Opaque(1)
                        } else {
                            let noise_val = noise
                                .get([voxel_pos.x as f64 / 100.0, voxel_pos.z as f64 / 100.0])
                                * 100.0;
                            if (voxel_pos.y as f64) < 100. + noise_val {
                                // Stone
                                Voxel::Opaque(2)
                            } else {
                                // Air
                                Voxel::Empty
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
    }
}

fn handle_generation(
    mut commands: Commands,
    world: Res<crate::world::World>,
    mut generation_tasks: Query<(Entity, &mut ComputeChunkData)>,
) {
    let mut loaded = Vec::new();
    for (task_entity, mut task) in generation_tasks.iter_mut() {
        if let Some((entity, pos, data)) = future::block_on(future::poll_once(&mut task.0)) {
            if let Some(mut commands) = commands.get_entity(entity) {
                commands.insert((data, NeedsMesh));
                loaded.push(pos);
            }

            commands.entity(task_entity).despawn_recursive();
        }
    }

    // Re-mesh all neighbors after loading new chunks to simplify geometry
    for neighbor in world.get_unique_chunk_neighbors(loaded) {
        commands.entity(neighbor).insert(NeedsMesh);
    }
}
