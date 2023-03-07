use bevy::{
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task},
};

use futures_lite::future;

use crate::chunk::{ChunkData, ChunkPos};

use self::{chunk_boundary::ChunkBoundary, generate::generate_mesh};

mod chunk_boundary;
mod face;
mod generate;
mod quads;
mod side;
mod visibility;

pub use visibility::VoxelVisibility;

pub struct MesherPlugin;

impl Plugin for MesherPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system(enqueue_meshes).add_system(ease_meshes);

        app.add_system(handle_meshes.in_base_set(CoreSet::PostUpdate));
    }
}

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct NeedsMesh;

#[derive(Component)]
struct ComputeMesh(Task<(Entity, ChunkPos, Mesh)>);

fn enqueue_meshes(
    mut commands: Commands,
    world: Res<crate::world::World>,
    needs_mesh: Query<(Entity, &ChunkPos, &ChunkData), With<NeedsMesh>>,
    chunks: Query<&ChunkData>,
) {
    if needs_mesh.is_empty() {
        return;
    }

    let thread_pool = AsyncComputeTaskPool::get();

    for (entity, pos, data) in needs_mesh.iter() {
        let neighbors: Vec<ChunkData> = world
            .get_chunk_neighbors(*pos)
            .into_iter()
            .filter_map(|entity| chunks.get(entity).map(|data| data.clone()).ok())
            .collect();

        // Skip meshing when we don't have data for all neighbors
        if neighbors.len() != 26 {
            continue;
        }

        // Clone out of needs_meshes before moving into task
        let pos = *pos;
        let data = data.clone();
        let task = thread_pool.spawn(async move {
            let mesh = generate_mesh(ChunkBoundary::new(data, neighbors));
            (entity, pos, mesh)
        });
        commands.spawn(ComputeMesh(task));

        commands.entity(entity).remove::<NeedsMesh>();
    }
}

fn handle_meshes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mesh_tasks: Query<(Entity, &mut ComputeMesh)>,
) {
    for (task_entity, mut task) in mesh_tasks.iter_mut() {
        if let Some((entity, pos, mesh)) = future::block_on(future::poll_once(&mut task.0)) {
            let chunk_world_pos = pos.to_global_coords();
            if let Some(mut commands) = commands.get_entity(entity) {
                if mesh.indices().map_or(false, |indices| !indices.is_empty()) {
                    commands.insert((
                        PbrBundle {
                            mesh: meshes.add(mesh),
                            transform: Transform::from_xyz(
                                chunk_world_pos.0,
                                -100.,
                                chunk_world_pos.2,
                            ),
                            ..default()
                        },
                        EaseToChunkPos,
                    ));
                } else {
                    commands.remove::<(PbrBundle, EaseToChunkPos)>();
                }
            }

            commands.entity(task_entity).despawn_recursive();
        }
    }
}

#[derive(Component)]
#[component(storage = "SparseSet")]
struct EaseToChunkPos;

#[allow(clippy::type_complexity)]
fn ease_meshes(
    timer: Res<Time>,
    mut commands: Commands,
    mut chunks: Query<
        (Entity, &mut Transform, &ChunkPos),
        (With<Handle<Mesh>>, With<EaseToChunkPos>),
    >,
) {
    for (entity, mut transform, pos) in chunks.iter_mut() {
        let dt = timer.delta_seconds();
        let (_global_x, global_y, _global_z) = pos.to_global_coords();
        transform.translation.y = global_y.min(transform.translation.y + 100. * dt);

        if transform.translation.y == global_y {
            commands.entity(entity).remove::<EaseToChunkPos>();
        }
    }
}
