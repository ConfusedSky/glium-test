use std::time::SystemTime;

use bevy::{
    app::{First, Plugin},
    ecs::system::{ResMut, Resource},
};

#[derive(Resource)]
pub struct Time {
    pub elapsed: f64,
    timer: SystemTime,
}

impl Default for Time {
    fn default() -> Self {
        Self {
            elapsed: 0.0,
            timer: SystemTime::now(),
        }
    }
}

pub struct TimePlugin;

impl Plugin for TimePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<Time>();
        app.add_systems(First, |mut system: ResMut<Time>| {
            system.elapsed = system.timer.elapsed().map(|s| s.as_secs_f64()).unwrap();
        });
    }
}
