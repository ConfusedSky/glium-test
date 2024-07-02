use glium::uniforms::AsUniformValue;

use crate::position::Position;

#[derive(Debug, Clone, Copy)]
pub struct Mat3([[f32; 3]; 3]);

impl Mat3 {
    pub fn world_to_view(window_size: Position) -> Mat3 {
        Mat3([
            [2.0 / window_size.x(), 0.0, -1.0],
            [0.0, -2.0 / window_size.y(), 1.0],
            [0.0, 0.0, 1.0],
        ])
    }
}

impl AsUniformValue for Mat3 {
    fn as_uniform_value(&self) -> glium::uniforms::UniformValue<'_> {
        glium::uniforms::UniformValue::Mat3(self.0)
    }
}
