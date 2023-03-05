use bevy::{
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task},
};

use futures_lite::future;
use noise::{Fbm, MultiFractal, NoiseFn, OpenSimplex, Perlin, RidgedMulti};

use crate::{
    chunk::{ChunkData, ChunkPos},
    mesher::NeedsMesh,
    voxel::{Voxel, VoxelPos},
};

const MIN_TERRAIN_HEIGHT: u32 = 40;

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
    let height_noise: Fbm<OpenSimplex> = Fbm::new(0).set_octaves(4);
    let pv_noise: RidgedMulti<OpenSimplex> = RidgedMulti::new(0).set_octaves(8).set_frequency(0.25);
    let density_noise: Fbm<Perlin> = Fbm::new(0).set_octaves(4).set_frequency(0.5);

    for (entity, pos) in needs_generation.iter() {
        let pos = *pos;
        let height_noise = height_noise.clone();
        let pv_noise = pv_noise.clone();
        let density_noise = density_noise.clone();

        let task = thread_pool.spawn(async move {
            let chunk = generate_chunk(&height_noise, &pv_noise, &density_noise, pos);
            (entity, pos, chunk)
        });
        commands.spawn(ComputeChunkData(task));

        commands.entity(entity).remove::<NeedsChunkData>();
    }
}

fn generate_chunk(
    height_noise: &dyn NoiseFn<f64, 2>,
    pv_noise: &dyn NoiseFn<f64, 2>,
    density_noise: &dyn NoiseFn<f64, 3>,
    pos: ChunkPos,
) -> ChunkData {
    let mut chunk = ChunkData::default();

    for z in 0..ChunkData::edge() {
        for y in 0..ChunkData::edge() {
            for x in 0..ChunkData::edge() {
                let voxel_pos = VoxelPos::from_chunk_coords(pos, x, y, z);
                let voxel = generate_voxel(height_noise, pv_noise, density_noise, voxel_pos);
                chunk.set(x, y, z, voxel);
            }
        }
    }

    chunk
}

fn generate_voxel(
    height_noise: &dyn NoiseFn<f64, 2>,
    pv_noise: &dyn NoiseFn<f64, 2>,
    density_noise: &dyn NoiseFn<f64, 3>,
    voxel_pos: VoxelPos,
) -> Voxel {
    if voxel_pos.y <= 3 {
        // Bedrock
        return Voxel::Opaque(1);
    }

    let scaled_x = voxel_pos.x as f64 / 100.;
    let scaled_y = voxel_pos.y as f64 / 100.;
    let scaled_z = voxel_pos.z as f64 / 100.;

    const SQUASH_FACTOR: f64 = 4.0;
    let density_value = density_noise.get([scaled_x, scaled_y, scaled_z]) * 100.
        - (voxel_pos.y as f64 / SQUASH_FACTOR);

    if density_value < 0.0 {
        return Voxel::Empty;
    }

    let height_value = height_noise.get([scaled_x, scaled_z]);

    let terrain_height = if height_value > 0. {
        let pv_value = pv_noise.get([scaled_x, scaled_z]);
        let noise_avg = (height_value * 20.0 + pv_value * 60.0) / 2.;
        MIN_TERRAIN_HEIGHT + noise_avg as u32
    } else {
        MIN_TERRAIN_HEIGHT + (height_value.abs() * 30.0) as u32
    };

    if (voxel_pos.y) < terrain_height {
        // Stone
        return Voxel::Opaque(2);
    }

    // Air
    Voxel::Empty
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
