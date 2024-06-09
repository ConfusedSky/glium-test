use bevy::{
    app::{Plugin, PostUpdate, Startup, Update},
    ecs::{
        bundle::Bundle,
        change_detection::DetectChanges,
        component::Component,
        entity::Entity,
        query::{Added, With},
        removal_detection::RemovedComponents,
        system::{Commands, EntityCommands, Local, Query, Res},
        world::Ref,
    },
};

use crate::{
    hidden::Hidden,
    position::Position,
    rendering::{
        point::{Point, Points},
        primitives::{self, Lines},
        Color, Stroke,
    },
    selection::{Connection, Draggable, Hoverable, Selectable, Selected},
};

/// Calculates a point t along a bezier curve
///
/// # Panics
/// panics if 0 <= t <= 1 is not true
fn bezier(
    start_point: Position,
    start_handle: Position,
    end_handle: Position,
    end_point: Position,
    t: f64,
) -> Position {
    assert!(t >= 0.0);
    assert!(t <= 1.0);

    let t_inv = 1.0 - t;
    let t1 = t_inv.powi(3);
    let t2 = 3.0 * t_inv.powi(2) * t;
    let t3 = 3.0 * t_inv * t.powi(2);
    let t4 = t.powi(3);

    t1 * start_point + t2 * start_handle + t3 * end_handle + t4 * end_point
}

fn generate_bezier_points(control_points: &[Position; 4]) -> impl Iterator<Item = Position> {
    generate_bezier_points_with_offset(control_points, None, None, true)
}

/// # Panics
/// Panics if subdivisions is 0
fn generate_bezier_points_with_offset(
    control_points: &[Position; 4],
    subdivisions: Option<usize>,
    offset: Option<f64>,
    include_end_point: bool,
) -> impl Iterator<Item = Position> {
    let offset = offset.unwrap_or_default();
    let subdivisions = subdivisions.unwrap_or(60);
    let mut shape_points = Vec::with_capacity(subdivisions + if include_end_point { 1 } else { 0 });

    assert_ne!(subdivisions, 0, "Subdivisions must be a positive integer");

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

    if include_end_point {
        shape_points.push(control_points[3]);
    }

    shape_points.into_iter()
}

// Implements De casteljaus algorithm to split a bezier
// curve into two bezier curves
fn split_bezier(
    start_point: Position,
    start_handle: Position,
    end_handle: Position,
    end_point: Position,
    t: f64,
) -> ([Position; 4], [Position; 4]) {
    let c1_start_point = start_point;
    let c1_start_handle = Position::lerp(start_point, start_handle, t);
    let c2_end_handle = Position::lerp(end_handle, end_point, t);
    let temp = Position::lerp(start_handle, end_handle, t);
    let c1_end_handle = Position::lerp(c1_start_handle, temp, t);
    let c2_start_handle = Position::lerp(temp, c2_end_handle, t);
    let c1_end_point = Position::lerp(c1_end_handle, c2_start_handle, t);
    let c2_start_point = c1_end_point;
    let c2_end_point = end_point;

    (
        [c1_start_point, c1_start_handle, c1_end_handle, c1_end_point],
        [c2_start_point, c2_start_handle, c2_end_handle, c2_end_point],
    )
}

// Components that exist for reverse lookup of a curve from a point
#[derive(Component)]
#[allow(dead_code)]
struct BezierHandle(Entity);

// Start and end points are different components so a mid point
// of a spline can have both
#[derive(Component)]
#[allow(dead_code)]
struct BezierStartPoint(Entity);
#[derive(Component)]
#[allow(dead_code)]
struct BezierEndPoint(Entity);

#[derive(Component, Clone)]
struct BezierCurve {
    pub start_point: Entity,
    pub start_handle: Entity,
    pub end_handle: Entity,
    pub end_point: Entity,

    pub curve_primitives: Entity,
}

#[derive(Bundle)]
struct BaseControlPointBundle {
    position: Position,
    point: Point,
    hoverable: Hoverable,
    draggable: Draggable,
    stroke: Stroke,
}

impl BaseControlPointBundle {
    fn new(position: Position) -> Self {
        Self {
            position,
            point: Point { size: 15.0 },
            hoverable: Hoverable { radius: 20.0 },
            draggable: Draggable,
            stroke: Stroke::Outline,
        }
    }
}

fn create_handle<'c>(
    commands: &'c mut Commands,
    position: Position,
    curve: Entity,
) -> EntityCommands<'c> {
    commands.spawn((
        BaseControlPointBundle::new(position),
        Hidden,
        BezierHandle(curve),
    ))
}

