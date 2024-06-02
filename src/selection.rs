use bevy::{
    app::{Plugin, Update},
    ecs::{
        component::Component,
        entity::Entity,
        event::EventReader,
        query::With,
        system::{Commands, Local, ParamSet, Query, Res, ResMut, Resource},
    },
    input::{mouse::MouseButton, ButtonInput},
    window::CursorMoved,
};

use crate::position::Position;

#[derive(Resource, Default)]
struct HeldItems {
    items: Vec<Entity>,
}

/// If this component is added to an entity
/// If gains the Hovered component when the
/// mouse is within radius of the entities
/// [Position]
#[derive(Component)]
pub struct Hoverable {
    pub radius: f32,
}

/// This component is added and removed to
/// components with the [Hoverable] component
/// if the mouse is with the [Hoverable]'s radius
#[derive(Component, Default)]
#[component(storage = "SparseSet")]
pub struct Hovered {
    connected: bool,
}

/// This component allows items to be dragged
/// if they have the [Hoverable] and [Position]
/// components as well
#[derive(Component)]
pub struct Draggable;

/// This entity is connected to another entity
/// Any hover or drag events are mirrored for
/// the host and the connected entity
#[derive(Component)]
pub struct Connection(pub Entity);

fn mouse_moved(
    mut commands: Commands,
    held: Res<HeldItems>,
    mut queries: ParamSet<(
        Query<(
            Entity,
            &Position,
            &Hoverable,
            Option<&Hovered>,
            Option<&Connection>,
        )>,
        Query<&mut Position>,
    )>,
    mut cursor_evr: EventReader<CursorMoved>,
    mut mouse_position_previous: Local<Position>,
) {
    let new_mouse_position = cursor_evr.read().last();
    // If there has been no change in mouse position since last frame
    // We shouldn't waste time calculating differences
    let Some(mouse_position) = new_mouse_position else {
        return;
    };
    let mouse_position = Position::from([mouse_position.position.x, mouse_position.position.y]);

    // Only look for things to hover if we don't have a selection
    if held.items.is_empty() {
        let hover_query = queries.p0();
        for (entity, position, hoverable, hovered, connection) in hover_query.iter() {
            let distance_squared = position.distance_squared(&mouse_position);
            let radius_squared = hoverable.radius.powi(2);

            // If we are in range the commponent under the
            // mouse is now hovered, otherwise if the component
            // is hovered it should no longer be hovered
            if distance_squared < radius_squared {
                commands.entity(entity).insert(Hovered::default());

                // If there is a connected entity also set hovered on that element as well
                if let Some(Connection(other)) = connection {
                    commands.entity(*other).insert(Hovered { connected: true });
                }
            } else if let Some(Hovered { connected }) = hovered {
                // Connected hovers should be taken care of when
                // the host entity loses hover
                if *connected {
                    continue;
                }

                commands.entity(entity).remove::<Hovered>();

                if let Some(Connection(other)) = connection {
                    commands.entity(*other).remove::<Hovered>();
                }
            }
        }
    // Otherwise we want to move the selection
    } else {
        let mut drag_query = queries.p1();
        let difference = mouse_position - *mouse_position_previous;
        for entity in &held.items {
            // The saved entity could have been removed
            // in between selection and mouse_moved
            if let Ok(mut position) = drag_query.get_mut(*entity) {
                *position = *position + difference;
            }
        }
    }

    *mouse_position_previous = mouse_position
}

fn grab_selection(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut held: ResMut<HeldItems>,
    hover_query: Query<Entity, (With<Hovered>, With<Draggable>)>,
) {
    if mouse_buttons.just_pressed(MouseButton::Left) {
        // Put all items that are being hovered into the selection
        held.items.extend(hover_query.iter());
    } else if mouse_buttons.just_released(MouseButton::Left) {
        // Clear all selection if the mouse is let go
        held.items.clear();
    }
}

pub struct SelectionPlugin;

impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<HeldItems>();
        app.add_systems(Update, (mouse_moved, grab_selection));
    }
}
