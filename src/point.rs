use crate::position::Position;
use glium::{
    dynamic_uniform, glutin::surface::WindowSurface, implement_vertex, index::PrimitiveType,
    Display, DrawParameters, Frame, IndexBuffer, Program, Surface, VertexBuffer,
};

#[derive(Clone, Copy)]
struct Vertex {
    position: Position,
    uv: Position,
}
implement_vertex!(Vertex, position, uv);

pub struct Points<'a> {
    buffer: VertexBuffer<Vertex>,
    indicies: IndexBuffer<u16>,
    program: Program,
    params: DrawParameters<'a>,
    pub points: Vec<Position>,
}

impl<'a> Points<'a> {
    const SIZE: f32 = 15.0;

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
        let index_buffer = glium::IndexBuffer::new(
            display,
            PrimitiveType::TrianglesList,
            &[0u16, 1, 2, 1, 3, 2],
        )
        .unwrap();

        let vertex_shader_src = r#"#version 400
            in vec2 position;
            in vec2 uv;

            uniform vec2 window_size;
            uniform float point_size;
            uniform vec2 offset;

            varying lowp vec2 texcoord;

            void main() {
                gl_Position = vec4((position * point_size / 2 + offset) * 2 / window_size - 1.0, 0.0, 1.0);
                gl_Position.y = -gl_Position.y;
                texcoord = uv;
            }
        "#;

        let fragment_shader_src = r#"#version 400
            out vec4 color;

            uniform vec3 point_color;

            varying lowp vec2 texcoord;

            float circle(in vec2 _st, in float _radius){
                vec2 dist = _st-vec2(0.5);
                return 1.-smoothstep(_radius-(_radius*0.01),
                                    _radius+(_radius*0.01),
                                    dot(dist,dist)*4.0);
            }

            void main() {
                float a = circle(texcoord, 1);
                color = vec4(point_color, a);
            }
        "#;

        let program =
            glium::Program::from_source(display, vertex_shader_src, fragment_shader_src, None)
                .unwrap();

        let params = DrawParameters {
            blend: glium::Blend::alpha_blending(),
            ..Default::default()
        };

        Self {
            buffer,
            indicies: index_buffer,
            program,
            params,
            points: vec![],
        }
    }

    pub fn hovered(&self, position: &Position) {
        let size = Self::SIZE + 5.0;
        let size_squared = size.powi(2);

        for point in &self.points {
            let distance_squared = crate::position::distance_squared(position, point);
            if distance_squared < size_squared {
                println!("Hovered over: {point:?}");
            }
        }
    }

    pub fn draw(&self, target: &mut Frame, screen_size: &Position) {
        let color: [f32; 3] = [1.0, 0.0, 0.0];

        for offset in &self.points {
            let uniforms = dynamic_uniform! {
                point_size: &Self::SIZE,
                window_size: screen_size,
                point_color: &color,
                offset: offset,
            };
            target
                .draw(
                    &self.buffer,
                    &self.indicies,
                    &self.program,
                    &uniforms,
                    &self.params,
                )
                .unwrap();
            }
    }
}
