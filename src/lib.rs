use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    pbr::wireframe::{WireframeConfig, WireframePlugin},
    prelude::*,
    render::settings::{WgpuFeatures, WgpuSettings},
    tasks::{AsyncComputeTaskPool, Task},
};
use bevy_flycam::{FlyCam, MovementSettings, NoCameraPlayerPlugin};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use chunk::{ChunkData, ChunkPos};
use futures_lite::future;

use mesher::{generate_mesh, ChunkBoundary};

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
        app.add_plugin(WorldInspectorPlugin)
            .add_plugin(FrameTimeDiagnosticsPlugin::default())
            .add_plugin(LogDiagnosticsPlugin::default())
            .insert_resource(WgpuSettings {
                features: WgpuFeatures::POLYGON_MODE_LINE,
                ..default()
            })
            .add_plugin(WireframePlugin);

        // Wireframe default to off
        app.add_system(toggle_wireframe);
    }

    app.add_startup_system(setup);

    app.add_system(load_around_camera)
        .add_system(enqueue_meshes)
        .add_system(handle_meshes);

    app
}

fn setup(mut commands: Commands) {
    let world = world::World::new();
    commands.insert_resource(world);

    // Setup flying camera
    commands.insert_resource(MovementSettings {
        speed: 30.0,
        ..default()
    });
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 6., 12.0)
                .looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
            ..default()
        },
        FlyCam,
    ));
}

#[derive(Component)]
pub struct NeedsMesh;

fn load_around_camera(
    mut commands: Commands,
    mut world: ResMut<world::World>,
    camera_query: Query<&Transform, With<FlyCam>>,
    chunk_query: Query<&ChunkPos>,
) {
    const VIEW_DISTANCE: u32 = 12;

    let camera_translation = camera_query.single().translation;
    let camera_chunk_pos = ChunkPos::from_global_coords(
        camera_translation.x,
        camera_translation.y,
        camera_translation.z,
    );

    let unloaded = world.unload_outside_range(camera_chunk_pos, VIEW_DISTANCE);
    for entity in unloaded.iter() {
        // Re-mesh all remaining neighbors after unloading a chunk
        for neighbor in world
            .get_chunk_neighbors(*chunk_query.get(*entity).unwrap())
            .into_iter()
            .flatten()
            .filter(|neighbor| !unloaded.contains(neighbor))
        {
            commands.entity(neighbor).insert(NeedsMesh);
        }

        commands.entity(*entity).despawn_recursive();
    }

    let loaded = world.load_inside_range(camera_chunk_pos, VIEW_DISTANCE);
    for (pos, chunk) in loaded {
        world.set(pos, commands.spawn((pos, chunk, NeedsMesh)).id());
    }
}

#[derive(Component)]
struct ComputeMesh(Task<(Entity, ChunkPos, Mesh)>);

fn enqueue_meshes(
    mut commands: Commands,
    world: Res<world::World>,
    needs_meshes: Query<(Entity, &ChunkPos, &ChunkData), With<NeedsMesh>>,
    chunks: Query<&ChunkData>,
) {
    if needs_meshes.is_empty() {
        return;
    }

    let thread_pool = AsyncComputeTaskPool::get();
    for (entity, pos, data) in needs_meshes.iter() {
        let neighbors = world.get_chunk_neighbors(*pos).map(|entity| {
            if let Some(entity) = entity {
                chunks.get(entity).unwrap().clone()
            } else {
                ChunkData::default()
            }
        });

        let boundary = ChunkBoundary::new(data.clone(), neighbors);
        let pos = *pos;
        let task = thread_pool.spawn(async move {
            let mesh = generate_mesh(boundary);
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
            if mesh.count_vertices() > 0 {
                if let Some(mut commands) = commands.get_entity(entity) {
                    commands.insert(PbrBundle {
                        mesh: meshes.add(mesh),
                        transform: Transform::from_xyz(
                            chunk_world_pos.0,
                            chunk_world_pos.1,
                            chunk_world_pos.2,
                        ),
                        ..default()
                    });
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
