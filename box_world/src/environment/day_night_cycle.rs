use std::{f32::consts::PI, time::Duration};

use bevy::prelude::*;
use bevy_atmosphere::prelude::*;

#[derive(Component)]
pub struct Sun;

#[derive(Resource)]
pub struct CycleTimer {
    timer: Timer,
    current_time: f32,
}

impl CycleTimer {
    pub fn new(current_time: f32) -> Self {
        let mut timer = Timer::from_seconds(0.1, TimerMode::Repeating);
        timer.set_elapsed(Duration::from_secs_f32(0.1));
        timer.pause();

        Self {
            timer,
            current_time,
        }
    }

    pub fn tick(&mut self, duration: Duration) {
        if !self.timer.paused() {
            self.timer.tick(duration);

            // Go around the unit circle
            self.current_time = (self.current_time + (duration.as_secs_f32() / 200.0)) % (PI * 2.0);
        }
    }

    pub fn toggle_paused(&mut self) {
        if self.timer.paused() {
            self.timer.unpause();
        } else {
            self.timer.pause()
        }
    }
}

pub fn setup_daylight_cycle(mut commands: Commands) {
    let mut atmosphere_model = Nishita::default();
    let t: f32 = 10.0;
    atmosphere_model.sun_position = Vec3::new(t, t.sin(), t.cos());

    commands.insert_resource(AtmosphereModel::new(atmosphere_model));
    commands.insert_resource(CycleTimer::new(t));

    commands.spawn((
        DirectionalLightBundle {
            directional_light: DirectionalLight {
                // TODO: Enable this when we have more frame budget
                //shadows_enabled: true,
                ..default()
            },
            ..default()
        },
        Sun,
    ));

    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.2,
    });
}

pub fn toggle_daylight_cycle(mut cycle: ResMut<CycleTimer>) {
    cycle.toggle_paused();
}

pub fn daylight_cycle(
    mut atmosphere: AtmosphereMut<Nishita>,
    mut light: Query<(&mut Transform, &mut DirectionalLight), With<Sun>>,
    mut cycle: ResMut<CycleTimer>,
    time: Res<Time>,
) {
    cycle.tick(time.delta());

    if cycle.timer.finished() {
        let t = cycle.current_time;
        atmosphere.sun_position = Vec3::new(0., t.sin(), t.cos());

        {
            let (mut transform, mut directional) = light.single_mut();
            transform.rotation = Quat::from_rotation_x(-t.sin().atan2(t.cos()));
            directional.illuminance = t.sin().max(0.0).powf(2.0) * 100000.0;
        }
    }
}
