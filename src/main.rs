mod bezier;
mod point;
mod position;
mod primitives;

use std::time::SystemTime;

use glium::{implement_vertex, Surface};
use winit::event::MouseButton;

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
}
implement_vertex!(Vertex, position);

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

        let control_points = point::Data::new(&control_positions, None);

        control_points
    };
    let mut follow_points = point::Data::new(&[], Some(10.0));

    let mut previous_position = None;

    let (mut curve_cloud, mut lines) = {
        let control_points = control_points.get_points();

        let curve_points = bezier::generate_bezier_points(control_points);
        let curve_cloud = primitives::Data::new(
            &display,
            &curve_points,
            primitives::Type::LineStrip,
            2.0,
        );

        let line_points: Vec<Vertex> = control_points
            .into_iter()
            .map(|x| Vertex {
                position: (*x).into(),
            })
            .collect();
        // Todo style this better
        let lines = primitives::Data::new(&display, &line_points, primitives::Type::Line, 2.0);

        (curve_cloud, lines)
    };

    let timer = SystemTime::now();
    let primitives_renderer = primitives::Renderer::new(&display);
    let point_renderer = point::Renderer::new(&display);

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
                    if control_points.mouse_moved(&position, &previous_position) {
                        let control_points = control_points.get_points();

                        let curve_points = bezier::generate_bezier_points(control_points);
                        curve_cloud.set_points(&display, &curve_points);

                        let line_points: Vec<Vertex> = control_points
                            .into_iter()
                            .map(|x| Vertex {
                                position: (*x).into(),
                            })
                            .collect();
                        lines.set_points(&display, &line_points);
                    }

                    previous_position = Some(position);
                }
                winit::event::WindowEvent::MouseInput { state, button, .. } => {
                    if button == MouseButton::Left {
                        if state.is_pressed() {
                            control_points.click();
                        } else {
                            control_points.release();
                        }
                    }
                }
                _ => (),
            },
            _ => (),
        };

        let elapsed = timer.elapsed().unwrap().as_secs_f64() / 4.0;
        let p = bezier::generate_bezier_points_with_offset(
            control_points.get_points(),
            Some(5),
            Some(elapsed),
        );
        follow_points.set_points(&p);

        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 1.0, 1.0);

        primitives_renderer.draw(&mut target, &lines, &window_size);
        point_renderer.draw(&mut target, &follow_points, &window_size);
        primitives_renderer.draw(&mut target, &curve_cloud, &window_size);
        point_renderer.draw(&mut target, &control_points, &window_size);

        target.finish().unwrap();
    });
}
