use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use super::Player;
use crate::chunk::LoadPoint;

#[derive(Bundle)]
pub struct PreSpawnPlayerBundle {
    pub player: Player,
    pub name: Name,
    pub load_point: LoadPoint,
    pub spatial: SpatialBundle,
}

impl PreSpawnPlayerBundle {
    pub fn new(horizontal_view_distance: u32, vertical_view_distance: u32, position: Vec3) -> Self {
        Self {
            player: Player,
            name: Name::new("Player"),
            load_point: LoadPoint {
                horizontal: horizontal_view_distance,
                vertical: vertical_view_distance,
            },
            spatial: SpatialBundle {
                transform: Transform::from_translation(position),
                ..default()
            },
        }
    }
}

#[derive(Bundle, Default)]
pub struct PostSpawnPlayerBundle {
    pub collider: ColliderBundle,
}

#[derive(Bundle)]
pub struct ColliderBundle {
    pub colliding_entities: CollidingEntities,
    pub collider: Collider,
    pub rigid_body: RigidBody,
    pub velocity: Velocity,
    pub friction: Friction,
    pub density: ColliderMassProperties,
    pub rotation_constraints: LockedAxes,
    pub collision_groups: CollisionGroups,
}

impl Default for ColliderBundle {
    fn default() -> Self {
        Self {
            collider: Collider::capsule_y(2., 1.5),
            rigid_body: RigidBody::KinematicVelocityBased,
            rotation_constraints: LockedAxes::ROTATION_LOCKED,
            collision_groups: CollisionGroups::new(
                Group::GROUP_1,
                Group::from_bits_truncate(Group::GROUP_2.bits()),
            ),
            colliding_entities: CollidingEntities::default(),
            velocity: Velocity::default(),
            friction: Friction::default(),
            density: ColliderMassProperties::default(),
        }
    }
}
