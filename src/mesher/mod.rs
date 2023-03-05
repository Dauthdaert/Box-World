use bevy::{
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task},
};

use futures_lite::future;
use rand::seq::IteratorRandom;

use crate::chunk::{ChunkData, ChunkPos};

use self::{chunk_boundary::ChunkBoundary, generate::generate_mesh};

mod chunk_boundary;
mod generate;

pub struct MesherPlugin;

impl Plugin for MesherPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system(enqueue_meshes)
            .add_system(periodic_mesh_maintenance);

        app.add_system_to_stage(CoreStage::PostUpdate, handle_meshes);
    }
}

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct NeedsMesh;

#[allow(clippy::type_complexity)]
fn periodic_mesh_maintenance(
    mut commands: Commands,
    chunks: Query<(Entity, &ChunkData), (With<Handle<Mesh>>, Without<NeedsMesh>)>,
) {
    let mut rng = rand::thread_rng();
    for entity in chunks
        .iter()
        .filter_map(|(entity, data)| {
            if data.is_uniform() {
                Some(entity)
            } else {
                None
            }
        })
        .choose_multiple(&mut rng, 2)
    {
        commands.entity(entity).insert(NeedsMesh);
    }
}

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
        let neighbors = world.get_chunk_neighbors(*pos).map(|entity| {
            if let Some(entity) = entity {
                if let Ok(data) = chunks.get(entity) {
                    return data.clone();
                }
            }
            ChunkData::default()
        });

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
                    commands.insert(PbrBundle {
                        mesh: meshes.add(mesh),
                        transform: Transform::from_xyz(
                            chunk_world_pos.0,
                            chunk_world_pos.1,
                            chunk_world_pos.2,
                        ),
                        ..default()
                    });
                } else {
                    commands.remove::<PbrBundle>();
                }
            }

            commands.entity(task_entity).despawn_recursive();
        }
    }
}
