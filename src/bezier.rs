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
/// t must be 0 <= t <= 1
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
    generate_bezier_points_with_offset(control_points, None, None)
}

fn generate_bezier_points_with_offset(
    control_points: &[Position; 4],
    subdivisions: Option<usize>,
    offset: Option<f64>,
) -> impl Iterator<Item = Position> {
    let offset = offset.unwrap_or_default();
    let subdivisions = subdivisions.unwrap_or(60);
    let mut shape_points = Vec::with_capacity(subdivisions);

    // Add one to the subdivisions if there is no offset so
    // if perfectly fills the space
    // if you add one to the subdivisions and there is
    // an offset there will be overlapping points
    for i in 0..subdivisions + if offset <= 0.1 { 1 } else { 0 } {
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

// Components that exist for reverse lookup of a curve from a point
#[derive(Component)]
struct BezierHandle(Entity);

// Start and end points are different components so a mid point
// of a spline can have both
#[derive(Component)]
struct BezierStartPoint(Entity);
#[derive(Component)]
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
    fn new(x: f32, y: f32) -> Self {
        Self {
            position: Position::new(x, y),
            point: Point { size: 15.0 },
            hoverable: Hoverable { radius: 20.0 },
            draggable: Draggable,
            stroke: Stroke::Outline,
        }
    }
}

fn create_handle<'c>(
    commands: &'c mut Commands,
    x: f32,
    y: f32,
    curve: Entity,
) -> EntityCommands<'c> {
    commands.spawn((
        BaseControlPointBundle::new(x, y),
        Hidden,
        BezierHandle(curve),
    ))
}

fn create_terminal_point<'c>(
    commands: &'c mut Commands,
    x: f32,
    y: f32,
    connections: &[Entity],
    // Set this to the curve that this terminal is
    // the end point for
    end_point_curve: Option<Entity>,
    // Set this to the curve that this terminal is
    // the start point for
    start_point_curve: Option<Entity>,
) -> EntityCommands<'c> {
    let mut commands = commands.spawn((
        BaseControlPointBundle::new(x, y),
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

fn initialize_bezier_curve(mut commands: Commands) {
    let curve_1 = commands.spawn_empty().id();
    let curve_2 = commands.spawn_empty().id();

    let start_handle_1 = create_handle(&mut commands, 400.0, 456.0, curve_1).id();
    let end_handle_1 = create_handle(&mut commands, 400.0, 24.0, curve_1).id();
    let start_handle_2 = create_handle(&mut commands, 800.0, 456.0, curve_2).id();
    let end_handle_2 = create_handle(&mut commands, 800.0, 24.0, curve_2).id();

    let start_point_1 = create_terminal_point(
        &mut commands,
        200.0,
        240.0,
        &[start_handle_1],
        None,
        Some(curve_1),
    )
    .id();

    let end_point_1 = create_terminal_point(
        &mut commands,
        600.0,
        240.0,
        &[end_handle_1, start_handle_2],
        Some(curve_1),
        Some(curve_2),
    )
    .id();

    let start_point_2 = end_point_1;
    let end_point_2 = create_terminal_point(
        &mut commands,
        1000.0,
        240.0,
        &[end_handle_2],
        Some(curve_2),
        None,
    )
    .id();

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

    let curve_primitives = primitives::Primatives::new(&[], primitives::Type::LineStrip, 2.0);
    let curve_primitives = commands.spawn(curve_primitives).id();

    let bezier_curve = BezierCurve {
        start_point: start_point_2,
        start_handle: start_handle_2,
        end_handle: end_handle_2,
        end_point: end_point_2,
        curve_primitives,
    };

    commands.entity(curve_2).insert(bezier_curve);
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

    let elapsed = system.elapsed / 4.0;
    let point_iterator =
        generate_bezier_points_with_offset(&control_points.0, Some(10), Some(elapsed));
    for point in point_iterator {
        points.draw_point(point, 10.0, Color::RED);
    }

    if start_selected {
        // Draw start handle line
        lines.draw_line(control_points.0[0], control_points.0[1]);
        commands
            .entity(bezier_curve.start_handle)
            .remove::<Hidden>();
    } else {
        commands.entity(bezier_curve.start_handle).insert(Hidden);
    }

    if end_selected {
        // Draw end handle line
        lines.draw_line(control_points.0[2], control_points.0[3]);
        commands.entity(bezier_curve.end_handle).remove::<Hidden>();
    } else {
        commands.entity(bezier_curve.end_handle).insert(Hidden);
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
