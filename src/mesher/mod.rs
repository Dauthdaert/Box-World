use bevy::{
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task},
};

use bevy_rapier3d::prelude::Collider;
use futures_lite::future;

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
            .add_system(rapier_slowdown_workaround);

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
struct ComputeMesh(Task<(Mesh, Option<Collider>)>);

fn enqueue_meshing_tasks(
    mut commands: Commands,
    world: Res<LoadedChunks>,
    needs_mesh: Query<(Entity, &ChunkPos, &ChunkData), With<NeedsMesh>>,
    chunks: Query<&ChunkData>,
) {
    let thread_pool = AsyncComputeTaskPool::get();

    needs_mesh.iter().take(512).for_each(|(entity, pos, data)| {
        commands.entity(entity).remove::<NeedsMesh>();

        // Skip meshing if chunk is empty, garanteed empty mesh
        if data.is_empty() {
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
        let data = data.clone();

        let task = thread_pool.spawn(async move {
            let _span = info_span!("Generate mesh task").entered();
            generate_mesh(ChunkBoundary::new(data, neighbors))
        });
        commands.entity(entity).insert(ComputeMesh(task));
    });
}

fn handle_done_meshing_tasks(
    mut commands: Commands,
    terrain_texture: Res<TerrainTexture>,
    mut mesh_tasks: Query<(Entity, &ChunkPos, Option<&Transform>, &mut ComputeMesh)>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    mesh_tasks.for_each_mut(|(task_entity, pos, transform, mut task)| {
        if let Some((mesh, collider)) = future::block_on(future::poll_once(&mut task.0)) {
            let (chunk_world_x, chunk_world_y, chunk_world_z) = pos.to_global_coords();
            let mut commands = commands.entity(task_entity);
            if mesh.indices().map_or(false, |indices| !indices.is_empty()) {
                if transform.is_some() {
                    commands.insert((
                        meshes.add(mesh),
                        RapierSlowdownWorkaround,
                        collider.expect("Collider should exist if mesh exists"),
                    ));
                } else {
                    commands.insert((
                        MaterialMeshBundle {
                            material: terrain_texture.material_handle().clone_weak(),
                            mesh: meshes.add(mesh),
                            transform: Transform::from_xyz(
                                chunk_world_x,
                                chunk_world_y,
                                chunk_world_z,
                            ),
                            ..default()
                        },
                        RapierSlowdownWorkaround,
                        collider.expect("Collider should exist if mesh exists"),
                    ));
                }
            } else {
                commands.remove::<(MaterialMeshBundle<TerrainTextureMaterial>, Collider)>();
            }

            commands.remove::<ComputeMesh>();
        }
    });
}

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct RapierSlowdownWorkaround;

fn rapier_slowdown_workaround(
    mut commands: Commands,
    mut chunks: Query<(Entity, &mut Transform), With<RapierSlowdownWorkaround>>,
) {
    chunks.for_each_mut(|(entity, mut transform)| {
        transform.set_changed();
        commands.entity(entity).remove::<RapierSlowdownWorkaround>();
    });
}
