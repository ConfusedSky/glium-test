use bevy_ecs::{
    change_detection::DetectChanges,
    component::Component,
    entity::Entity,
    system::{Commands, Query, Res},
};

use crate::{mouse::MousePosition, position::Position};

#[derive(Component)]
pub struct Hoverable {
    pub radius: f32,
}

#[derive(Component)]
pub struct Hovered;

pub fn search_for_hovered(
    mut commands: Commands,
    mouse_position: Res<MousePosition>,
    query: Query<(Entity, &Position, &Hoverable, Option<&Hovered>)>,
) {
    // If there has been no change in mouse position since last frame
    // We shouldn't waste time calculating differences
    if !mouse_position.is_changed() {
        return;
    }

    for (entity, position, hoverable, hovered) in query.iter() {
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
}
