use crate::{position::Position, renderer::RenderParams, selection};
use bevy_ecs::{
    component::Component,
    system::{Command, Resource},
    world::{Mut, World},
};
use glium::{
    dynamic_uniform, glutin::surface::WindowSurface, implement_vertex, index::PrimitiveType,
    Display, DrawParameters, IndexBuffer, Program, Surface, VertexBuffer,
};

#[derive(Clone, Copy)]
struct Vertex {
    position: [f32; 2],
    uv: [f32; 2],
}
implement_vertex!(Vertex, position, uv);

#[derive(Default, Clone)]
pub struct RenderData {
    pub position: Position,
    pub size: f32,
    pub hovered: bool,
}

// Move this back over to point.rs after the refactor is complete
#[derive(Component)]
pub struct Point {
    pub size: f32,
}

#[derive(Resource, Default)]
pub struct FrameData {
    buffer: Vec<RenderData>,
}

pub struct DrawPoint {
    pub position: Position,
    pub size: f32,
}

impl Command for DrawPoint {
    fn apply(self, world: &mut World) {
        let mut frame_data = world.resource_mut::<FrameData>();
        frame_data.buffer.push(RenderData {
            position: self.position,
            size: self.size,
            hovered: false,
        })
    }
}

pub struct Renderer<'draw> {
    /// Quad buffer for rendering points
    buffer: VertexBuffer<Vertex>,
    /// Indicies of the quad buffer
    indicies: IndexBuffer<u16>,
    /// Program that renders a circle on a quad mesh
    program: Program,
    /// This is here so we properly set alpha blending
    params: DrawParameters<'draw>,
}

impl<'draw> Renderer<'draw> {
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

    // Maybe make this public eventually since it could end up being more efficient to use this
    // than render single point
    pub fn draw_points<'a, It>(&self, render_params: &mut RenderParams, data: It)
    where
        It: IntoIterator<Item = RenderData>,
    {
        let color: [f32; 3] = [1.0, 0.0, 0.0];
        let hover_color: [f32; 3] = [0.0, 1.0, 0.0];

        for point in data.into_iter() {
            let uniforms = dynamic_uniform! {
                point_size: &point.size,
                window_size: render_params.screen_size,
                point_color: if point.hovered { &hover_color } else { &color } ,
                offset: &point.position,
            };
            render_params
                .target
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

    #[allow(dead_code)]
    pub fn draw(&self, render_params: &mut RenderParams, data: &Collection) {
        self.draw_points(render_params, data.points.iter().cloned());
    }

    #[allow(dead_code)]
    pub fn draw_single(&self, render_params: &mut RenderParams, data: RenderData) {
        self.draw_points(render_params, [data]);
    }

    pub fn draw_from_world(&self, render_params: &mut RenderParams, world: &mut World) {
        // Draw all points from ecs
        let mut query = world.query::<(&Position, &Point, Option<&selection::Hovered>)>();
        let iter = query
            .iter(world)
            .map(|(position, Point { size }, hovered)| RenderData {
                position: *position,
                size: *size,
                hovered: hovered.is_some(),
            });
        self.draw_points(render_params, iter);

        // Draw single frame points
        let mut frame_points: Mut<FrameData> = world.resource_mut();

        // We use drain here so we can remove all elements from the buffer _and_
        // we don't have to do any copying
        let iter = frame_points.buffer.drain(..);
        self.draw_points(render_params, iter);
    }
}

#[derive(Component)]
pub struct Collection {
    /// Point objects for each point to render
    points: Vec<RenderData>,
    /// Size for each point
    point_size: f32,
}

impl Collection {
    const DEFAULT_SIZE: f32 = 15.0;

    pub fn new(points: &[Position], size: Option<f32>) -> Self {
        let mut result = Self {
            point_size: size.unwrap_or(Self::DEFAULT_SIZE),
            points: vec![],
        };
        result.set_points(points);

        result
    }

    pub fn set_points(&mut self, points: &[Position]) {
        // Points are reset so the held point no longer makes any sense
        self.points = points
            .iter()
            .map(|x| RenderData {
                size: self.point_size,
                position: *x,
                hovered: false,
            })
            .collect();
    }
}
