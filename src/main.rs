mod bezier;
mod hidden;
mod matrix;
mod my_time;
mod position;
mod rendering;
mod selection;

use bevy::{
    prelude::PluginGroup,
    window::{Window, WindowPlugin, WindowResolution},
    DefaultPlugins,
};
use bezier::BezierPlugin;
use my_time::TimePlugin;
use rendering::RenderingPlugin;
use selection::SelectionPlugin;

fn main() {
    let mut app = bevy::prelude::App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            resolution: WindowResolution::new(800.0, 480.0).with_scale_factor_override(1.),
            resizable: true,
            present_mode: bevy::window::PresentMode::AutoNoVsync,
            ..Default::default()
        }),
        ..Default::default()
    }));
    app.add_plugins((TimePlugin, SelectionPlugin, RenderingPlugin, BezierPlugin));

    app.run();
}
