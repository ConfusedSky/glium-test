use glium::{implement_vertex, Surface};

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
}
implement_vertex!(Vertex, position);

fn main() {
    let event_loop = winit::event_loop::EventLoopBuilder::new()
        .build()
        .expect("event loop building");
    let (_window, display) = glium::backend::glutin::SimpleWindowBuilder::new().build(&event_loop);

    #[rustfmt::skip]
    let verticies = [
        [-0.5, -0.5],
        [-0.2, 0.5],
        [0.2, 0.5],
        [0.5, -0.5],
    ];

    let shape: Vec<_> = verticies
        .into_iter()
        .map(|x| Vertex { position: x })
        .collect();
    let control_points_buffer = glium::VertexBuffer::new(&display, &shape).unwrap();

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

        target.finish().unwrap();
    });
}
