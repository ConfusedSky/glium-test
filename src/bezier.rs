use bevy::{
    app::{Plugin, PostUpdate, Startup, Update},
    ecs::{
        bundle::Bundle,
        entity::Entity,
        system::{Commands, EntityCommands},
    },
};
use systems::SolidWhenSelected;

use crate::{
    hidden::Hidden,
    position::Position,
    rendering::{point::Point, primitives, Stroke},
    selection::{Connection, Draggable, Hoverable, Selectable},
};

mod components;
mod systems;

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
fn split_bezier(control_points: &[Position; 4], t: f64) -> ([Position; 4], [Position; 4]) {
    let [start_point, start_handle, end_handle, end_point] = *control_points;

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
        components::BezierHandle(curve),
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
        commands.insert(components::BezierEndPoint(entity));
    };

    if let Some(entity) = start_point_curve {
        commands.insert(components::BezierStartPoint(entity));
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

    let bezier_curve = components::BezierCurve {
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
        &[
            start_point + offset,
            start_handle + offset,
            end_handle + offset,
            end_point + offset,
        ],
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

pub struct BezierPlugin;

impl Plugin for BezierPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Startup, initialize_bezier_curve);
        app.add_systems(Update, systems::solid_when_selected_system);
        app.add_systems(PostUpdate, systems::update_bezier_curve_system);
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
        control_points: [Position; 4],
        t: f64,
        point_count: usize,
    ) -> Result<(), SplitError> {
        let single_curve =
            generate_bezier_points_with_offset(&control_points, Some(point_count), None, false);

        let (curve_1, curve_2) = split_bezier(&control_points, t);
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
            if p1 != p2 {
                return Err(SplitError {
                    start_point: control_points[0],
                    start_handle: control_points[1],
                    end_handle: control_points[2],
                    end_point: control_points[2],
                    t,
                    point_count,
                    curve_1_points,
                    point_index: i,
                    p1,
                    p2,
                });
            }
        }

        Ok(())
    }

    fn test_split(t: f64, point_count: usize) -> Result<(), SplitError> {
        let start_point = Position::new(100.0, 100.0);
        let start_handle = Position::new(150.0, 100.0);
        let end_handle = Position::new(150.0, -100.0);
        let end_point = Position::new(100.0, -100.0);

        test_split_granular(
            [start_point, start_handle, end_handle, end_point],
            t,
            point_count,
        )
    }

    #[test]
    fn split_simple() {
        test_split(0.5, 20).unwrap();
    }

    #[test]
    fn split_single() {
        test_split(0.5, 2).unwrap();
    }

    #[test]
    fn split_many() {
        test_split(0.5, 20000).unwrap();
    }

    #[test]
    fn split_all_t() {
        let subdivisions = 60;

        for i in 1..subdivisions {
            let t = i as f64 / subdivisions as f64;
            test_split(t, subdivisions).unwrap();
        }
    }

    #[test]
    #[ignore]
    fn split_fuzz() {
        use rand::prelude::*;
        for i in 0..10000 {
            let mut rand = rand::thread_rng();
            let mut position_rand = || rand.gen_range(-1000.0..1000.0);
            let points = [Position::new(position_rand(), position_rand()); 4];

            let result = test_split_granular(
                points,
                // We want some well behaved values here so we don't fail automatically
                // due to point size missmatch
                0.5,
                rand.gen_range(1..5000) * 2,
            );

            if let Err(result) = result {
                eprintln!("Error in iteration {i}: ");
                eprintln!("{result:#?}");
                panic!();
            }
        }
    }
}
