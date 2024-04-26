use crate::Vertex;
use glium::{
    glutin::surface::WindowSurface, Display, DrawParameters, Frame, Program, Surface, VertexBuffer,
};

pub struct PointCloud<'a> {
    buffer: VertexBuffer<Vertex>,
    program: Program,
    params: DrawParameters<'a>,
}

impl<'a> PointCloud<'a> {
    pub fn new(display: &Display<WindowSurface>, points: &[Vertex], size: f32) -> Self {
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

    pub fn draw(&self, target: &mut Frame) {
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
