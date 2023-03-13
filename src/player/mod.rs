use crate::{
    chunk::{ChunkData, LoadPoint},
    states::GameStates,
    voxel::VOXEL_SIZE,
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

use self::input::{FPSCamera, MouseSensitivity};

mod bundle;
mod collision;
mod input;

pub struct PlayerPlugin;

/// Implements player controller
/// Based on the version used in Vinox : https://github.com/Vixeliz/Vinox
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
            .insert_resource(RapierConfiguration {
                gravity: Vec3::new(0.0, -35.0, 0.0),
                ..default()
            });
        app.insert_resource(MouseSensitivity(1.0));

        app.add_startup_system(spawn_player_load_point);

        app.add_system(spawn_player_cam_and_collider.in_schedule(OnEnter(GameStates::InGame)));

        app.add_systems(
            (
                input::movement_input,
                input::interact.after(collision::movement),
            )
                .in_set(OnUpdate(GameStates::InGame)),
        );

        app.add_system(collision::movement.in_set(OnUpdate(GameStates::InGame)));
    }
}

#[derive(Component, Default)]
pub struct Player;

fn spawn_player_load_point(mut commands: Commands) {
    // Initially only load a small area around the player for speed
    // We will load to view distance after spawning
    commands.spawn(bundle::PreSpawnPlayerBundle::new(
        16,
        10,
        Vec3::new(10000., 400., 10000.),
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
            fov: std::f32::consts::PI / 1.8,
            near: 0.001,
            far: 1000.0,
            aspect_ratio: 1.0,
        };
        let view_projection = perspective_projection.get_projection_matrix();
        let frustum = Frustum::from_view_projection_custom_far(
            &view_projection,
            &Vec3::ZERO,
            &Vec3::Z,
            perspective_projection.far(),
        );
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
                        start: ((HORIZONTAL_VIEW_DISTANCE - 4) * ChunkData::edge()) as f32
                            * VOXEL_SIZE,
                        end: ((HORIZONTAL_VIEW_DISTANCE - 2) * ChunkData::edge()) as f32
                            * VOXEL_SIZE,
                    },
                    ..default()
                },
            ));
        });
}
