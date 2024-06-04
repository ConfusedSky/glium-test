use super::{renderer::RenderParams, Color, Stroke};
use crate::{position::Position, selection};
use bevy::ecs::{
    component::Component,
    system::{ResMut, Resource, SystemParam},
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
    pub color: Color,
    pub outline: bool,
}

// Move this back over to point.rs after the refactor is complete
#[derive(Component)]
pub struct Point {
    pub size: f32,
}

#[derive(Resource, Default)]
pub(super) struct PointsData {
    buffer: Vec<RenderData>,
}

/// Note this probably limits parallelizability by having a
/// Mutable reference here especially if multiple systems
/// need to draw points
#[derive(SystemParam)]
pub struct Points<'w> {
    data: ResMut<'w, PointsData>,
}

impl Points<'_> {
    pub fn draw_point(&mut self, position: Position, size: f32, color: Color) {
        self.data.buffer.push(RenderData {
            position,
            size,
            hovered: false,
            color,
            outline: false,
        });
    }
}

pub(super) struct Renderer<'draw> {
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

            uniform vec4 point_color;
            uniform float inner_fill_cutoff;

            varying lowp vec2 texcoord;

            float circle(in vec2 _dist, in float _radius, in float _edge ){
                return 1.-smoothstep(_radius-(_radius*_edge),
                                    _radius+(_radius*_edge),
                                    dot(_dist,_dist)*4.0);
            }

            void main() {
                vec2 dist = texcoord-vec2(0.5);
                float a = circle(dist, .9, 0.02);
                a -= circle(dist, inner_fill_cutoff, 0.02);
                color = vec4(point_color.rgb, point_color.a * a);
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
        let hover_color: [f32; 4] = [0.0, 1.0, 0.0, 1.0];

        for point in data.into_iter() {
            let uniforms = dynamic_uniform! {
                point_size: &point.size,
                window_size: render_params.screen_size,
                point_color: if point.hovered { &hover_color } else { &point.color } ,
                offset: &point.position,
                inner_fill_cutoff: if point.outline { &(0.7 as f32) } else { &(0.0 as f32) },
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
    pub fn draw_single(&self, render_params: &mut RenderParams, data: RenderData) {
        self.draw_points(render_params, [data]);
    }

    pub fn draw_from_world(&self, render_params: &mut RenderParams, world: &mut World) {
        // Draw all points from ecs
        let mut query = world.query::<(
            &Position,
            &Point,
            Option<&selection::Hovered>,
            Option<&Stroke>,
        )>();
        let iter = query
            .iter(world)
            .map(|(position, Point { size }, hovered, stroke)| RenderData {
                position: *position,
                size: *size,
                hovered: hovered.is_some(),
                color: Color::RED,
                outline: matches!(
                    stroke.map(|stroke| matches!(stroke, Stroke::Outline)),
                    Some(true)
                ),
            });
        self.draw_points(render_params, iter);

        // Draw single frame points
        let mut frame_points: Mut<PointsData> = world.resource_mut();

        // We use drain here so we can remove all elements from the buffer _and_
        // we don't have to do any additional cloning
        let iter = frame_points.buffer.drain(..);
        self.draw_points(render_params, iter);
    }
}
