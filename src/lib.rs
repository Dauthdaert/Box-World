use bevy::{
    pbr::wireframe::{Wireframe, WireframeConfig, WireframePlugin},
    prelude::*,
    render::settings::{WgpuFeatures, WgpuSettings},
    tasks::{AsyncComputeTaskPool, Task},
};
use bevy_flycam::{FlyCam, NoCameraPlayerPlugin};
use bevy_inspector_egui::*;
use futures_lite::future;

use mesher::generate_mesh;
use world::ChunkPos;

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
        app.insert_resource(WorldInspectorParams {
            enabled: true,
            ..default()
        })
        .add_plugin(WorldInspectorPlugin::new())
        .insert_resource(WgpuSettings {
            features: WgpuFeatures::POLYGON_MODE_LINE,
            ..default()
        })
        .add_plugin(WireframePlugin);
    }

    app.add_startup_system(setup);

    app.add_system(load_around_camera).add_system(handle_meshes);

    app
}

#[derive(Component)]
struct Shape;

fn setup(
    mut commands: Commands,
    mut wireframe_config: ResMut<WireframeConfig>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    wireframe_config.global = false;

    let world = world::World::new();
    commands.insert_resource(world);

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 9000.0,
            range: 100.,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(8.0, 16.0, 8.0),
        ..default()
    });

    // ground plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Plane { size: 50. }.into()),
        material: materials.add(Color::SILVER.into()),
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
struct ComputeMesh(Task<Vec<(ChunkPos, Mesh)>>);

fn load_around_camera(
    mut commands: Commands,
    mut world: ResMut<world::World>,
    camera_query: Query<&Transform, With<FlyCam>>,
    chunk_query: Query<(Entity, &ChunkPos)>,
) {
    const VIEW_DISTANCE: u32 = 12;

    let camera_translation = camera_query.single().translation;
    let camera_chunk_pos = ChunkPos::from_world(
        camera_translation.x,
        camera_translation.y,
        camera_translation.z,
    );

    let unloaded = world.unload_outside_range(camera_chunk_pos, VIEW_DISTANCE);
    // TODO: Requires heavy optimisation. Should have a pos to entity index in world.
    for (chunk, pos) in chunk_query.iter() {
        if unloaded.contains(pos) {
            commands.entity(chunk).despawn_recursive();
        }
    }

    let thread_pool = AsyncComputeTaskPool::get();
    let loaded = world.load_inside_range(camera_chunk_pos, VIEW_DISTANCE);
    for chunk_pos_chunk in loaded.chunks(8) {
        let mut boundaries = Vec::new();
        for chunk_pos in chunk_pos_chunk.iter() {
            if let Some(boundary) = world.get_chunk_boundary(*chunk_pos) {
                boundaries.push((*chunk_pos, boundary));
            }
        }
        let task = thread_pool.spawn(async move {
            boundaries
                .iter()
                .map(|(chunk_pos, boundary)| (*chunk_pos, generate_mesh(boundary)))
                .collect()
        });
        commands.spawn(ComputeMesh(task));
    }
}

fn handle_meshes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mesh_tasks: Query<(Entity, &mut ComputeMesh)>,
) {
    for (entity, mut task) in mesh_tasks.iter_mut() {
        if let Some(chunks) = future::block_on(future::poll_once(&mut task.0)) {
            for (chunk_pos, mesh) in chunks {
                let chunk_world_pos = chunk_pos.to_world();
                commands.spawn((
                    PbrBundle {
                        mesh: meshes.add(mesh),
                        transform: Transform::from_xyz(
                            chunk_world_pos.0,
                            chunk_world_pos.1,
                            chunk_world_pos.2,
                        ),
                        ..default()
                    },
                    Shape,
                    Wireframe,
                    chunk_pos,
                ));
            }

            commands.entity(entity).despawn_recursive();
        }
    }
}
