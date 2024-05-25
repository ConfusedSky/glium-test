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
    generate_bezier_points_with_offset(control_points, None, None)
        .into_iter()
        .map(|x| Vertex { position: x.into() })
        .collect()
}

pub fn generate_bezier_points_with_offset(
    control_points: &[Position],
    subdivisions: Option<usize>,
    offset: Option<f64>,
) -> Vec<Position> {
    let subdivisions = subdivisions.unwrap_or(60);
    let mut shape_points = Vec::with_capacity(subdivisions);
    let offset = offset.unwrap_or_default();

    for i in 0..subdivisions + 1 {
        let t = if offset > 0.0 {
            let t = i as f64 / subdivisions as f64 + offset;
            t.fract()
        } else {
            i as f64 / subdivisions as f64
        };

        let point = bezier(
            control_points[0],
            control_points[1],
            control_points[2],
            control_points[3],
            t,
        );
        shape_points.push(point);
    }

    shape_points
}
