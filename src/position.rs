pub type Position = [f32; 2];

pub fn distance_squared(a: &Position, b: &Position) -> f32 {
    let [x, y] = difference(b, a);

    x.powi(2) + y.powi(2)
}

pub fn sum(a: &Position, b: &Position) -> Position {
    let x = a[0] + b[0];
    let y = a[1] + b[1];

    [x, y]
}

pub fn difference(a: &Position, b: &Position) -> Position {
    let x = a[0] - b[0];
    let y = a[1] - b[1];

    [x, y]
}