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
        world::World,
    },
    prelude::PluginGroup,
    window::{Window, WindowPlugin},
    winit::WinitWindows,
    DefaultPlugins,
};
use rendering::renderer::RenderingPlugin;

use crate::{
    bezier::update_bezier_curve,
    mouse::{MouseButtons, MousePosition},
    position::Position,
    rendering::renderer::WindowSize,
    selection::{grab_selection, mouse_moved, HeldItems},
};

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
            resolution: (800., 480.).into(),
            resizable: true,
            present_mode: bevy::window::PresentMode::AutoNoVsync,
            ..Default::default()
        }),
        ..Default::default() // Make sure we have access to the winit windows before initializing the rendering windows
    }));
    app.add_plugins(RenderingPlugin);

    app.add_systems(Startup, |world: &mut World| {
        let winit_data = world.non_send_resource::<WinitWindows>();
        assert_eq!(winit_data.windows.len(), 1);

        let window = winit_data.windows.values().next().unwrap();
        let (width, height): (f32, f32) = window.inner_size().into();
        let window_size = Position::from([width, height]);
        world.insert_resource(WindowSize(window_size));
    });
    app.add_systems(Startup, bezier::initialize_bezier_curve);
    app.init_resource::<MousePosition>();
    app.init_resource::<MouseButtons>();
    app.init_resource::<HeldItems>();
    app.init_resource::<System>();

    app.add_systems(Update, (mouse_moved, grab_selection));
    app.add_systems(Update, update_bezier_curve.after(mouse_moved));

    app.add_systems(First, |mut system: ResMut<System>| {
        system.elapsed = system.timer.elapsed().map(|s| s.as_secs_f64()).unwrap();
    });

    app.run();
}
