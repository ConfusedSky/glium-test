mod bezier;
mod point;
mod point_cloud;
mod position;

use crate::point_cloud::PointCloud;
use glium::{implement_vertex, Surface};
use point::Points;
use winit::event::MouseButton;

#[derive(Copy, Clone)]
struct Vertex {
    position: position::Position,
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

    let mut control_points = {
        #[rustfmt::skip]
        let control_positions = [
            [200.0, 240.0],
            [400.0, 456.0],
            [400.0, 24.0],
            [600.0, 240.0],
        ];

        let mut control_points = Points::new(&display);
        control_points.set_points(&control_positions);

        control_points
    };

    let curve_points = bezier::generate_bezier_points(control_points.get_points());
    let curve_cloud = PointCloud::new(&display, &curve_points, 2.0);

    let _ = event_loop.run(move |event, window_target| {
        match event {
            winit::event::Event::WindowEvent { event, .. } => match event {
                winit::event::WindowEvent::CloseRequested => window_target.exit(),
                winit::event::WindowEvent::Resized(new_size) => {
                    // TODO: Make the it render the original 800x480 in centered
                    // TODO: Zoom in the camera appropriately so the original 800x480 fits the screen as closely
                    //       as possible
                    window_size = [new_size.width as f32, new_size.height as f32]
                }
                winit::event::WindowEvent::CursorMoved { position, .. } => {
                    let position = [position.x as f32, position.y as f32];
                    control_points.mouse_moved(&position);
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

        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 1.0, 1.0);

        curve_cloud.draw(&mut target, &window_size);
        control_points.draw(&mut target, &window_size);

        target.finish().unwrap();
    });
}
