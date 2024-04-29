pub type Position = [f32; 2];

pub fn distance(a: Position, b: Position) -> f32 {
    let x = b[0] - a[0];
    let y = b[1] - a[1];

    (x * x + y * y).sqrt()
}