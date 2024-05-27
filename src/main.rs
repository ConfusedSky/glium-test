mod bezier;
mod mouse;
mod position;
mod rendering;
mod selection;

use std::time::SystemTime;

use bevy_ecs::{
    schedule::{IntoSystemConfigs, Schedule},
    system::Resource,
    world,
};

use crate::{
    bezier::update_bezier_curve,
    mouse::{MouseButtons, MousePosition},
    position::Position,
    rendering::point::FrameData,
    selection::{grab_selection, mouse_moved, HeldItems},
};

#[derive(Resource, Default)]
struct System {
    elapsed: f64,
}

fn main() {
    let event_loop = winit::event_loop::EventLoopBuilder::new()
        .build()
        .expect("event loop building");
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    let (window, display) = glium::backend::glutin::SimpleWindowBuilder::new().build(&event_loop);
    let window_size = window.inner_size();
    println!("{window_size:?}");
    let mut window_size = Position::from([window_size.width as f32, window_size.height as f32]);

    let timer = SystemTime::now();
    let mut renderer = rendering::renderer::Renderer::new(display);

    let mut world = world::World::new();
    let initialize_points = world.register_system(bezier::initialize_bezier_curve);
    world
        .run_system(initialize_points)
        .expect("Control points weren't successfully initialized");
    world.init_resource::<MousePosition>();
    world.init_resource::<MouseButtons>();
    world.init_resource::<HeldItems>();
    world.init_resource::<System>();
    world.init_resource::<FrameData>();

    let mut schedule: Schedule = Default::default();
    schedule.add_systems((mouse_moved, grab_selection));
    schedule.add_systems(update_bezier_curve.after(mouse_moved));

    let _ = event_loop.run(move |event, window_target| {
        match event {
            winit::event::Event::WindowEvent { event, .. } => match event {
                winit::event::WindowEvent::CloseRequested => window_target.exit(),
                winit::event::WindowEvent::Resized(new_size) => {
                    // TODO: Make the it render the original 800x480 in centered
                    // TODO: Zoom in the camera appropriately so the original 800x480 fits the screen as closely
                    //       as possible
                    window_size = Position::from([new_size.width as f32, new_size.height as f32]);
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

        renderer.draw(&mut world, &window_size);

        let mut mouse_buttons = world.resource_mut::<MouseButtons>();
        if mouse_buttons.needs_end_frame() {
            // We want to make sure we don't trigger change detection every frame
            mouse_buttons.end_frame();
        }

        world.clear_trackers();
    });
}
