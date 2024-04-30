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

struct Point {
    position: Position,
    hovered: bool,
}

pub struct Points<'a> {
    // Quad buffer for rendering points
    buffer: VertexBuffer<Vertex>,
    // Indicies of the quad buffer
    indicies: IndexBuffer<u16>,
    // Program that renders a circle on a quad mesh
    program: Program,
    // This is here so we properly set alpha blending
    params: DrawParameters<'a>,
    // Point objects for each point to render
    points: Vec<Point>,

    // This is just a buffer that is used when calling get_points
    // Should not be used for anything else
    positions: Vec<Position>,
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
            positions: vec![],
        }
    }

    pub fn set_points(&mut self, points: &[Position]) {
        self.points = points
            .iter()
            .map(|x| Point {
                position: *x,
                hovered: false,
            })
            .collect();
    }

    pub fn get_points(&mut self) -> &[Position] {
        self.positions = self.points.iter().map(|x| x.position).collect();
        &self.positions
    }

    pub fn hovered(&mut self, position: &Position) {
        let size = Self::SIZE + 5.0;
        let size_squared = size.powi(2);

        for point in &mut self.points {
            let point_position = &point.position;
            let distance_squared = crate::position::distance_squared(point_position, position);

            point.hovered = distance_squared < size_squared;
        }
    }

    pub fn click(&self) {
        let point = self.points.iter().find(|x| x.hovered);

        if let Some(point) = point {
            println!("Clicked point at {:?}", point.position);
        }
    }

    pub fn draw(&self, target: &mut Frame, screen_size: &Position) {
        let color: [f32; 3] = [1.0, 0.0, 0.0];
        let hover_color: [f32; 3] = [0.0, 1.0, 0.0];

        for point in &self.points {
            let uniforms = dynamic_uniform! {
                point_size: &Self::SIZE,
                window_size: screen_size,
                point_color: if point.hovered { &hover_color } else { &color } ,
                offset: &point.position,
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
