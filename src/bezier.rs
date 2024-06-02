use bevy::{
    app::{Plugin, PostUpdate, Startup},
    ecs::{
        change_detection::DetectChanges,
        component::Component,
        entity::Entity,
        system::{Commands, EntityCommands, Local, Query, Res},
        world::Ref,
    },
};

use crate::{
    position::Position,
    rendering::{
        point::{Point, Points},
        primitives,
    },
    selection::{Connection, Draggable, Hoverable},
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

fn generate_bezier_points(control_points: &[Position; 4]) -> impl Iterator<Item = Position> {
    generate_bezier_points_with_offset(control_points, None, None)
}

fn generate_bezier_points_with_offset(
    control_points: &[Position; 4],
    subdivisions: Option<usize>,
    offset: Option<f64>,
) -> impl Iterator<Item = Position> {
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

    shape_points.into_iter()
}

#[derive(Component, Clone)]
struct BezierCurve {
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

fn initialize_bezier_curve(mut commands: Commands) {
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

    let bezier_curve = BezierCurve {
        start_point,
        start_handle,
        end_handle,
        end_point,
        handles,
        curve,
    };

    commands.spawn(bezier_curve);
}

#[derive(Default)]
struct ControlPoints([Position; 4]);

fn update_bezier_curve(
    bezier_curve: &BezierCurve,
    points: &mut Points,
    positions_query: &Query<Ref<Position>>,
    primitives_query: &mut Query<&mut primitives::Primatives>,
    system: &Res<crate::my_time::Time>,
    control_points: &mut Local<ControlPoints>,
) {
    // Look at each point if any of them have a position that has changed
    let control_points_query = positions_query.iter_many([
        bezier_curve.start_point,
        bezier_curve.start_handle,
        bezier_curve.end_handle,
        bezier_curve.end_point,
    ]);

    let mut has_change = false;

    for (i, point) in control_points_query.enumerate() {
        if point.is_changed() {
            has_change = true;
        }

        // Clone here because many below processes can only take
        // pure position objects
        // TODO: Figure out a way to use either refs _or_ objects
        // below. May be able to make improvements in a lot of places
        // since right now we are doing a lot of unnecesary cloning/copying
        control_points.0[i] = point.as_ref().clone();
    }
    // We always want to update the curve follower
    let elapsed = system.elapsed / 4.0;
    let point_iterator =
        generate_bezier_points_with_offset(&control_points.0, Some(10), Some(elapsed));
    for point in point_iterator {
        points.draw_point(point, 10.0);
    }

    // If any of the curve points have been changed we need to update the curve parts
    if !has_change {
        return;
    }

    let [mut handles, mut curve] =
        primitives_query.many_mut([bezier_curve.handles, bezier_curve.curve]);

    handles.set_positions(control_points.0.clone());

    let curve_points = generate_bezier_points(&control_points.0);
    curve.set_positions(curve_points);
}

fn update_bezier_curve_system(
    bezier_curve_query: Query<&BezierCurve>,
    mut points: Points,
    positions_query: Query<Ref<Position>>,
    mut primitives_query: Query<&mut primitives::Primatives>,
    system: Res<crate::my_time::Time>,
    mut control_points: Local<ControlPoints>,
) {
    for bezier_curve in bezier_curve_query.iter() {
        update_bezier_curve(
            bezier_curve,
            &mut points,
            &positions_query,
            &mut primitives_query,
            &system,
            &mut control_points,
        );
    }
}

pub struct BezierPlugin;

impl Plugin for BezierPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Startup, initialize_bezier_curve);
        app.add_systems(PostUpdate, update_bezier_curve_system);
    }
}
