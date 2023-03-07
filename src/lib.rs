use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    pbr::wireframe::{WireframeConfig, WireframePlugin},
    prelude::*,
    render::{
        settings::{WgpuFeatures, WgpuSettings},
        RenderPlugin,
    },
};
use bevy_flycam::{FlyCam, MovementSettings, NoCameraPlayerPlugin};
use chunk::ChunkPos;

use crate::{mesher::NeedsMesh, world::NeedsChunkData};

mod chunk;
mod mesher;
mod voxel;
mod world;

pub fn app() -> App {
    let mut app = App::new();

    app.add_plugins(
        DefaultPlugins
            .set(ImagePlugin::default_nearest())
            .set(RenderPlugin {
                wgpu_settings: WgpuSettings {
                    features: WgpuFeatures::POLYGON_MODE_LINE,
                    ..default()
                },
            }),
    )
    .add_plugin(NoCameraPlayerPlugin);

    #[cfg(debug_assertions)]
    {
        /*use bevy_inspector_egui::quick::WorldInspectorPlugin;
        app.add_plugin(WorldInspectorPlugin);*/

        app.add_plugin(FrameTimeDiagnosticsPlugin::default())
            .add_plugin(LogDiagnosticsPlugin::default())
            .add_plugin(WireframePlugin);

        // Wireframe defaults to off
        app.add_system(toggle_wireframe);
    }

    app.add_startup_system(setup);

    app.add_plugin(world::WorldPlugin);
    app.add_plugin(mesher::MesherPlugin);

    app.add_system(load_around_camera);

    app
}

fn setup(mut commands: Commands) {
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

fn load_around_camera(
    mut commands: Commands,
    mut world: ResMut<world::World>,
    camera_query: Query<&Transform, With<FlyCam>>,
) {
    const HORIZONTAL_VIEW_DISTANCE: usize = 32;
    const VERTICAL_VIEW_DISTANCE: usize = 12;

    let camera_translation = camera_query.single().translation;
    let camera_chunk_pos = ChunkPos::from_global_coords(
        camera_translation.x,
        camera_translation.y,
        camera_translation.z,
    );

    let unloaded = world.unload_outside_range(camera_chunk_pos, HORIZONTAL_VIEW_DISTANCE);
    for entity in unloaded.iter() {
        commands.entity(*entity).despawn_recursive();
    }

    let loaded = world.load_inside_range(
        camera_chunk_pos,
        HORIZONTAL_VIEW_DISTANCE,
        VERTICAL_VIEW_DISTANCE,
    );
    for (pos, chunk) in loaded.into_iter() {
        let entity = if let Some(chunk) = chunk {
            commands.spawn((pos, chunk, NeedsMesh)).id()
        } else {
            commands.spawn((pos, NeedsChunkData)).id()
        };
        world.set(pos, entity);
    }
}

fn toggle_wireframe(mut wireframe_config: ResMut<WireframeConfig>, kb_input: Res<Input<KeyCode>>) {
    if kb_input.just_pressed(KeyCode::F1) {
        wireframe_config.global = !wireframe_config.global;
    }
}
