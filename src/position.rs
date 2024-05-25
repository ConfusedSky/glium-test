use core::ops::{Add, Sub};
use std::ops::{Deref, DerefMut};

#[derive(Clone, Copy)]
pub struct Position([f32; 2]);

impl Position {
    pub fn distance_squared(&self, other: &Position) -> f32 {
        let [x, y] = (*other - *self).0;

        x.powi(2) + y.powi(2)
    }
    
    pub fn lerp(this: Self, other: Self, t: f64) -> Self {
        let x = this.0[0] as f64 * (1. - t) + other.0[0] as f64 * t;
        let y = this.0[1] as f64 * (1. - t) + other.0[1] as f64 * t;
        [x as f32, y as f32].into()
    }
}

impl Add<Position> for Position {
    type Output = Position;

    fn add(self, rhs: Position) -> Self::Output {
        let x = self.0[0] + rhs.0[0];
        let y = self.0[1] + rhs.0[1];

        Self([x, y])
    }
}

impl Sub<Position> for Position {
    type Output = Position;

    fn sub(self, rhs: Position) -> Self::Output {
        let x = self.0[0] - rhs.0[0];
        let y = self.0[1] - rhs.0[1];

        Self([x, y])
    }
}

impl Deref for Position {
    type Target = [f32; 2];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Position {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<[f32; 2]> for Position {
    fn from(value: [f32; 2]) -> Self {
        Self(value)
    }
}
