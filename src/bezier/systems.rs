use bevy::ecs::{
    change_detection::DetectChanges,
    component::Component,
    entity::Entity,
    query::{Added, With},
    removal_detection::RemovedComponents,
    system::{Commands, Local, Query, Res},
    world::Ref,
};

use crate::{
    hidden::Hidden,
    position::Position,
    rendering::{
        point::Points,
        primitives::{self, Lines},
        Color, Stroke,
    },
    selection::Selected,
};

use super::{components::BezierCurve, generate_bezier_points, generate_bezier_points_with_offset};

fn update_bezier_curve(
    commands: &mut Commands,
    bezier_curve: &BezierCurve,
    points: &mut Points,
    lines: &mut Lines,
    positions_query: &Query<(Ref<Position>, Option<&Selected>)>,
    primitives_query: &mut Query<&mut primitives::Primatives>,
    system: &Res<crate::my_time::Time>,
    control_points: &mut Local<[Position; 4]>,
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
        control_points[i] = point.as_ref().clone();

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
        generate_bezier_points_with_offset(&control_points, Some(10), Some(offset), false);
    for point in point_iterator {
        points.draw_point(point, 10.0, Color::RED);
    }

    let [start_point_position, start_handle_position, end_handle_position, end_point_position] =
        control_points[..]
    else {
        unreachable!()
    };

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

        let curve_points = generate_bezier_points(&control_points);
        curve.set_positions(curve_points);
    }
}

pub fn update_bezier_curve_system(
    mut commands: Commands,
    bezier_curve_query: Query<&BezierCurve>,
    mut points: Points,
    mut lines: Lines,
    positions_query: Query<(Ref<Position>, Option<&Selected>)>,
    mut primitives_query: Query<&mut primitives::Primatives>,
    system: Res<crate::my_time::Time>,
    mut control_points: Local<[Position; 4]>,
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
pub struct SolidWhenSelected;

pub fn solid_when_selected_system(
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
