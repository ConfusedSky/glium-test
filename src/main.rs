mod bezier;
mod mouse;
mod position;
mod rendering;
mod selection;

use std::time::SystemTime;

use bevy::{
    app::{First, Startup, Update},
    ecs::{
        schedule::IntoSystemConfigs,
        system::{ResMut, Resource},
    },
    prelude::PluginGroup,
    window::{Window, WindowPlugin, WindowResolution},
    DefaultPlugins,
};
use bezier::update_bezier_curve;
use mouse::MousePlugin;
use rendering::RenderingPlugin;
use selection::{grab_selection, mouse_moved, HeldItems};

#[derive(Resource)]
struct System {
    elapsed: f64,
    timer: SystemTime,
}

impl Default for System {
    fn default() -> Self {
        Self {
            elapsed: 0.0,
            timer: SystemTime::now(),
        }
    }
}

fn main() {
    let mut app = bevy::prelude::App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            resolution: WindowResolution::new(800.0, 480.0).with_scale_factor_override(1.),
            resizable: true,
            present_mode: bevy::window::PresentMode::AutoNoVsync,
            ..Default::default()
        }),
        ..Default::default() // Make sure we have access to the winit windows before initializing the rendering windows
    }));
    app.add_plugins((RenderingPlugin, MousePlugin));

    app.add_systems(Startup, bezier::initialize_bezier_curve);
    app.init_resource::<HeldItems>();
    app.init_resource::<System>();

    app.add_systems(Update, (mouse_moved, grab_selection));
    app.add_systems(Update, update_bezier_curve.after(mouse_moved));

    app.add_systems(First, |mut system: ResMut<System>| {
        system.elapsed = system.timer.elapsed().map(|s| s.as_secs_f64()).unwrap();
    });

    app.run();
}
