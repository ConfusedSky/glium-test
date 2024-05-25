use core::ops::{Add, Sub};

#[derive(Clone, Copy)]
pub struct Position(pub [f32; 2]);

impl Position {
    pub fn distance_squared(&self, other: &Position) -> f32 {
        let [x, y] = (*other - *self).0;

        x.powi(2) + y.powi(2)
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