fn create_terminal_point<'c>(
    commands: &'c mut Commands,
    position: Position,
    connections: &[Entity],
    // Set this to the curve that this terminal is
    // the end point for
    end_point_curve: Option<Entity>,
    // Set this to the curve that this terminal is
    // the start point for
    start_point_curve: Option<Entity>,
) -> EntityCommands<'c> {
    let mut commands = commands.spawn((
        BaseControlPointBundle::new(position),
        Selectable,
        SolidWhenSelected,
        Connection(Vec::from(connections)),
    ));

    if let Some(entity) = end_point_curve {
        commands.insert(BezierEndPoint(entity));
    };

    if let Some(entity) = start_point_curve {
        commands.insert(BezierStartPoint(entity));
    };

    commands
}

fn create_bezier_curve(
    commands: &mut Commands,
    start_point: Position,
    start_handle: Position,
    end_handle: Position,
    end_point: Position,
) {
    let curve_1 = commands.spawn_empty().id();

    let start_handle_1 = create_handle(commands, start_handle, curve_1).id();
    let end_handle_1 = create_handle(commands, end_handle, curve_1).id();

    let start_point_1 = create_terminal_point(
        commands,
        start_point,
        &[start_handle_1],
        None,
        Some(curve_1),
    )
    .id();

    let end_point_1 =
        create_terminal_point(commands, end_point, &[end_handle_1], Some(curve_1), None).id();

    let curve_primitives = primitives::Primatives::new(&[], primitives::Type::LineStrip, 2.0);
    let curve_primitives = commands.spawn(curve_primitives).id();

    let bezier_curve = BezierCurve {
        start_point: start_point_1,
        start_handle: start_handle_1,
        end_handle: end_handle_1,
        end_point: end_point_1,
        curve_primitives,
    };

    commands.entity(curve_1).insert(bezier_curve);
}

fn initialize_bezier_curve(mut commands: Commands) {
    let offset = Position::new(0.0, 100.0);
    let start_point = Position::new(200.0, 240.0);
    let start_handle = Position::new(400.0, 456.0);
    let end_handle = Position::new(400.0, 24.0);
    let end_point = Position::new(600.0, 240.0);

    create_bezier_curve(
        &mut commands,
        start_point,
        start_handle,
        end_handle,
        end_point,
    );
    let (
        [start_point, start_handle, end_handle, end_point],
        [start_point_2, start_handle_2, end_handle_2, end_point_2],
    ) = split_bezier(
        start_point + offset,
        start_handle + offset,
        end_handle + offset,
        end_point + offset,
        0.3,
    );
    create_bezier_curve(
        &mut commands,
        start_point,
        start_handle,
        end_handle,
        end_point,
    );
    create_bezier_curve(
        &mut commands,
        start_point_2,
        start_handle_2,
        end_handle_2,
        end_point_2,
    );
}

#[derive(Default)]
struct ControlPoints([Position; 4]);

fn update_bezier_curve(
    commands: &mut Commands,
    bezier_curve: &BezierCurve,
    points: &mut Points,
    lines: &mut Lines,
    positions_query: &Query<(Ref<Position>, Option<&Selected>)>,
    primitives_query: &mut Query<&mut primitives::Primatives>,
    system: &Res<crate::my_time::Time>,
    control_points: &mut Local<ControlPoints>,
) {
    let control_points_query = positions_query.iter_many([
        bezier_curve.start_point,
        bezier_curve.start_handle,
        bezier_curve.end_handle,
        bezier_curve.end_point,
    ]);

    let mut control_points_changed = false;
    let mut start_selected = false;
    let mut end_selected = false;

    for (i, (point, selected)) in control_points_query.enumerate() {
        if point.is_changed() {
            control_points_changed = true;
        }

        // Clone here because many below processes can only take
        // pure position objects
        // TODO: Figure out a way to use either refs _or_ objects
        // below. May be able to make improvements in a lot of places
        // since right now we are doing a lot of unnecesary cloning/copying
        control_points.0[i] = point.as_ref().clone();

        if i == 0 && selected.is_some() {
            start_selected = true;
        }

        if i == 3 && selected.is_some() {
            end_selected = true;
        }
    }

    // TODO: Modify this to always have a constant speed and distance between points instead of
    // what it currently does
    let offset = system.elapsed / 4.0;
    let point_iterator =
        generate_bezier_points_with_offset(&control_points.0, Some(10), Some(offset), false);
    for point in point_iterator {
        points.draw_point(point, 10.0, Color::RED);
    }

    let [start_point_position, start_handle_position, end_handle_position, end_point_position] =
        control_points.0;

    let mut start_handle = commands.entity(bezier_curve.start_handle);
    if start_selected {
        lines.draw_line(start_point_position, start_handle_position);
        start_handle.remove::<Hidden>();
    } else {
        start_handle.insert(Hidden);
    }

    let mut end_handle = commands.entity(bezier_curve.end_handle);
    if end_selected {
        lines.draw_line(end_point_position, end_handle_position);
        end_handle.remove::<Hidden>();
    } else {
        end_handle.insert(Hidden);
    }

    if control_points_changed {
        let mut curve = primitives_query
            .get_mut(bezier_curve.curve_primitives)
            .unwrap();

        let curve_points = generate_bezier_points(&control_points.0);
        curve.set_positions(curve_points);
    }
}

