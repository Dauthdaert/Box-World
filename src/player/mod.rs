use crate::{chunk::LoadPoint, states::GameStates};
use bevy::prelude::*;
use bevy_rapier3d::prelude::{
    Collider, ColliderMassProperties, CollidingEntities, CollisionGroups, Friction, Group,
    LockedAxes, NoUserData, RapierConfiguration, RapierPhysicsPlugin, RigidBody, Velocity,
};

use self::input::MouseSensitivity;

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

        app.add_system(input::spawn.in_schedule(OnEnter(GameStates::InGame)));

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

#[derive(Component)]
struct Player;

#[derive(Bundle)]
struct PlayerBundle {
    pub player: Player,
    pub load_point: LoadPoint,
    #[bundle]
    pub collider: ColliderBundle,
    #[bundle]
    pub spatial: SpatialBundle,
}

impl Default for PlayerBundle {
    fn default() -> Self {
        Self {
            player: Player,
            load_point: LoadPoint,
            collider: ColliderBundle::default(),
            spatial: SpatialBundle {
                transform: Transform::from_xyz(10000., 400., 10000.),
                ..default()
            },
        }
    }
}

#[derive(Bundle)]
struct ColliderBundle {
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
