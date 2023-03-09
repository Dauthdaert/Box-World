use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    input::common_conditions::input_toggle_active,
    pbr::wireframe::{WireframeConfig, WireframePlugin},
    prelude::*,
    render::{
        settings::{WgpuFeatures, WgpuSettings},
        RenderPlugin,
    },
    window::PresentMode,
};
use bevy_flycam::{FlyCam, MovementSettings, NoCameraPlayerPlugin};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use chunk::{LoadPoint, CHUNK_EDGE};
use voxel::VOXEL_SIZE;

mod chunk;
mod mesher;
mod voxel;
mod world_generator;

const HORIZONTAL_VIEW_DISTANCE: usize = 32;
const VERTICAL_VIEW_DISTANCE: usize = 12;

pub fn app() -> App {
    let mut app = App::new();

    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Box World".into(),
                    resolution: (1280., 720.).into(),
                    present_mode: PresentMode::AutoVsync,
                    ..default()
                }),
                ..default()
            })
            .set(ImagePlugin::default_nearest())
            .set(RenderPlugin {
                wgpu_settings: WgpuSettings {
                    features: WgpuFeatures::POLYGON_MODE_LINE,
                    ..default()
                },
            })
            .set(AssetPlugin {
                watch_for_changes: true,
                ..default()
            }),
    )
    .add_plugin(NoCameraPlayerPlugin);

    #[cfg(debug_assertions)]
    {
        app.add_plugin(
            WorldInspectorPlugin::default().run_if(input_toggle_active(false, KeyCode::F3)),
        );

        app.add_plugin(FrameTimeDiagnosticsPlugin::default())
            .add_plugin(LogDiagnosticsPlugin::default())
            .add_plugin(WireframePlugin);

        // Wireframe defaults to off
        app.add_system(toggle_wireframe);
    }

    app.add_startup_system(setup);

    app.add_plugin(world_generator::GeneratorPlugin);
    app.add_plugin(chunk::ChunkPlugin);
    app.add_plugin(mesher::MesherPlugin);

    app
}

fn setup(mut commands: Commands) {
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.8,
    });

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
        FogSettings {
            color: Color::rgba(0.5, 0.5, 0.5, 1.0),
            falloff: FogFalloff::Linear {
                start: ((HORIZONTAL_VIEW_DISTANCE - 4) * CHUNK_EDGE) as f32 * VOXEL_SIZE,
                end: ((HORIZONTAL_VIEW_DISTANCE - 2) * CHUNK_EDGE) as f32 * VOXEL_SIZE,
            },
            ..default()
        },
        LoadPoint,
        FlyCam,
    ));
}

fn toggle_wireframe(mut wireframe_config: ResMut<WireframeConfig>, kb_input: Res<Input<KeyCode>>) {
    if kb_input.just_pressed(KeyCode::F1) {
        wireframe_config.global = !wireframe_config.global;
    }
}
