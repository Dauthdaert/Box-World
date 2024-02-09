use std::f32::consts::PI;

use bevy::{
    input::mouse::MouseMotion,
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow},
};
use bevy_rapier3d::prelude::{
    Collider, CollisionGroups, Group, QueryFilter, RapierContext, Rot, TOIStatus,
};

use crate::chunk::{ChunkPos, LoadedChunks};

use super::{
    camera::{FPSCamera, MouseSensitivity},
    Player, GRAVITY,
};

const PLAYER_JUMP_SPEED: f32 = 10.0;
const PLAYER_RUN_SPEED: f32 = 5.0;
const PLAYER_SPRINT_MOD: f32 = 2.0;

#[allow(clippy::too_many_arguments)]
pub(super) fn movement_input(
    mut player: Query<&mut FPSCamera>,
    player_position: Query<&Transform, With<Player>>,
    camera_transform: Query<&Transform, With<Camera>>,
    mut mouse_events: EventReader<MouseMotion>,
    mouse_sensitivity: Res<MouseSensitivity>,
    key_events: Res<Input<KeyCode>>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    time: Res<Time>,
    mut stationary_frames: Local<i32>,
    current_chunks: Res<LoadedChunks>,
) {
    let mut window = windows.get_single_mut().unwrap();
    if key_events.just_pressed(KeyCode::Escape) {
        window.cursor.grab_mode = match window.cursor.grab_mode {
            CursorGrabMode::None => CursorGrabMode::Locked,
            CursorGrabMode::Confined | CursorGrabMode::Locked => CursorGrabMode::None,
        };
        window.cursor.visible = !window.cursor.visible;
    }

    if let Ok(translation) = player_position.get_single() {
        let translation = translation.translation;
        if current_chunks
            .get_chunk(ChunkPos::from_global_coords(translation))
            .is_none()
        {
            return;
        }

        let mut movement = Vec3::default();
        if let Ok(mut fps_camera) = player.get_single_mut() {
            let transform = camera_transform.single();

            if window.cursor.grab_mode == CursorGrabMode::Locked {
                for MouseMotion { delta } in mouse_events.iter() {
                    fps_camera.phi += delta.x * mouse_sensitivity.0 * 0.003;
                    fps_camera.theta = (fps_camera.theta + delta.y * mouse_sensitivity.0 * 0.003)
                        .clamp(0.00005, PI - 0.00005);
                }

                if key_events.pressed(KeyCode::W) {
                    let mut fwd = transform.forward();
                    fwd.y = 0.0;
                    let fwd = fwd.normalize();
                    movement += fwd;
                }
                if key_events.pressed(KeyCode::A) {
                    movement += transform.left()
                }
                if key_events.pressed(KeyCode::D) {
                    movement += transform.right()
                }
                if key_events.pressed(KeyCode::S) {
                    let mut back = transform.back();
                    back.y = 0.0;
                    let back = back.normalize();
                    movement += back;
                }

                if key_events.pressed(KeyCode::Space) && *stationary_frames > 2 {
                    *stationary_frames = 0;
                    fps_camera.velocity.y = PLAYER_JUMP_SPEED;
                }
            }

            movement = movement.normalize_or_zero();

            if fps_camera.velocity.y.abs() < 0.001 && *stationary_frames < 10 {
                *stationary_frames += 4;
            } else if *stationary_frames >= 0 {
                *stationary_frames -= 1;
            }

            let y = fps_camera.velocity.y;
            fps_camera.velocity.y = 0.0;
            fps_camera.velocity = movement;
            if key_events.pressed(KeyCode::ShiftLeft) {
                fps_camera.velocity *= PLAYER_RUN_SPEED * PLAYER_SPRINT_MOD;
            } else {
                fps_camera.velocity *= PLAYER_RUN_SPEED;
            }
            fps_camera.velocity.y = y;
            let chunk_pos = ChunkPos::from_global_coords(translation);

            if current_chunks.get_chunk(chunk_pos).is_none() {
                return;
            }

            fps_camera.velocity.y -= GRAVITY * time.delta().as_secs_f32().clamp(0.0, 0.1);
        }
    }
}

pub(super) fn movement_collision(
    mut camera: Query<(Entity, &mut FPSCamera)>,
    player: Query<Entity, With<Player>>,
    mut transforms: Query<&mut Transform>,
    time: Res<Time>,
    rapier_context: Res<RapierContext>,
) {
    if let Ok((entity_camera, mut fps_camera)) = camera.get_single_mut() {
        let entity_player = player.single();

        let looking_at = Vec3::new(
            10.0 * fps_camera.phi.cos() * fps_camera.theta.sin(),
            10.0 * fps_camera.theta.cos(),
            10.0 * fps_camera.phi.sin() * fps_camera.theta.sin(),
        );

        let mut camera_t = transforms.get_mut(entity_camera).unwrap();
        camera_t.look_at(looking_at, Vec3::new(0.0, 1.0, 0.0));

        let shape = Collider::cylinder(0.745, 0.2);
        let feet_shape = Collider::cylinder(0.05, 0.2);

        let mut movement_left = fps_camera.velocity * time.delta().as_secs_f32();
        let leg_height = 0.26;

        let filter = QueryFilter {
            flags: Default::default(),
            groups: Some(CollisionGroups::new(Group::GROUP_1, Group::GROUP_2)),
            exclude_collider: None,
            exclude_rigid_body: None,
            predicate: None,
        };

        loop {
            if movement_left.length() <= 0.0 {
                break;
            }
            let mut player_transform = transforms.get_mut(entity_player).unwrap();
            let position = player_transform.translation - Vec3::new(0.0, 0.495, 0.0);

            match rapier_context.cast_shape(
                position,
                Rot::default(),
                movement_left,
                &shape,
                1.0,
                true,
                filter,
            ) {
                None => {
                    player_transform.translation =
                        position + movement_left + Vec3::new(0.0, 0.495, 0.0);
                    break;
                }
                Some((collision_entity, toi)) => {
                    if toi.status != TOIStatus::Converged {
                        let unstuck_vector =
                            transforms.get(collision_entity).unwrap().translation - position;
                        transforms.get_mut(entity_player).unwrap().translation -=
                            unstuck_vector.normalize() * 0.01;
                        fps_camera.velocity = Vec3::new(0.0, 0.0, 0.0);
                        break;
                    }
                    if let Some(details) = toi.details {
                        movement_left -= movement_left.dot(details.normal1) * details.normal1;
                    }
                    fps_camera.velocity = movement_left / time.delta().as_secs_f32();
                }
            }
        }

        if fps_camera.velocity.y <= 0.0 {
            let position =
                transforms.get(entity_player).unwrap().translation - Vec3::new(0.0, 1.19, 0.0);

            if let Some((_, toi)) = rapier_context.cast_shape(
                position,
                Rot::default(),
                Vec3::new(0.0, -1.0, 0.0),
                &feet_shape,
                leg_height,
                true,
                filter,
            ) {
                transforms.get_mut(entity_player).unwrap().translation -=
                    Vec3::new(0.0, toi.toi - leg_height, 0.0);
                fps_camera.velocity.y = 0.0;
            }
        }
    }
}
