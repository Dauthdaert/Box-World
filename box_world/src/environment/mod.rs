use bevy::prelude::*;
use bevy_atmosphere::prelude::*;

use crate::states::GameStates;

mod day_night_cycle;

pub struct EnvironmentPlugin;

impl Plugin for EnvironmentPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AtmospherePlugin);

        app.add_systems(Startup, day_night_cycle::setup_daylight_cycle);
        app.add_systems(
            OnEnter(GameStates::InGame),
            day_night_cycle::toggle_daylight_cycle,
        );
        app.add_systems(
            Update,
            day_night_cycle::daylight_cycle.run_if(in_state(GameStates::InGame)),
        );
        app.add_systems(
            OnExit(GameStates::InGame),
            day_night_cycle::toggle_daylight_cycle,
        );
    }
}
