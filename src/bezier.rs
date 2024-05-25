use crate::{position::Position, Vertex};

fn bezier(p1: Position, p2: Position, p3: Position, p4: Position, t: f64) -> Position {
    let a = Position::lerp(p1, p2, t);
    let b = Position::lerp(p2, p3, t);
    let c = Position::lerp(p3, p4, t);
    let d = Position::lerp(a, b, t);
    let e = Position::lerp(b, c, t);
    Position::lerp(d, e, t)
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
        shape_points.push(Vertex { position: *point });
    }

    shape_points
}
