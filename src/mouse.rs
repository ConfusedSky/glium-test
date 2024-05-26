use bevy_ecs::system::Resource;

use crate::position::Position;

#[derive(Resource, Default)]
pub struct MousePosition {
    position: Position,
    previous_position: Position,
}

impl MousePosition {
    pub fn position(&self) -> Position {
        self.position
    }
    pub fn previous_position(&self) -> Position {
        self.previous_position
    }

    pub fn update(&mut self, new_position: Position) {
        self.previous_position = self.position;
        self.position = new_position;
    }
}

// pub struct MouseButtons {
// pub left_mouse_pressed
// }
