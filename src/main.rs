use glium::{implement_vertex, Surface};

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

fn main() {
    let event_loop = winit::event_loop::EventLoopBuilder::new()
        .build()
        .expect("event loop building");
    let (_window, display) = glium::backend::glutin::SimpleWindowBuilder::new().build(&event_loop);

    #[rustfmt::skip]
    let verticies = [
        [-0.5, -0.5],
        [-0.75, 0.5],
        [0.75, 0.5],
        [0.5, -0.5],
    ];

    let shape: Vec<_> = verticies
        .into_iter()
        .map(|x| Vertex { position: x })
        .collect();
    let control_points_buffer = glium::VertexBuffer::new(&display, &shape).unwrap();

    let subdivisions = 60;
    let mut shape_points = vec![];

    for i in 0..subdivisions + 1 {
        let t = i as f64 / subdivisions as f64;
        let point = bezier(verticies[0], verticies[1], verticies[2], verticies[3], t);
        shape_points.push(point);
    }
    let shape: Vec<_> = shape_points
        .into_iter()
        .map(|x| Vertex { position: x })
        .collect();
    let curve_buffer = glium::VertexBuffer::new(&display, &shape).unwrap();

    let indices = glium::index::NoIndices(glium::index::PrimitiveType::LineStrip);
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
        glium::Program::from_source(&display, vertex_shader_src, fragment_shader_src, None)
            .unwrap();

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
        target
            .draw(
                &control_points_buffer,
                &indices,
                &program,
                &glium::uniforms::EmptyUniforms,
                &Default::default(),
            )
            .unwrap();
        target
            .draw(
                &curve_buffer,
                &indices,
                &program,
                &glium::uniforms::EmptyUniforms,
                &Default::default(),
            )
            .unwrap();

        target.finish().unwrap();
    });
}
