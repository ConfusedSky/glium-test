use glium::{
    glutin::surface::WindowSurface, implement_vertex, Display, DrawParameters, Frame, Program,
    Surface, VertexBuffer,
};

type Point = [f32; 2];

#[derive(Copy, Clone)]
struct Vertex {
    position: Point,
}
implement_vertex!(Vertex, position);

fn lerp(p1: Point, p2: Point, t: f64) -> Point {
    let x = p1[0] as f64 * (1. - t) + p2[0] as f64 * t;
    let y = p1[1] as f64 * (1. - t) + p2[1] as f64 * t;
    [x as f32, y as f32]
}

fn bezier(p1: Point, p2: Point, p3: Point, p4: Point, t: f64) -> Point {
    let a = lerp(p1, p2, t);
    let b = lerp(p2, p3, t);
    let c = lerp(p3, p4, t);
    let d = lerp(a, b, t);
    let e = lerp(b, c, t);
    lerp(d, e, t)
}

fn generate_bezier_points(control_points: &[[f32; 2]; 4]) -> Vec<Vertex> {
    let subdivisions = 60;
    let mut shape_points = Vec::with_capacity(subdivisions);

    for i in 0..subdivisions + 1 {
        let t = i as f64 / subdivisions as f64;
        let point = bezier(
            control_points[0],
            control_points[1],
            control_points[2],
            control_points[3],
            t,
        );
        shape_points.push(Vertex { position: point });
    }

    shape_points
}

struct PointCloud<'a> {
    buffer: VertexBuffer<Vertex>,
    program: Program,
    params: DrawParameters<'a>,
}

impl<'a> PointCloud<'a> {
    fn new(display: &Display<WindowSurface>, points: &[Vertex], size: f32) -> Self {
        let buffer = glium::VertexBuffer::new(display, &points).unwrap();

        let vertex_shader_src = r#"#version 400

            in vec2 position;

            void main() {
                gl_Position = vec4(position, 0.0, 1.0);
            }
        "#;

        let fragment_shader_src = r#"#version 400

            out vec4 color;

            void main() {
                color = vec4(1.0, 0.0, 0.0, 1.0);
            }
        "#;

        let program =
            glium::Program::from_source(display, vertex_shader_src, fragment_shader_src, None)
                .unwrap();

        let params = DrawParameters {
            point_size: Some(size),
            ..Default::default()
        };

        Self {
            buffer,
            program,
            params,
        }
    }

    fn draw(&self, target: &mut Frame) {
        target
            .draw(
                &self.buffer,
                &glium::index::NoIndices(glium::index::PrimitiveType::Points),
                &self.program,
                &glium::uniforms::EmptyUniforms,
                &self.params,
            )
            .unwrap();
    }
}

fn main() {
    let event_loop = winit::event_loop::EventLoopBuilder::new()
        .build()
        .expect("event loop building");
    let (_window, display) = glium::backend::glutin::SimpleWindowBuilder::new().build(&event_loop);

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

    let curve_points = generate_bezier_points(&control_points);
    let curve_cloud = PointCloud::new(&display, &curve_points, 2.0);

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

        target.finish().unwrap();
    });
}
