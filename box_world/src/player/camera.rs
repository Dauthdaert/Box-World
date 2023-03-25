use std::f32::consts::FRAC_PI_2;

use bevy::prelude::{Component, Resource};
use bevy_rapier3d::prelude::Vect;

#[derive(Component)]
pub struct FPSCamera {
    pub phi: f32,
    pub theta: f32,
    pub velocity: Vect,
}

impl Default for FPSCamera {
    fn default() -> Self {
        Self {
            phi: 0.0,
            theta: FRAC_PI_2,
            velocity: Vect::ZERO,
        }
    }
}

#[derive(Resource)]
pub struct MouseSensitivity(pub f32);
