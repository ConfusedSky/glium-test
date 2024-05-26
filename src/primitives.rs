use std::{collections::HashMap, sync::atomic::AtomicUsize};

use crate::{RenderParams, Vertex};
use bevy_ecs::component::Component;
use glium::{
    dynamic_uniform, glutin::surface::WindowSurface, Display, DrawParameters, Program, Surface,
    VertexBuffer,
};

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

pub struct Renderer {
    program: Program,
    buffers: HashMap<usize, VertexBuffer<Vertex>>,
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
            buffers: Default::default(),
            program,
        }
    }

    pub fn draw(&mut self, render_params: &mut RenderParams, data: &mut Data) {
        let uniforms = dynamic_uniform! {
            window_size: render_params.screen_size,
        };

        let buffer = {
            if data.dirty || !self.buffers.contains_key(&data.id) {
                let buffer =
                    glium::VertexBuffer::new(render_params.display, &data.primitive_data).unwrap();
                self.buffers.insert(data.id, buffer);
                data.dirty = false;
            }

            &self.buffers[&data.id]
        };

        render_params
            .target
            .draw(
                buffer,
                &glium::index::NoIndices(data.primitive_type.into()),
                &self.program,
                &uniforms,
                &data.params,
            )
            .unwrap();
    }
}

#[derive(Component)]
pub struct Data<'a> {
    id: usize,
    primitive_data: Vec<Vertex>,
    params: DrawParameters<'a>,
    primitive_type: Type,
    dirty: bool,
}

impl Data<'_> {
    pub fn new(points: &[Vertex], primitive_type: Type, size: f32) -> Self {
        static ID_COUNTER: AtomicUsize = AtomicUsize::new(0);
        let id = ID_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        let primitive_data = points.to_vec();

        let params = DrawParameters {
            point_size: Some(size),
            line_width: Some(size),
            ..Default::default()
        };

        Self {
            id,
            primitive_data,
            params,
            primitive_type,
            dirty: true,
        }
    }

    pub fn set_points(&mut self, points: &[Vertex]) {
        self.primitive_data = points.to_vec();
        self.dirty = true;
    }
}
