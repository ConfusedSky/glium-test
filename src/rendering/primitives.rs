use std::sync::atomic::AtomicUsize;

use super::renderer::RenderParams;
use crate::position::Position;
use bevy::ecs::{
    component::Component,
    system::{ResMut, Resource, SystemParam},
};
use glium::{
    dynamic_uniform, glutin::surface::WindowSurface, implement_vertex, Display, DrawParameters,
    Program, Surface, VertexBuffer,
};

static DATA_ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone, Copy)]
pub enum Type {
    Point,
    Line,
    LineStrip,
}

impl From<Type> for glium::index::PrimitiveType {
    fn from(value: Type) -> Self {
        match value {
            Type::Point => Self::Points,
            Type::Line => Self::LinesList,
            Type::LineStrip => Self::LineStrip,
        }
    }
}

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
}
implement_vertex!(Vertex, position);

impl From<Position> for Vertex {
    fn from(value: Position) -> Self {
        Self {
            position: value.into(),
        }
    }
}

const MAX_LINES: usize = 100;

#[derive(Resource, Default)]
pub(super) struct LinesData {
    lines_data: Vec<Position>,
}

#[derive(SystemParam)]
pub struct Lines<'w> {
    data: ResMut<'w, LinesData>,
}

impl Lines<'_> {
    pub fn draw_line(&mut self, from: Position, to: Position) {
        self.data.lines_data.push(from);
        self.data.lines_data.push(to);
    }
}

pub(super) struct Renderer {
    program: Program,
    // Maybe it could be a good idea to just have the array be a fixed size if this becomes a performance
    // Issue and set it with a constant if we would like to reduce indirection
    buffers: Vec<Option<VertexBuffer<Vertex>>>,
}

impl Renderer {
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

        Self {
            // Have a sufficiently large base capacity so we don't have to resize the array as much
            buffers: Vec::with_capacity(64),
            program,
        }
    }

    pub fn draw_immediate(&mut self, render_params: &mut RenderParams, data: &mut LinesData) {
        assert!(data.lines_data.len() < MAX_LINES * 2);

        let uniforms = dynamic_uniform! {
            window_size: render_params.screen_size,
        };

        let data: Vec<_> = data.lines_data.drain(..).map(Vertex::from).collect();
        let buffer = glium::VertexBuffer::new(render_params.display, &data).unwrap();

        let params = DrawParameters {
            line_width: Some(2.0),
            ..Default::default()
        };

        render_params
            .target
            .draw(
                &buffer,
                &glium::index::NoIndices(glium::index::PrimitiveType::LinesList),
                &self.program,
                &uniforms,
                &params,
            )
            .unwrap();
    }

    pub fn draw(&mut self, render_params: &mut RenderParams, data: &mut Primatives) {
        let uniforms = dynamic_uniform! {
            window_size: render_params.screen_size,
        };

        let buffer = {
            if data.buffer_needs_refresh
                || self.buffers.len() <= data.id
                || self.buffers[data.id].is_none()
            {
                // If the vector is not large enough to contain the new buffer then we resize based on
                // the largeset id we have available
                // Note: with this solution if we create and delete primitive data objects enough
                // we will be running into a sort of memory leak where the id keeps going up but old
                // id's are never reused so going forward this may not be the best solution
                if self.buffers.len() <= data.id {
                    let largest = DATA_ID_COUNTER.load(std::sync::atomic::Ordering::Relaxed);
                    self.buffers.resize_with(largest, || None);
                }

                if let Some(ref mut buffer) = self.buffers[data.id] {
                    buffer.write(&data.primitive_data);
                } else {
                    let buffer =
                        glium::VertexBuffer::dynamic(render_params.display, &data.primitive_data)
                            .unwrap();

                    self.buffers[data.id] = Some(buffer);
                }

                data.buffer_needs_refresh = false;
            }

            // Safety: Based on above code we can guarentee that we have a buffer that has been created
            self.buffers[data.id].as_ref().unwrap()
        };

        let params = DrawParameters {
            point_size: Some(data.size),
            line_width: Some(data.size),
            ..Default::default()
        };

        render_params
            .target
            .draw(
                buffer,
                &glium::index::NoIndices(data.primitive_type.into()),
                &self.program,
                &uniforms,
                &params,
            )
            .unwrap();
    }
}

#[derive(Component)]
pub struct Primatives {
    id: usize,
    size: f32,
    primitive_type: Type,

    primitive_data: Vec<Vertex>,
    buffer_needs_refresh: bool,
}

impl Primatives {
    pub fn new(positions: &[Position], primitive_type: Type, size: f32) -> Self {
        let id = DATA_ID_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        let primitive_data = positions.into_iter().map(|&x| Vertex::from(x)).collect();

        Self {
            id,
            size,
            primitive_data,
            primitive_type,
            buffer_needs_refresh: true,
        }
    }

    pub fn set_positions<'a, Iter: IntoIterator<Item = Position>>(&mut self, positions: Iter) {
        self.primitive_data.clear();
        self.primitive_data
            .extend(positions.into_iter().map(|x| Vertex::from(x)));
        self.buffer_needs_refresh = true;
    }
}
