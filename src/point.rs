use crate::position::Position;
use bevy_ecs::component::Component;
use glium::{
    dynamic_uniform, glutin::surface::WindowSurface, implement_vertex, index::PrimitiveType,
    Display, DrawParameters, Frame, IndexBuffer, Program, Surface, VertexBuffer,
};

#[derive(Clone, Copy)]
struct Vertex {
    position: [f32; 2],
    uv: [f32; 2],
}
implement_vertex!(Vertex, position, uv);

struct Point {
    position: Position,
    hovered: bool,
    attached_point: Option<usize>,
}

pub struct Renderer<'a> {
    /// Quad buffer for rendering points
    buffer: VertexBuffer<Vertex>,
    /// Indicies of the quad buffer
    indicies: IndexBuffer<u16>,
    /// Program that renders a circle on a quad mesh
    program: Program,
    /// This is here so we properly set alpha blending
    params: DrawParameters<'a>,
}

impl<'a> Renderer<'a> {
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
        let indicies = glium::IndexBuffer::new(
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

            float circle(in vec2 _st, in float _radius, in float edge){
                vec2 dist = _st-vec2(0.5);
                return 1.-smoothstep(_radius-(_radius*edge),
                                    _radius+(_radius*edge),
                                    dot(dist,dist)*4.0);
            }

            void main() {
                float a = circle(texcoord, .9, 0.02);
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
            params,
            program,
            buffer,
            indicies,
        }
    }

    pub fn draw(&self, target: &mut Frame, data: &Data, screen_size: &Position) {
        let color: [f32; 3] = [1.0, 0.0, 0.0];
        let hover_color: [f32; 3] = [0.0, 1.0, 0.0];

        for point in &data.points {
            let uniforms = dynamic_uniform! {
                point_size: &data.point_size,
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

#[derive(Component)]
pub struct Data {
    /// Point objects for each point to render
    points: Vec<Point>,
    /// Size for each point
    point_size: f32,

    /// Index into the points vector that contains the held point
    held_point: Option<usize>,

    /// This is just a buffer that is used when calling get_points
    /// Should not be used for anything else
    positions: Vec<Position>,
}

impl Data {
    const DEFAULT_SIZE: f32 = 15.0;

    pub fn new(points: &[Position], size: Option<f32>) -> Self {
        let mut result = Self {
            point_size: size.unwrap_or(Self::DEFAULT_SIZE),
            points: vec![],
            positions: vec![],
            held_point: None,
        };
        result.set_points(points);

        result
    }

    pub fn set_points(&mut self, points: &[Position]) {
        // Points are reset so the held point no longer makes any sense
        self.held_point = None;
        self.points = points
            .iter()
            .enumerate()
            .map(|(i, x)| Point {
                position: *x,
                hovered: false,
                attached_point: {
                    match i {
                        0 => Some(1),
                        3 => Some(2),
                        _ => None,
                    }
                },
            })
            .collect();
    }

    pub fn get_points(&mut self) -> &[Position] {
        self.positions = self.points.iter().map(|x| x.position).collect();
        &self.positions
    }

    pub fn mouse_moved(
        &mut self,
        position: &Position,
        previous_position: &Option<Position>,
    ) -> bool {
        let size = Self::DEFAULT_SIZE + 5.0;
        let size_squared = size.powi(2);

        for point in &mut self.points {
            let point_position = &point.position;
            let distance_squared = point_position.distance_squared(position);

            point.hovered = distance_squared < size_squared;
        }

        if let Some(point) = self.held_point {
            let point = &mut self.points[point];
            if let Some(previous_position) = previous_position {
                let difference = *position - *previous_position;
                point.position = point.position + difference;

                if let Some(attached) = point.attached_point {
                    let attached = &mut self.points[attached];
                    attached.position = attached.position + difference;
                }

                return true;
            }
        }

        false
    }

    pub fn click(&mut self) {
        self.held_point = self.points.iter().position(|x| x.hovered);
    }

    pub fn release(&mut self) {
        self.held_point = None;
    }

}
