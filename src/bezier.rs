use bevy_ecs::{
    entity::Entity,
    system::{Commands, EntityCommands, Resource},
};

use crate::{
    point::Point, position::Position, selection::{Connection, Draggable, Hoverable}, Vertex
};

fn bezier(
    start_point: Position,
    start_handle: Position,
    end_handle: Position,
    end_point: Position,
    t: f64,
) -> Position {
    let a = Position::lerp(start_point, start_handle, t);
    let b = Position::lerp(start_handle, end_handle, t);
    let c = Position::lerp(end_handle, end_point, t);
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

#[derive(Resource)]
pub struct BezierCurve {
    pub start_point: Entity,
    pub start_handle: Entity,
    pub end_handle: Entity,
    pub end_point: Entity,

    pub handles: Entity,
    pub curve: Entity,
}

#[derive(Resource)]
pub struct ControlPointArray(pub [Position; 4]);

fn create_control_point<'c>(commands: &'c mut Commands, x: f32, y: f32) -> EntityCommands<'c> {
    commands.spawn((
        Position::new(x, y),
        Point { size: 15.0 },
        Hoverable { radius: 20.0 },
        Draggable,
    ))
}

pub fn initialize_bezier_curve(mut commands: Commands) {
    let start_handle = create_control_point(&mut commands, 400.0, 456.0).id();
    let end_handle = create_control_point(&mut commands, 400.0, 24.0).id();

    let start_point = create_control_point(&mut commands, 200.0, 240.0)
        .insert(Connection(start_handle))
        .id();
    let end_point = create_control_point(&mut commands, 600.0, 240.0)
        .insert(Connection(end_handle))
        .id();

    let handles = commands.spawn_empty().id();
    let curve = commands.spawn_empty().id();

    commands.insert_resource(BezierCurve {
        start_point,
        start_handle,
        end_handle,
        end_point,
        handles,
        curve,
    });
}
