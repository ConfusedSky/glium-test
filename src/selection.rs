use bevy_ecs::{
    change_detection::DetectChanges,
    component::Component,
    entity::Entity,
    query::With,
    system::{Commands, ParamSet, Query, Res, ResMut, Resource},
};

use crate::{
    mouse::{MouseButtons, MousePosition},
    position::Position,
};

#[derive(Resource, Default)]
pub struct HeldItems {
    pub items: Vec<Entity>,
}

#[derive(Component)]
pub struct Hoverable {
    pub radius: f32,
}

#[derive(Component)]
pub struct Hovered;

#[derive(Component)]
pub struct Draggable;

pub fn mouse_moved(
    mut commands: Commands,
    held: Res<HeldItems>,
    mouse_position: Res<MousePosition>,
    mut queries: ParamSet<(
        Query<(Entity, &Position, &Hoverable, Option<&Hovered>)>,
        Query<&mut Position>,
    )>,
) {
    // If there has been no change in mouse position since last frame
    // We shouldn't waste time calculating differences
    if !mouse_position.is_changed() {
        return;
    }

    // Only look for things to hover if we don't have a selection
    if held.items.is_empty() {
        let hover_query = queries.p0();
        for (entity, position, hoverable, hovered) in hover_query.iter() {
            let distance_squared = position.distance_squared(&mouse_position.position());
            let radius_squared = hoverable.radius.powi(2);

            // If we are in range the commponent under the
            // mouse is now hovered, otherwise if the component
            // is hovered it should no longer be hovered
            if distance_squared < radius_squared {
                commands.entity(entity).insert(Hovered);
            } else if hovered.is_some() {
                commands.entity(entity).remove::<Hovered>();
            }
        }
    // Otherwise we want to move the selection
    } else {
        let mut drag_query = queries.p1();
        let difference = mouse_position.position() - mouse_position.previous_position();
        for entity in &held.items {
            // The saved entity could have been removed
            // in between selection and mouse_moved
            if let Ok(mut position) = drag_query.get_mut(*entity) {
                *position = *position + difference;
            }
        }
    }
}

pub fn grab_selection(
    // mut commands: Commands,
    mouse_buttons: Res<MouseButtons>,
    mut held: ResMut<HeldItems>,
    hover_query: Query<Entity, (With<Hovered>, With<Draggable>)>,
) {
    if !mouse_buttons.is_changed() {
        return;
    }

    if mouse_buttons.left_mouse_pressed() {
        // Put all items that are being hovered into the selection
        let data: Vec<_> = hover_query.iter().collect();
        held.items = data;
    } else if mouse_buttons.left_mouse_released() {
        // Clear all selection if the mouse is let go
        held.items.clear();
    }
}
