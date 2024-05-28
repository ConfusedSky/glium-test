use bevy_ecs::{
    change_detection::DetectChanges,
    entity::Entity,
    system::{Commands, EntityCommands, Query, Res, Resource},
    world::Ref,
};

use crate::{
    position::Position,
    rendering::{
        point::{self, Point},
        primitives,
    },
    selection::{Connection, Draggable, Hoverable},
    System,
};

/// Calculates a point t along a bezier curve
/// t must be 0 <= t < 1
fn bezier(
    start_point: Position,
    start_handle: Position,
    end_handle: Position,
    end_point: Position,
    t: f64,
) -> Position {
    assert!(t >= 0.0);
    assert!(t < 1.0);

    let t_inv = 1.0 - t;
    let t1 = t_inv.powi(3);
    let t2 = 3.0 * t_inv.powi(2) * t;
    let t3 = 3.0 * t_inv * t.powi(2);
    let t4 = t.powi(3);

    t1 * start_point + t2 * start_handle + t3 * end_handle + t4 * end_point
}

pub fn generate_bezier_points(control_points: &[Position]) -> Vec<Position> {
    generate_bezier_points_with_offset(control_points, None, None)
}

pub fn generate_bezier_points_with_offset(
    control_points: &[Position],
    subdivisions: Option<usize>,
    offset: Option<f64>,
) -> Vec<Position> {
    let subdivisions = subdivisions.unwrap_or(60);
    let mut shape_points = Vec::with_capacity(subdivisions);
    let offset = offset.unwrap_or_default();

    for i in 0..subdivisions {
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

#[derive(Resource, Clone)]
pub struct BezierCurve {
    pub start_point: Entity,
    pub start_handle: Entity,
    pub end_handle: Entity,
    pub end_point: Entity,

    pub handles: Entity,
    pub curve: Entity,
}

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

    let handles = primitives::Primatives::new(&[], primitives::Type::Line, 2.0);
    let handles = commands.spawn(handles).id();

    let curve = primitives::Primatives::new(&[], primitives::Type::LineStrip, 2.0);
    let curve = commands.spawn(curve).id();

    let resource = BezierCurve {
        start_point,
        start_handle,
        end_handle,
        end_point,
        handles,
        curve,
    };
    commands.insert_resource(resource);
}

pub fn update_bezier_curve(
    mut commands: Commands,
    positions_query: Query<Ref<Position>>,
    mut primitives_query: Query<&mut primitives::Primatives>,
    bezier_curve: Res<BezierCurve>,
    system: Res<System>,
) {
    // Look at each point if any of them have a position that has changed
    let curve_points: Vec<_> = positions_query
        .iter_many([
            bezier_curve.start_point,
            bezier_curve.start_handle,
            bezier_curve.end_handle,
            bezier_curve.end_point,
        ])
        .collect();

    let mut has_change = false;

    let mut control_points: Vec<_> = Vec::with_capacity(4);

    for point in curve_points {
        if point.is_changed() {
            has_change = true;
        }

        // Clone here because many below processes can only take
        // pure position objects
        // TODO: Figure out a way to use either refs _or_ objects
        // below. May be able to make improvements in a lot of places
        // since right now we are doing a lot of unnecesary cloning/copying
        control_points.push(point.as_ref().clone());
    }
    // We always want to update the curve follower
    let elapsed = system.elapsed / 4.0;
    let p = generate_bezier_points_with_offset(&control_points, Some(10), Some(elapsed));
    for point in p {
        let draw_command = point::DrawPoint {
            position: point,
            size: 10.0,
        };
        commands.add(draw_command);
    }

    // If any of the curve points have been changed we need to update the curve parts
    if !has_change {
        return;
    }

    let [mut handles, mut curve] =
        primitives_query.many_mut([bezier_curve.handles, bezier_curve.curve]);

    handles.set_positions(&control_points);

    let curve_points: Vec<_> = generate_bezier_points(&control_points);
    curve.set_positions(&curve_points);
}
