mod bezier;
mod point;
mod point_cloud;

use crate::point_cloud::PointCloud;
use glium::{implement_vertex, Surface};
use point::Points;

type Position = [f32; 2];

#[derive(Copy, Clone)]
struct Vertex {
    position: Position,
}
implement_vertex!(Vertex, position);

fn main() {
    let event_loop = winit::event_loop::EventLoopBuilder::new()
        .build()
        .expect("event loop building");
    let (window, display) = glium::backend::glutin::SimpleWindowBuilder::new().build(&event_loop);
    let window_size = window.inner_size();
    println!("{window_size:?}");
    let mut window_size = [window_size.width as f32, window_size.height as f32];

    #[rustfmt::skip]
    let control_positions = [
        [200.0, 240.0],
        [400.0, 456.0],
        [400.0, 24.0],
        [600.0, 240.0],
    ];

    let curve_points = bezier::generate_bezier_points(&control_positions);
    let curve_cloud = PointCloud::new(&display, &curve_points, 2.0);

    let mut control_points = Points::new(&display);
    control_points.points = control_positions.into_iter().collect();

    let _ = event_loop.run(move |event, window_target| {
        match event {
            winit::event::Event::WindowEvent { event, .. } => match event {
                winit::event::WindowEvent::CloseRequested => window_target.exit(),
                winit::event::WindowEvent::Resized(new_size) => {
                    window_size = [new_size.width as f32, new_size.height as f32]
                },
                _ => (),
            },
            _ => (),
        };

        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 1.0, 1.0);

        curve_cloud.draw(&mut target, &window_size);
        control_points.draw(&mut target, &window_size);

        target.finish().unwrap();
    });
}
