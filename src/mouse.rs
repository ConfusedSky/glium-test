use bevy_ecs::system::Resource;

use crate::position::Position;

#[derive(Resource, Default)]
pub struct MousePosition {
    pub position: Position,
    pub previous_position: Position,
}
