use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    pbr::wireframe::{WireframeConfig, WireframePlugin},
    prelude::*,
    render::settings::{WgpuFeatures, WgpuSettings},
    tasks::{AsyncComputeTaskPool, Task},
    utils::FloatOrd,
};
use bevy_flycam::{FlyCam, MovementSettings, NoCameraPlayerPlugin};
use chunk::{ChunkData, ChunkPos};
use futures_lite::future;

use mesher::{generate_mesh, ChunkBoundary};
use noise::{MultiFractal, NoiseFn, OpenSimplex, RidgedMulti};
use rand::seq::IteratorRandom;
use voxel::{Voxel, VoxelPos};

mod chunk;
mod mesher;
mod voxel;
mod world;

pub fn app() -> App {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugin(NoCameraPlayerPlugin);

    #[cfg(debug_assertions)]
    {
        /*use bevy_inspector_egui::quick::WorldInspectorPlugin;
        app.add_plugin(WorldInspectorPlugin);*/

        app.add_plugin(FrameTimeDiagnosticsPlugin::default())
            .add_plugin(LogDiagnosticsPlugin::default())
            .insert_resource(WgpuSettings {
                features: WgpuFeatures::POLYGON_MODE_LINE,
                ..default()
            })
            .add_plugin(WireframePlugin);

        // Wireframe defaults to off
        app.add_system(toggle_wireframe);
    }

    app.add_startup_system(setup);

    app.add_system(load_around_camera)
        .add_system(enqueue_chunk_generation)
        .add_system(enqueue_meshes)
        .add_system(periodic_mesh_maintenance)
        .add_system(periodic_chunk_trim);

    app.add_system_to_stage(CoreStage::PostUpdate, handle_generation);
    app.add_system_to_stage(CoreStage::PostUpdate, handle_meshes);

    app
}

fn setup(mut commands: Commands) {
    let world = world::World::new();
    commands.insert_resource(world);

    // Setup flying camera
    commands.insert_resource(MovementSettings {
        speed: 60.0,
        ..default()
    });
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(10000., 400., 10000.).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        FlyCam,
    ));
}

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct NeedsMesh;

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct NeedsChunkData;

fn load_around_camera(
    mut commands: Commands,
    mut world: ResMut<world::World>,
    camera_query: Query<&Transform, With<FlyCam>>,
) {
    const VIEW_DISTANCE: u32 = 32;

    let camera_translation = camera_query.single().translation;
    let camera_chunk_pos = ChunkPos::from_global_coords(
        camera_translation.x,
        camera_translation.y,
        camera_translation.z,
    );

    let unloaded = world.unload_outside_range(camera_chunk_pos, VIEW_DISTANCE);
    for entity in unloaded.iter() {
        commands.entity(*entity).despawn_recursive();
    }

    let loaded = world.load_inside_range(camera_chunk_pos, VIEW_DISTANCE);
    for (pos, chunk) in loaded.into_iter() {
        let entity = if let Some(chunk) = chunk {
            commands.spawn((pos, chunk, NeedsMesh)).id()
        } else {
            commands.spawn((pos, NeedsChunkData)).id()
        };
        world.set(pos, entity);
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

#[derive(Component)]
struct ComputeChunkData(Task<(Entity, ChunkPos, ChunkData)>);

fn enqueue_chunk_generation(
    mut commands: Commands,
    needs_generation: Query<(Entity, &ChunkPos), With<NeedsChunkData>>,
    camera_query: Query<&Transform, With<FlyCam>>,
) {
    if needs_generation.is_empty() {
        return;
    }

    let camera_translation = camera_query.single().translation;
    let camera_chunk_pos = ChunkPos::from_global_coords(
        camera_translation.x,
        camera_translation.y,
        camera_translation.z,
    );

    let thread_pool = AsyncComputeTaskPool::get();
    let noise: RidgedMulti<OpenSimplex> =
        RidgedMulti::new(RidgedMulti::<OpenSimplex>::DEFAULT_SEED)
            .set_octaves(8)
            .set_frequency(0.25);

    let mut queue: Vec<(Entity, &ChunkPos)> = needs_generation.iter().collect();
    queue.sort_unstable_by_key(|(_entity, pos)| FloatOrd(pos.distance(&camera_chunk_pos)));

    for (entity, pos) in queue.into_iter().take(5000) {
        let pos = *pos;
        let noise = noise.clone();

        let task = thread_pool.spawn(async move {
            let mut chunk = ChunkData::default();

            for z in 0..ChunkData::edge() {
                for y in 0..ChunkData::edge() {
                    for x in 0..ChunkData::edge() {
                        let voxel_pos = VoxelPos::from_chunk_coords(pos, x, y, z);
                        // Bedrock
                        let voxel = if voxel_pos.y <= 3 {
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
    world: Res<world::World>,
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
    world: Res<world::World>,
    needs_meshes: Query<(Entity, &ChunkPos, &ChunkData), With<NeedsMesh>>,
    chunks: Query<&ChunkData>,
    camera_query: Query<&Transform, With<FlyCam>>,
) {
    if needs_meshes.is_empty() {
        return;
    }

    let camera_translation = camera_query.single().translation;
    let camera_chunk_pos = ChunkPos::from_global_coords(
        camera_translation.x,
        camera_translation.y,
        camera_translation.z,
    );

    let thread_pool = AsyncComputeTaskPool::get();

    let mut queue: Vec<(Entity, &ChunkPos, &ChunkData)> = needs_meshes.iter().collect();
    queue.sort_unstable_by_key(|(_entity, pos, _data)| FloatOrd(pos.distance(&camera_chunk_pos)));
    for (entity, pos, data) in queue.into_iter().take(5000) {
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

fn toggle_wireframe(mut wireframe_config: ResMut<WireframeConfig>, kb_input: Res<Input<KeyCode>>) {
    if kb_input.just_pressed(KeyCode::F1) {
        wireframe_config.global = !wireframe_config.global;
    }
}
