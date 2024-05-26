mod bezier;
mod mouse;
mod point;
mod position;
mod primitives;
mod renderer;
mod selection;

use std::time::SystemTime;

use winit::event::MouseButton;

use bevy_ecs::{schedule::{IntoSystemConfigs, Schedule}, world};

use crate::{
    bezier::update_bezier_curve, mouse::{MouseButtons, MousePosition}, selection::{grab_selection, mouse_moved, HeldItems}
};

fn main() {
    let event_loop = winit::event_loop::EventLoopBuilder::new()
        .build()
        .expect("event loop building");
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    let (window, display) = glium::backend::glutin::SimpleWindowBuilder::new().build(&event_loop);
    let window_size = window.inner_size();
    println!("{window_size:?}");
    let mut window_size = [window_size.width as f32, window_size.height as f32].into();

    let mut control_points = {
        #[rustfmt::skip]
        let control_positions: Vec<_> = [
            [200.0, 240.0],
            [400.0, 456.0],
            [400.0, 24.0],
            [600.0, 240.0],
        ].into_iter().map(Into::into).collect();

        let control_points = point::Collection::new(&control_positions, None);

        control_points
    };
    let follow_points = point::Collection::new(&[], Some(10.0));

    let (bezier_curve, lines) = {
        let control_points = control_points.get_points();

        let curve_points = bezier::generate_bezier_points(control_points);
        let bezier_curve =
            primitives::Primatives::new(&curve_points, primitives::Type::LineStrip, 2.0);

        let line_points: Vec<primitives::Vertex> = control_points
            .into_iter()
            .map(|x| *x)
            .map(Into::into)
            .collect();
        // Todo style this better
        let lines = primitives::Primatives::new(&line_points, primitives::Type::Line, 2.0);

        (bezier_curve, lines)
    };

    let timer = SystemTime::now();
    let mut renderer = renderer::Renderer::new(display);

    let mut world = world::World::new();
    let initialize_points = world.register_system(bezier::initialize_bezier_curve);
    world
        .run_system(initialize_points)
        .expect("Control points weren't successfully initialized");
    world.insert_resource::<MousePosition>(Default::default());
    world.insert_resource::<MouseButtons>(Default::default());
    world.insert_resource::<HeldItems>(Default::default());

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
                    window_size = [new_size.width as f32, new_size.height as f32].into();
                }
                winit::event::WindowEvent::CursorMoved { position, .. } => {
                    let position = [position.x as f32, position.y as f32].into();
                    let mut mouse_position = world.resource_mut::<MousePosition>();
                    mouse_position.update(position);

                    // if control_points.mouse_moved(&position, &previous_position) {
                    // let control_points = control_points.get_points();

                    // let curve_points = bezier::generate_bezier_points(control_points);
                    // bezier_curve.set_points(&curve_points);

                    // let line_points: Vec<Vertex> = control_points
                    // .into_iter()
                    // .map(|x| Vertex {
                    // position: (*x).into(),
                    // })
                    // .collect();
                    // lines.set_points(&line_points);
                    // }
                }
                winit::event::WindowEvent::MouseInput { state, button, .. } => {
                    let mut mouse_buttons = world.resource_mut::<MouseButtons>();
                    mouse_buttons.update(state, button);
                }
                _ => (),
            },
            _ => (),
        };

        schedule.run(&mut world);

        // let elapsed = timer.elapsed().unwrap().as_secs_f64() / 4.0;

        // let p = bezier::generate_bezier_points_with_offset(
        // control_points.get_points(),
        // Some(5),
        // Some(elapsed),
        // );
        // follow_points.set_points(&p);

        renderer.draw(&mut world, &window_size);

        let mut mouse_buttons = world.resource_mut::<MouseButtons>();
        if mouse_buttons.needs_end_frame() {
            // We want to make sure we don't trigger change detection every frame
            mouse_buttons.end_frame();
        }
    });
}
