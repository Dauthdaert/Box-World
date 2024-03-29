use crate::{
    chunk::{ChunkData, LoadPoint},
    states::GameStates,
    voxel::GlobalVoxelPos,
    HORIZONTAL_VIEW_DISTANCE, VERTICAL_VIEW_DISTANCE,
};
use bevy::{
    prelude::*,
    render::{camera::CameraProjection, primitives::Frustum},
    window::{CursorGrabMode, PrimaryWindow},
};
use bevy_atmosphere::prelude::AtmosphereCamera;
use bevy_rapier3d::prelude::{
    Collider, CollisionGroups, Group, NoUserData, RapierConfiguration, RapierPhysicsPlugin,
    SolverGroups,
};

mod bundle;
mod camera;
mod input;
mod movement;

use camera::{FPSCamera, MouseSensitivity};

const GRAVITY: f32 = 25.0;

pub struct PlayerPlugin;

/// Implements player controller
/// Based on the version used in Vinox : https://github.com/Vixeliz/Vinox
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
            .insert_resource(RapierConfiguration {
                gravity: Vec3::new(0.0, -GRAVITY, 0.0),
                ..default()
            });
        app.insert_resource(MouseSensitivity(1.0));

        app.init_resource::<input::CurrentBlock>();

        app.add_systems(Startup, spawn_player_load_point);

        app.add_systems(OnEnter(GameStates::InGame), spawn_player_cam_and_collider);

        app.add_systems(
            Update,
            (
                movement::movement_input,
                movement::movement_collision,
                input::interact.after(movement::movement_collision),
                input::change_current_block,
            )
                .run_if(in_state(GameStates::InGame)),
        );
    }
}

#[derive(Component, Default)]
pub struct Player;

fn spawn_player_load_point(mut commands: Commands) {
    // Initially only load a small area around the player for speed
    // We will load to view distance after spawning
    let player_pos = GlobalVoxelPos::new(5000, 200, 5000);
    commands.spawn(bundle::PreSpawnPlayerBundle::new(
        16,
        10,
        player_pos.as_vec3(),
    ));
}

pub fn spawn_player_cam_and_collider(
    mut commands: Commands,
    cameras: Query<Entity, With<Camera>>,
    player: Query<Entity, With<Player>>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    cameras
        .iter()
        .for_each(|entity| commands.entity(entity).despawn_recursive());

    let mut window = windows.get_single_mut().unwrap();
    window.cursor.grab_mode = CursorGrabMode::Locked;
    window.cursor.visible = false;

    let camera = {
        let perspective_projection = PerspectiveProjection {
            fov: 80.0_f32.to_radians(),
            near: 0.001,
            far: 1250.0,
            aspect_ratio: 1.0,
        };
        let view_projection = perspective_projection.get_projection_matrix();
        let frustum = Frustum::from_view_projection(&view_projection);
        Camera3dBundle {
            projection: Projection::Perspective(perspective_projection),
            frustum,
            ..default()
        }
    };

    let player_entity = player.single();
    commands
        .entity(player_entity)
        .insert(LoadPoint {
            horizontal: HORIZONTAL_VIEW_DISTANCE,
            vertical: VERTICAL_VIEW_DISTANCE,
        })
        .insert(bundle::PostSpawnPlayerBundle::default())
        .with_children(|c| {
            c.spawn((
                GlobalTransform::default(),
                Transform::from_xyz(0.0, 2.0, 0.0),
                Collider::cylinder(1.6, 0.4),
                SolverGroups::new(Group::GROUP_1, Group::GROUP_2),
                CollisionGroups::new(Group::GROUP_1, Group::GROUP_2),
            ));
            c.spawn((
                FPSCamera::default(),
                camera,
                AtmosphereCamera::default(),
                FogSettings {
                    color: Color::rgba(0.5, 0.5, 0.5, 1.0),
                    falloff: FogFalloff::Linear {
                        start: ((HORIZONTAL_VIEW_DISTANCE - 4) * ChunkData::edge()) as f32,
                        end: ((HORIZONTAL_VIEW_DISTANCE - 2) * ChunkData::edge()) as f32,
                    },
                    ..default()
                },
            ));
        });
}
