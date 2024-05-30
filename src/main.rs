mod bezier;
mod mouse;
mod position;
mod rendering;
mod selection;

use std::{num::NonZeroU32, time::SystemTime};

use bevy::{
    ecs::{
        schedule::{IntoSystemConfigs, Schedule},
        system::Resource,
        world,
    },
    prelude::PluginGroup,
    window::{RequestRedraw, Window, WindowPlugin},
    winit::WinitWindows,
    DefaultPlugins,
};
use glutin::display::GetGlDisplay;
use raw_window_handle::HasRawWindowHandle;
use winit::raw_window_handle::HasWindowHandle;

use crate::{
    bezier::update_bezier_curve,
    mouse::{MouseButtons, MousePosition},
    position::Position,
    rendering::{
        point::PointsData,
        renderer::{render_system, WindowSize},
    },
    selection::{grab_selection, mouse_moved, HeldItems},
};

#[derive(Resource, Default)]
struct System {
    elapsed: f64,
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
        ..Default::default()
    }));
    app.update();

    {
        let event_loop = app
            .world
            .non_send_resource::<winit::event_loop::EventLoop<RequestRedraw>>();

        // First we start by opening a new Window
        let display_builder = glutin_winit::DisplayBuilder::new();
        let config_template_builder = glutin::config::ConfigTemplateBuilder::new();
        let (_, gl_config) = display_builder
            .build(&event_loop, config_template_builder, |mut configs| {
                // Just use the first configuration since we don't have any special preferences here
                configs.next().unwrap()
            })
            .unwrap();

        let winit_data = app.world.non_send_resource::<WinitWindows>();
        assert_eq!(winit_data.windows.len(), 1);

        let window = winit_data.windows.values().next().unwrap();
        let (width, height): (u32, u32) = window.inner_size().into();
    
        let display = gl_config.display();

        let raw_window_handle = window
            .window_handle().map(|handle| handle.as_raw()).ok();
    }
    app.run();

    let event_loop = winit::event_loop::EventLoopBuilder::new()
        .build()
        .expect("event loop building");
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    let (window, display) = glium::backend::glutin::SimpleWindowBuilder::new().build(&event_loop);
    let window_size = window.inner_size();
    println!("{window_size:?}");
    let window_size = Position::from([window_size.width as f32, window_size.height as f32]);

    let timer = SystemTime::now();
    let renderer = rendering::renderer::Renderer::new(display);

    let mut world = world::World::new();
    let initialize_points = world.register_system(bezier::initialize_bezier_curve);
    world
        .run_system(initialize_points)
        .expect("Control points weren't successfully initialized");
    world.init_resource::<MousePosition>();
    world.init_resource::<MouseButtons>();
    world.init_resource::<HeldItems>();
    world.init_resource::<System>();
    world.init_resource::<PointsData>();
    world.insert_resource(WindowSize(window_size));
    world.insert_non_send_resource(renderer);

    let mut schedule: Schedule = Default::default();
    schedule.add_systems((mouse_moved, grab_selection));
    schedule.add_systems(update_bezier_curve.after(mouse_moved));

    let mut render_schedule: Schedule = Default::default();
    render_schedule.add_systems(render_system);

    let _ = event_loop.run(move |event, window_target| {
        match event {
            winit::event::Event::WindowEvent { event, .. } => match event {
                winit::event::WindowEvent::CloseRequested => window_target.exit(),
                winit::event::WindowEvent::Resized(new_size) => {
                    // TODO: Make the it render the original 800x480 in centered
                    // TODO: Zoom in the camera appropriately so the original 800x480 fits the screen as closely
                    //       as possible
                    world.resource_mut::<WindowSize>().0 =
                        Position::from([new_size.width as f32, new_size.height as f32])
                }
                winit::event::WindowEvent::CursorMoved { position, .. } => {
                    let position = [position.x as f32, position.y as f32].into();
                    let mut mouse_position = world.resource_mut::<MousePosition>();
                    mouse_position.update(position);
                }
                winit::event::WindowEvent::MouseInput { state, button, .. } => {
                    let mut mouse_buttons = world.resource_mut::<MouseButtons>();
                    mouse_buttons.update(state, button);
                }
                _ => (),
            },
            _ => (),
        };

        world.resource_mut::<System>().elapsed = timer.elapsed().unwrap().as_secs_f64();

        schedule.run(&mut world);
        render_schedule.run(&mut world);

        world.clear_trackers();

        let mut mouse_buttons = world.resource_mut::<MouseButtons>();
        if mouse_buttons.needs_end_frame() {
            // We want to make sure we don't trigger change detection every frame
            mouse_buttons.end_frame();
        }
    });
}
