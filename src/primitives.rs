use crate::{position::Position, Vertex};
use glium::{
    dynamic_uniform, glutin::surface::WindowSurface, Display, DrawParameters, Frame, Program,
    Surface, VertexBuffer,
};

#[derive(Clone, Copy)]
pub enum PrimitiveType {
    Point,
    Line,
    LineStrip,
}

impl From<PrimitiveType> for glium::index::PrimitiveType {
    fn from(value: PrimitiveType) -> Self {
        match value {
            PrimitiveType::Point => Self::Points,
            PrimitiveType::Line => Self::LinesList,
            PrimitiveType::LineStrip => Self::LineStrip,
        }
    }
}

pub struct PrimitivesRenderer {
    program: Program,
}

impl PrimitivesRenderer {
    pub fn new(display: &Display<WindowSurface>) -> Self {
        let vertex_shader_src = r#"#version 400

            in vec2 position;

            uniform vec2 window_size;

            void main() {
                gl_Position = vec4(position * 2 / window_size - 1.0, 0.0, 1.0);
                gl_Position.y = -gl_Position.y;
            }
        "#;

        let fragment_shader_src = r#"#version 400

            out vec4 color;

            void main() {
                color = vec4(1.0, 0.0, 0.0, 1.0);
            }
        "#;

        let source = glium::program::SourceCode {
            vertex_shader: vertex_shader_src,
            fragment_shader: fragment_shader_src,
            geometry_shader: None,
            tessellation_control_shader: None,
            tessellation_evaluation_shader: None,
        };

        let program = glium::Program::new(display, source).unwrap();

        Self { program }
    }

    pub fn draw(&self, target: &mut Frame, data: &PrimitivesData, screen_size: &Position) {
        let uniforms = dynamic_uniform! {
            window_size: screen_size,
        };

        target
            .draw(
                &data.buffer,
                &glium::index::NoIndices(data.primitive_type.into()),
                &self.program,
                &uniforms,
                &data.params,
            )
            .unwrap();
    }
}

pub struct PrimitivesData<'a> {
    buffer: VertexBuffer<Vertex>,
    params: DrawParameters<'a>,
    primitive_type: PrimitiveType,
}

impl<'a> PrimitivesData<'a> {
    pub fn new(
        display: &Display<WindowSurface>,
        points: &[Vertex],
        primitive_type: PrimitiveType,
        size: f32,
    ) -> Self {
        let buffer = glium::VertexBuffer::new(display, &points).unwrap();


        let params = DrawParameters {
            point_size: Some(size),
            line_width: Some(size),
            ..Default::default()
        };

        Self {
            buffer,
            params,
            primitive_type,
        }
    }

    pub fn set_points(&mut self, display: &Display<WindowSurface>, points: &[Vertex]) {
        self.buffer = glium::VertexBuffer::new(display, &points).unwrap();
    }

}
