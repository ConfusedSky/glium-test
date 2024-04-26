use crate::{Position, PositionU};
use glium::{
    dynamic_uniform, glutin::surface::WindowSurface, implement_vertex, index::PrimitiveType, Display, Frame, IndexBuffer, Program, Surface, VertexBuffer
};

#[derive(Clone, Copy)]
struct Vertex {
    position: Position,
    uv: Position,
}
implement_vertex!(Vertex, position, uv);

pub struct Points {
    buffer: VertexBuffer<Vertex>,
    indicies: IndexBuffer<u16>,
    program: Program,
    points: Vec<PositionU>,
}

impl Points {
    pub fn new(display: &Display<WindowSurface>) -> Self {
        let points = vec![
            Vertex {
                position: [-1.0, -1.0],
                uv: [0.0, 0.0],
            },
            Vertex {
                position: [1.0, -1.0],
                uv: [1.0, 0.0],
            },
            Vertex {
                position: [-1.0, 1.0],
                uv: [0.0, 1.0],
            },
            Vertex {
                position: [1.0, 1.0],
                uv: [1.0, 1.0],
            },
        ];
        let buffer = glium::VertexBuffer::new(display, &points).unwrap();
        let index_buffer =
            glium::IndexBuffer::new(display, PrimitiveType::TrianglesList, &[0u16, 1, 2, 1, 3, 2]).unwrap();

        let vertex_shader_src = r#"#version 400
            in vec2 position;
            in vec2 uv;

            uniform vec2 scale;

            varying lowp vec2 texcoord;

            void main() {
                gl_Position = vec4(position * scale, 0.0, 1.0);
                texcoord = uv;
            }
        "#;

        let fragment_shader_src = r#"#version 400
            out vec4 color;

            varying lowp vec2 texcoord;

            void main() {
                color = vec4(1.0, texcoord, 1.0);
            }
        "#;

        let program =
            glium::Program::from_source(display, vertex_shader_src, fragment_shader_src, None)
                .unwrap();

        Self {
            buffer,
            indicies: index_buffer,
            program,
            points: vec![],
        }
    }

    pub fn draw(&self, target: &mut Frame, screen_size: &PositionU) {
        let size = 12.0;
        let scale = [
            size / screen_size[0] as f32, 
            size / screen_size[1] as f32, 
        ];
        let uniforms = dynamic_uniform! {
            scale: &scale,
        };
        target
            .draw(
                &self.buffer,
                &self.indicies,
                &self.program,
                &uniforms,
                &Default::default(),
            )
            .unwrap();
    }
}
