use core::ops::{Add, Sub};

#[derive(Clone, Copy)]
pub struct Position([f32; 2]);

impl Position {
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
        [x as f32, y as f32].into()
    }
}

impl Add<Position> for Position {
    type Output = Position;

    fn add(self, rhs: Position) -> Self::Output {
        let x = self.x() + rhs.x();
        let y = self.y() + rhs.y();

        Self([x, y])
    }
}

impl Sub<Position> for Position {
    type Output = Position;

    fn sub(self, rhs: Position) -> Self::Output {
        let x = self.x() - rhs.x();
        let y = self.y() - rhs.y();

        Self([x, y])
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
