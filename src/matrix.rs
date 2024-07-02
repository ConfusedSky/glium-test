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

    #[allow(dead_code)]
    pub fn translate(x_offset: f32, y_offset: f32) -> Mat3 {
        Mat3([[1.0, 0.0, x_offset], [0.0, 1.0, y_offset], [0.0, 0.0, 1.0]])
    }

    #[allow(dead_code)]
    pub fn multiply(&self, other: &Mat3) -> Mat3 {
        let mut result = [[0.0; 3]; 3];
        for i in 0..3 {
            for j in 0..3 {
                for k in 0..3 {
                    result[i][j] += self.0[i][k] * other.0[k][j];
                }
            }
        }
        Mat3(result)
    }
}

impl AsUniformValue for Mat3 {
    fn as_uniform_value(&self) -> glium::uniforms::UniformValue<'_> {
        glium::uniforms::UniformValue::Mat3(self.0)
    }
}
