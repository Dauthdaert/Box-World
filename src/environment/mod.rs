use bevy::prelude::*;
use bevy_atmosphere::prelude::*;

use crate::states::GameStates;

mod day_night_cycle;

pub struct EnvironmentPlugin;

impl Plugin for EnvironmentPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(AtmospherePlugin);

        app.add_startup_system(day_night_cycle::setup_daylight_cycle);
        app.add_system(
            day_night_cycle::toggle_daylight_cycle.in_schedule(OnEnter(GameStates::InGame)),
        );
        app.add_system(day_night_cycle::daylight_cycle.run_if(in_state(GameStates::InGame)));
        app.add_system(
            day_night_cycle::toggle_daylight_cycle.in_schedule(OnExit(GameStates::InGame)),
        );
    }
}
