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
use bevy_asset_loader::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use chunk::{ChunkData, ChunkPos, LoadPoint, LoadedChunks};
use lighting::NeedsLightPass;
use mesher::NeedsMesh;
use player::Player;
use states::GameStates;

mod chunk;
mod environment;
mod lighting;
mod mesher;
mod player;
mod states;
mod voxel;
mod world_generator;

const HORIZONTAL_VIEW_DISTANCE: u32 = 32;
const VERTICAL_VIEW_DISTANCE: u32 = 12;

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
    .insert_resource(Msaa::Sample8);

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

    app.add_state::<GameStates>().add_loading_state(
        LoadingState::new(GameStates::AssetLoading).continue_to_state(GameStates::WorldLoading),
    );

    app.add_startup_system(setup)
        .insert_resource(LoadingTimer::new())
        .add_system(setup.in_schedule(OnEnter(GameStates::WorldLoading)))
        .add_system(transition_after_load.run_if(in_state::<GameStates>(GameStates::WorldLoading)));

    app.add_plugin(voxel::VoxelPlugin);
    app.add_plugin(chunk::ChunkPlugin);
    app.add_plugin(world_generator::GeneratorPlugin);
    app.add_plugin(mesher::MesherPlugin);
    app.add_plugin(player::PlayerPlugin);
    app.add_plugin(lighting::LightingPlugin);
    app.add_plugin(environment::EnvironmentPlugin);

    app
}

fn setup(mut commands: Commands) {
    commands.spawn((
        /*Camera3dBundle {
            transform: Transform::from_xyz(10000., 400., 10000.),
            ..default()
        },*/
        TransformBundle {
            local: Transform::from_xyz(10000., 400., 10000.),
            ..default()
        },
        LoadPoint {
            horizontal: 8,
            vertical: 4,
        },
        Name::new("Spawn"),
    ));
}

#[derive(Resource)]
struct LoadingTimer(pub Timer);

impl LoadingTimer {
    fn new() -> Self {
        Self(Timer::from_seconds(1.0, TimerMode::Repeating))
    }
}

#[allow(clippy::type_complexity)]
fn transition_after_load(
    mut next_state: ResMut<NextState<GameStates>>,
    loaded_chunks: Res<LoadedChunks>,
    player: Query<&Transform, With<Player>>,
    chunks: Query<(), (With<ChunkData>, Without<NeedsLightPass>, Without<NeedsMesh>)>,
    time: Res<Time>,
    mut loading_timer: ResMut<LoadingTimer>,
) {
    if loading_timer.0.tick(time.delta()).just_finished() {
        info!("Don't worry, we're still loading.");

        let player = player.single().translation;
        let player_chunk_pos = ChunkPos::from_global_coords(player);

        // Check current chunk
        let Some(current) = loaded_chunks.get_chunk(player_chunk_pos) else { return; };
        if chunks.get(*current).is_err() {
            return;
        }

        // Check neighbor chunks
        for neighbor in ChunkPos::neighbors(&player_chunk_pos) {
            let Some(neighbor) = loaded_chunks.get_chunk(neighbor) else { return; };
            if chunks.get(*neighbor).is_err() {
                return;
            }
        }

        // All important chunks loaded
        next_state.set(GameStates::InGame);
        info!("Done loading!");
    }
}

fn toggle_wireframe(mut wireframe_config: ResMut<WireframeConfig>, kb_input: Res<Input<KeyCode>>) {
    if kb_input.just_pressed(KeyCode::F1) {
        wireframe_config.global = !wireframe_config.global;
    }
}
