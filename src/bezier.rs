use crate::{position::Position, Vertex};

fn lerp(p1: Position, p2: Position, t: f64) -> Position {
    let x = p1.0[0] as f64 * (1. - t) + p2.0[0] as f64 * t;
    let y = p1.0[1] as f64 * (1. - t) + p2.0[1] as f64 * t;
    Position([x as f32, y as f32])
}

fn bezier(p1: Position, p2: Position, p3: Position, p4: Position, t: f64) -> Position {
    let a = lerp(p1, p2, t);
    let b = lerp(p2, p3, t);
    let c = lerp(p3, p4, t);
    let d = lerp(a, b, t);
    let e = lerp(b, c, t);
    lerp(d, e, t)
}

pub fn generate_bezier_points(control_points: &[Position]) -> Vec<Vertex> {
    let subdivisions = 60;
    let mut shape_points = Vec::with_capacity(subdivisions);

    for i in 0..subdivisions + 1 {
        let t = i as f64 / subdivisions as f64;
        let point = bezier(
            control_points[0],
            control_points[1],
            control_points[2],
            control_points[3],
            t,
        );
        shape_points.push(Vertex { position: point.0 });
    }

    shape_points
}