fn update_bezier_curve_system(
    mut commands: Commands,
    bezier_curve_query: Query<&BezierCurve>,
    mut points: Points,
    mut lines: Lines,
    positions_query: Query<(Ref<Position>, Option<&Selected>)>,
    mut primitives_query: Query<&mut primitives::Primatives>,
    system: Res<crate::my_time::Time>,
    mut control_points: Local<ControlPoints>,
) {
    for bezier_curve in bezier_curve_query.iter() {
        update_bezier_curve(
            &mut commands,
            bezier_curve,
            &mut points,
            &mut lines,
            &positions_query,
            &mut primitives_query,
            &system,
            &mut control_points,
        );
    }
}

#[derive(Component)]
struct SolidWhenSelected;

fn solid_when_selected_system(
    mut stroke_query: Query<&mut Stroke, With<SolidWhenSelected>>,
    selected_added: Query<Entity, Added<Selected>>,
    mut selected_removed: RemovedComponents<Selected>,
) {
    for entity in selected_added.iter() {
        let Ok(mut stroke) = stroke_query.get_mut(entity) else {
            continue;
        };
        *stroke = Stroke::Solid;
    }

    for entity in selected_removed.read() {
        let Ok(mut stroke) = stroke_query.get_mut(entity) else {
            continue;
        };
        *stroke = Stroke::Outline;
    }
}

pub struct BezierPlugin;

impl Plugin for BezierPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Startup, initialize_bezier_curve);
        app.add_systems(Update, solid_when_selected_system);
        app.add_systems(PostUpdate, update_bezier_curve_system);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    #[allow(dead_code)]
    struct SplitError {
        t: f64,
        point_count: usize,
        curve_1_points: usize,
        point_index: usize,
        p1: Position,
        p2: Position,
        start_point: Position,
        start_handle: Position,
        end_handle: Position,
        end_point: Position,
    }

    fn test_split_granular(
        start_point: Position,
        start_handle: Position,
        end_handle: Position,
        end_point: Position,
        t: f64,
        point_count: usize,
    ) -> Result<(), SplitError> {
        let single_curve = generate_bezier_points_with_offset(
            &[start_point, start_handle, end_handle, end_point],
            Some(point_count),
            None,
            false,
        );

        let (curve_1, curve_2) = split_bezier(start_point, start_handle, end_handle, end_point, t);
        let curve_1_points = (point_count as f64 * t).round() as usize;

        let curve_1 =
            generate_bezier_points_with_offset(&curve_1, Some(curve_1_points), None, false);

        let curve_2 = generate_bezier_points_with_offset(
            &curve_2,
            Some(point_count - curve_1_points),
            None,
            false,
        );

        let final_points = curve_1.chain(curve_2);
        let zipped_points = single_curve.zip(final_points);

        for (i, (p1, p2)) in zipped_points.enumerate() {
            println!("{p1:?}, {p2:?} ");
            if p1 != p2 {
                return Err(SplitError {
                    start_point,
                    start_handle,
                    end_handle,
                    end_point,
                    t,
                    point_count,
                    curve_1_points,
                    point_index: i,
                    p1,
                    p2,
                });
            }
        }
        println!("====");

        Ok(())
    }

    fn test_split(t: f64, point_count: usize) -> Result<(), SplitError> {
        let start_point = Position::new(100.0, 100.0);
        let start_handle = Position::new(150.0, 100.0);
        let end_handle = Position::new(150.0, -100.0);
        let end_point = Position::new(100.0, -100.0);

        test_split_granular(
            start_point,
            start_handle,
            end_handle,
            end_point,
            t,
            point_count,
        )
    }

    #[test]
    fn split_bezier_simple() {
        test_split(0.5, 20).unwrap();
    }

    #[test]
    fn split_bezier_single() {
        test_split(0.5, 2).unwrap();
    }

    #[test]
    fn split_bezier_many() {
        test_split(0.5, 20000).unwrap();
    }

    #[test]
    fn split_bezier_all_t() {
        let subdivisions = 60;

        for i in 1..subdivisions {
            let t = i as f64 / subdivisions as f64;
            test_split(t, subdivisions).unwrap();
        }
    }
}
