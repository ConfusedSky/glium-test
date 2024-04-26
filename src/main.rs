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
    let window_size = [window_size.width as f32, window_size.height as f32];

    #[rustfmt::skip]
    let control_positions = [
        [-0.5, 0.0],
        [-0.0, 0.9],
        [0.0, -0.9],
        [0.5, 0.0],
    ];

    let curve_points = bezier::generate_bezier_points(&control_positions);
    let curve_cloud = PointCloud::new(&display, &curve_points, 2.0);

    let mut control_points = Points::new(&display);
    control_points.points = control_positions
        .iter()
        .map(|[x, y]| [x * window_size[0], y * window_size[1]])
        .collect();

    let _ = event_loop.run(move |event, window_target| {
        match event {
            winit::event::Event::WindowEvent { event, .. } => match event {
                winit::event::WindowEvent::CloseRequested => window_target.exit(),
                _ => (),
            },
            _ => (),
        };

        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 1.0, 1.0);

        curve_cloud.draw(&mut target);
        control_points.draw(&mut target, &window_size);

        target.finish().unwrap();
    });
}
