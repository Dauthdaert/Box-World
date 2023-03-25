use bevy::prelude::*;
use bevy_rapier3d::prelude::TOIStatus::Converged;
use bevy_rapier3d::prelude::{Collider, CollisionGroups, Group, QueryFilter, RapierContext, Rot};

use super::{input::FPSCamera, Player};

pub(super) fn movement(
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
                filter,
            ) {
                None => {
                    player_transform.translation =
                        position + movement_left + Vec3::new(0.0, 0.495, 0.0);
                    break;
                }
                Some((collision_entity, toi)) => {
                    if toi.status != Converged {
                        let unstuck_vector =
                            transforms.get(collision_entity).unwrap().translation - position;
                        transforms.get_mut(entity_player).unwrap().translation -=
                            unstuck_vector.normalize() * 0.01;
                        fps_camera.velocity = Vec3::new(0.0, 0.0, 0.0);
                        break;
                    }
                    movement_left -= movement_left.dot(toi.normal1) * toi.normal1;
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
                filter,
            ) {
                transforms.get_mut(entity_player).unwrap().translation -=
                    Vec3::new(0.0, toi.toi - leg_height, 0.0);
                fps_camera.velocity.y = 0.0;
            }
        }
    }
}
