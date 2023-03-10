use bevy::{
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task},
};

use bevy_rapier3d::prelude::Collider;
use futures_lite::future;
use rand::seq::IteratorRandom;

use crate::chunk::{ChunkData, ChunkPos, LoadedChunks};

use self::{chunk_boundary::ChunkBoundary, generate::generate_mesh, render::*};

mod chunk_boundary;
mod face;
mod generate;
mod quads;
mod render;
mod side;
mod visibility;

pub use visibility::VoxelVisibility;

pub struct MesherPlugin;

impl Plugin for MesherPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system(enqueue_meshing_tasks)
            .add_system(handle_done_meshing_tasks.in_base_set(CoreSet::PostUpdate))
            .add_system(ease_new_meshes_to_position);

        app.add_plugin(MaterialPlugin::<TerrainTextureMaterial>::default())
            .add_startup_system(load_terrain_texture)
            .add_system(
                create_terrain_texture_material
                    .run_if(|texture: Res<TerrainTexture>| !texture.is_loaded()),
            );
    }
}

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct NeedsMesh;

#[derive(Component)]
struct ComputeMesh(Task<(Entity, ChunkPos, Mesh, Option<Collider>)>);

fn enqueue_meshing_tasks(
    mut commands: Commands,
    world: Res<LoadedChunks>,
    needs_mesh: Query<(Entity, &ChunkPos, &ChunkData), With<NeedsMesh>>,
    chunks: Query<&ChunkData>,
) {
    if needs_mesh.is_empty() {
        return;
    }

    let thread_pool = AsyncComputeTaskPool::get();

    let mut rng = rand::thread_rng();
    needs_mesh
        .iter()
        .choose_multiple(&mut rng, 256)
        .into_iter()
        .for_each(|(entity, pos, data)| {
            // Skip meshing if chunk is empty, garanteed empty mesh
            if data.is_empty() {
                commands.entity(entity).remove::<NeedsMesh>();
                return;
            }

            let neighbors_entity = world.get_loaded_chunk_neighbors(*pos);

            // Skip getting chunk data when we don't have data for all neighbors
            if neighbors_entity.len() != 26 {
                return;
            }

            let mut neighbors = Vec::with_capacity(26);
            for entity in neighbors_entity.into_iter() {
                if let Ok(data) = chunks.get(entity) {
                    neighbors.push(data);
                } else {
                    // Skip meshing when we don't have data for all neighbors
                    return;
                }
            }

            // Clone out of needs_meshes before moving into task
            let neighbors: Vec<ChunkData> = neighbors.into_iter().cloned().collect();
            let pos = *pos;
            let data = data.clone();

            let task = thread_pool.spawn(async move {
                let (mesh, collider) = generate_mesh(ChunkBoundary::new(data, neighbors));
                (entity, pos, mesh, collider)
            });
            commands.spawn(ComputeMesh(task));

            commands.entity(entity).remove::<NeedsMesh>();
        });
}

fn handle_done_meshing_tasks(
    mut commands: Commands,
    terrain_texture: Res<TerrainTexture>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mesh_tasks: Query<(Entity, &mut ComputeMesh)>,
) {
    mesh_tasks.for_each_mut(|(task_entity, mut task)| {
        if let Some((entity, pos, mesh, collider)) =
            future::block_on(future::poll_once(&mut task.0))
        {
            let chunk_world_pos = pos.to_global_coords();
            if let Some(mut commands) = commands.get_entity(entity) {
                if mesh.indices().map_or(false, |indices| !indices.is_empty()) {
                    commands.insert((
                        MaterialMeshBundle {
                            material: terrain_texture.material_handle().clone_weak(),
                            mesh: meshes.add(mesh),
                            transform: Transform::from_xyz(
                                chunk_world_pos.0,
                                -100.,
                                chunk_world_pos.2,
                            ),
                            ..default()
                        },
                        collider.expect("Collider should exist if mesh exists"),
                        EaseToChunkPos,
                    ));
                } else {
                    commands.remove::<(
                        MaterialMeshBundle<TerrainTextureMaterial>,
                        Collider,
                        EaseToChunkPos,
                    )>();
                }
            }

            commands.entity(task_entity).despawn();
        }
    });
}

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct EaseToChunkPos;

#[allow(clippy::type_complexity)]
fn ease_new_meshes_to_position(
    timer: Res<Time>,
    mut commands: Commands,
    mut chunks: Query<
        (Entity, &mut Transform, &ChunkPos),
        (With<Handle<Mesh>>, With<EaseToChunkPos>),
    >,
) {
    chunks.for_each_mut(|(entity, mut transform, pos)| {
        let dt = timer.delta_seconds();
        let (_global_x, global_y, _global_z) = pos.to_global_coords();
        transform.translation.y = global_y.min(transform.translation.y + 100. * dt);

        if transform.translation.y == global_y {
            commands.entity(entity).remove::<EaseToChunkPos>();
        }
    });
}
