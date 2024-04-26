mod bezier;
mod point;
mod point_cloud;

use glium::{implement_vertex, Surface};
use point::Points;
use crate::point_cloud::PointCloud;

type Position = [f32; 2];
type PositionU = [u32; 2];

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
    let window_size = [window_size.width, window_size.height];

    #[rustfmt::skip]
    let control_points = [
        [-0.5, 0.0],
        [-0.0, 0.9],
        [0.0, -0.9],
        [0.5, 0.0],
    ];

    let shape: Vec<_> = control_points
        .into_iter()
        .map(|x| Vertex { position: x })
        .collect();
    let control_points_cloud = PointCloud::new(&display, &shape, 12.0);

    let curve_points = bezier::generate_bezier_points(&control_points);
    let curve_cloud = PointCloud::new(&display, &curve_points, 2.0);

    let indicies = Points::new(&display);

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

        control_points_cloud.draw(&mut target);
        curve_cloud.draw(&mut target);
        indicies.draw(&mut target, &window_size);

        target.finish().unwrap();
    });
}
