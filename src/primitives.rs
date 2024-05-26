use std::sync::atomic::AtomicUsize;

use crate::{position::Position, renderer::RenderParams};
use bevy_ecs::component::Component;
use glium::{
    dynamic_uniform, glutin::surface::WindowSurface, implement_vertex, Display, DrawParameters, Program, Surface, VertexBuffer
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
pub struct Vertex {
    position: [f32; 2],
}
implement_vertex!(Vertex, position);

impl From<Position> for Vertex {
    fn from(value: Position) -> Self {
        Self {
            position: value.into()
        }
    }
}

pub struct Renderer {
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

    pub fn draw(&mut self, render_params: &mut RenderParams, data: &mut Primatives) {
        let uniforms = dynamic_uniform! {
            window_size: render_params.screen_size,
        };

        let buffer = {
            if data.buffer_needs_refresh
                || self.buffers.len() <= data.id
                || self.buffers[data.id].is_none()
            {
                let buffer =
                    glium::VertexBuffer::new(render_params.display, &data.primitive_data).unwrap();
                // If the vector is not large enough to contain the new buffer then we resize based on
                // the largeset id we have available
                // Note: with this solution if we create and delete primitive data objects enough
                // we will be running into a sort of memory leak where the id keeps going up but old
                // id's are never reused so going forward this may not be the best solution
                if self.buffers.len() <= data.id {
                    let largest = DATA_ID_COUNTER.load(std::sync::atomic::Ordering::Relaxed);
                    self.buffers.resize_with(largest, || None);
                }

                self.buffers[data.id] = Some(buffer);
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
    pub fn new(points: &[Vertex], primitive_type: Type, size: f32) -> Self {
        let id = DATA_ID_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        let primitive_data = points.to_vec();

        Self {
            id,
            size,
            primitive_data,
            primitive_type,
            buffer_needs_refresh: true,
        }
    }

    pub fn set_points(&mut self, points: &[Vertex]) {
        self.primitive_data = points.to_vec();
        self.buffer_needs_refresh = true;
    }
}
