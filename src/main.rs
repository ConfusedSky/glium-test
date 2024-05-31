mod bezier;
mod my_time;
mod position;
mod rendering;
mod selection;

use bevy::{
    app::{Startup, Update}, ecs::schedule::IntoSystemConfigs, prelude::PluginGroup, window::{Window, WindowPlugin, WindowResolution}, DefaultPlugins
};
use bezier::update_bezier_curve;
use my_time::TimePlugin;
use rendering::RenderingPlugin;
use selection::{grab_selection, mouse_moved, HeldItems};

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
    app.add_plugins(RenderingPlugin);
    app.add_plugins(TimePlugin);

    app.add_systems(Startup, bezier::initialize_bezier_curve);
    app.init_resource::<HeldItems>();

    app.add_systems(Update, (mouse_moved, grab_selection));
    app.add_systems(Update, update_bezier_curve.after(mouse_moved));

    app.run();
}
