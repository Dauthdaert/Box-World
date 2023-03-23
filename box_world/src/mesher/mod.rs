use bevy::{
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task},
};
use bevy_asset_loader::prelude::*;
use bevy_rapier3d::prelude::{AsyncCollider, Collider, ComputedColliderShape};
use futures_lite::future;

use crate::{
    chunk::{ChunkData, ChunkPos, LoadedChunks},
    states::GameStates,
};

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
            .add_system(
                handle_done_meshing_tasks
                    .run_if(resource_exists::<TerrainMaterial>())
                    .in_base_set(CoreSet::PostUpdate),
            )
            .add_system(rapier_slowdown_workaround);

        app.add_plugin(MaterialPlugin::<TerrainTextureMaterial>::default())
            .add_collection_to_loading_state::<_, render::TerrainTexture>(GameStates::AssetLoading)
            .init_resource_after_loading_state::<_, TerrainMaterial>(GameStates::AssetLoading);
    }
}

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct NeedsMesh;

struct ComputedMesh {
    solid_mesh: Option<Mesh>,
    transparent_mesh: Option<Mesh>,
}

#[derive(Component)]
struct ComputeMesh(Task<ComputedMesh>);

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
            let _span = info_span!("Generate mesh and chunk boundary").entered();
            let result = generate_mesh(ChunkBoundary::new(data, neighbors));
            ComputedMesh {
                solid_mesh: result.0,
                transparent_mesh: result.1,
            }
        });
        commands.entity(entity).insert(ComputeMesh(task));
    });
}

#[allow(clippy::type_complexity)]
fn handle_done_meshing_tasks(
    mut commands: Commands,
    terrain_texture: Res<TerrainMaterial>,
    mut mesh_tasks: Query<(
        Entity,
        Option<&Children>,
        &ChunkPos,
        Option<&Transform>,
        &mut ComputeMesh,
    )>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    mesh_tasks.for_each_mut(|(chunk_entity, children, pos, transform, mut task)| {
        if let Some(computed_mesh) = future::block_on(future::poll_once(&mut task.0)) {
            let (chunk_world_x, chunk_world_y, chunk_world_z) = pos.to_global_coords();
            let mut solid_commands = commands.entity(chunk_entity);

            let solid_mesh = computed_mesh.solid_mesh;
            let transparent_mesh = computed_mesh.transparent_mesh;

            if let Some(solid_mesh) = solid_mesh {
                if transform.is_some() {
                    solid_commands.insert((
                        meshes.add(solid_mesh),
                        RapierSlowdownWorkaround,
                        AsyncCollider(ComputedColliderShape::TriMesh),
                    ));
                } else {
                    solid_commands.insert((
                        MaterialMeshBundle {
                            material: terrain_texture.opaque().clone_weak(),
                            mesh: meshes.add(solid_mesh),
                            transform: Transform::from_xyz(
                                chunk_world_x,
                                chunk_world_y,
                                chunk_world_z,
                            ),
                            ..default()
                        },
                        RapierSlowdownWorkaround,
                        AsyncCollider(ComputedColliderShape::TriMesh),
                    ));
                }
            } else if transparent_mesh.is_some() {
                solid_commands.remove::<(Handle<Mesh>, Collider)>();
            } else {
                solid_commands.remove::<(MaterialMeshBundle<TerrainTextureMaterial>, Collider)>();
            }

            solid_commands.remove::<ComputeMesh>();

            let transparent_chunk_entity = children.and_then(|children| children.get(0));
            if let Some(transparent_mesh) = transparent_mesh {
                if let Some(transparent_chunk_entity) = transparent_chunk_entity {
                    let mut transparent_commands = commands.entity(*transparent_chunk_entity);
                    transparent_commands.insert((
                        meshes.add(transparent_mesh),
                        AsyncCollider(ComputedColliderShape::TriMesh),
                    ));
                } else {
                    let child = commands
                        .spawn((
                            MaterialMeshBundle {
                                material: terrain_texture.transparent().clone_weak(),
                                mesh: meshes.add(transparent_mesh),
                                ..default()
                            },
                            AsyncCollider(ComputedColliderShape::TriMesh),
                        ))
                        .id();
                    commands.entity(chunk_entity).add_child(child);
                }
            } else {
                commands.entity(chunk_entity).despawn_descendants();
            }
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
