use bevy::ecs::component::Component;
use glium::uniforms::AsUniformValue;

#[derive(Component)]
pub enum Stroke {
    Outline,
}

#[derive(Clone, Copy, Debug, Component)]
pub struct Color([f32; 4]);

#[allow(dead_code)]
impl Color {
    pub const RED: Color = Color([1.0, 0.0, 0.0, 1.0]);
    pub const GREEN: Color = Color([0.0, 1.0, 0.0, 1.0]);
    pub const BLUE: Color = Color([0.0, 0.0, 1.0, 1.0]);
    pub const BLACK: Color = Color([0.0, 0.0, 0.0, 1.0]);
    pub const WHITE: Color = Color([1.0, 1.0, 1.0, 1.0]);

    pub fn new(r: f32, g: f32, b: f32) -> Self {
        Self([r, g, b, 1.0])
    }

    pub fn new_with_alpha(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self([r, g, b, a])
    }
}

impl Default for Color {
    fn default() -> Self {
        Self([0.0, 0.0, 0.0, 1.0])
    }
}

impl AsUniformValue for Color {
    fn as_uniform_value(&self) -> glium::uniforms::UniformValue<'_> {
        glium::uniforms::UniformValue::Vec4(self.0)
    }
}
