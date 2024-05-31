use core::ops::{Add, Sub};
use std::ops::Mul;

use bevy::ecs::component::Component;
use glium::uniforms::AsUniformValue;

#[derive(Clone, Copy, Component, Default, Debug)]
pub struct Position([f32; 2]);

impl Position {
    pub fn new(x: f32, y: f32) -> Self {
        Self([x, y])
    }

    pub fn x(&self) -> f32 {
        self.0[0]
    }

    pub fn y(&self) -> f32 {
        self.0[1]
    }

    pub fn distance_squared(&self, other: &Position) -> f32 {
        let [x, y] = (*other - *self).0;

        x.powi(2) + y.powi(2)
    }

    pub fn lerp(this: Self, other: Self, t: f64) -> Self {
        let x = this.x() as f64 * (1. - t) + other.x() as f64 * t;
        let y = this.y() as f64 * (1. - t) + other.y() as f64 * t;
        Self([x as f32, y as f32])
    }
}

impl Add<Position> for Position {
    type Output = Position;

    fn add(self, rhs: Position) -> Self::Output {
        let x = self.x() + rhs.x();
        let y = self.y() + rhs.y();

        Position([x, y])
    }
}

impl Sub<Position> for Position {
    type Output = Position;

    fn sub(self, rhs: Position) -> Self::Output {
        let x = self.x() - rhs.x();
        let y = self.y() - rhs.y();

        Position([x, y])
    }
}

impl Mul<f64> for Position {
    type Output = Position;

    fn mul(self, rhs: f64) -> Self::Output {
        let x = self.x() as f64 * rhs;
        let y = self.y() as f64 * rhs;
        Position([x as f32, y as f32])
    }
}

impl Mul<Position> for f64 {
    type Output = Position;

    fn mul(self, rhs: Position) -> Self::Output {
        rhs * self
    }
}

impl Mul<f32> for Position {
    type Output = Position;

    fn mul(self, rhs: f32) -> Self::Output {
        let x = self.x() * rhs;
        let y = self.y() * rhs;
        Position([x, y])
    }
}

impl Mul<Position> for f32 {
    type Output = Position;

    fn mul(self, rhs: Position) -> Self::Output {
        rhs * self
    }
}

impl From<[f32; 2]> for Position {
    fn from(value: [f32; 2]) -> Self {
        Self(value)
    }
}

impl From<Position> for [f32; 2] {
    fn from(value: Position) -> Self {
        value.0
    }
}

impl AsUniformValue for Position {
    fn as_uniform_value(&self) -> glium::uniforms::UniformValue<'_> {
        glium::uniforms::UniformValue::Vec2(self.0)
    }
}
