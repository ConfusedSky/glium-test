use bevy_ecs::system::Resource;
use winit::event::{ElementState, MouseButton, WindowEvent};

use crate::position::Position;

#[derive(Resource, Default)]
pub struct MousePosition {
    position: Position,
    previous_position: Position,
}

impl MousePosition {
    #[allow(dead_code)]
    pub fn position(&self) -> Position {
        self.position
    }

    #[allow(dead_code)]
    pub fn previous_position(&self) -> Position {
        self.previous_position
    }

    pub fn update(&mut self, new_position: Position) {
        self.previous_position = self.position;
        self.position = new_position;
    }
}

#[derive(Resource, Default, Debug)]
pub struct MouseButtons {
    left_mouse_pressed: bool,
    left_mouse_down: bool,
    left_mouse_released: bool
}

impl MouseButtons {
    pub fn update(&mut self, state: ElementState, button: MouseButton) {
        if button == MouseButton::Left {
            if state.is_pressed() {
                self.left_mouse_pressed = true;
                self.left_mouse_down = true;
            } else {
                self.left_mouse_down = false;
                self.left_mouse_released = true;
            }
        }
    }

    pub fn end_frame(&mut self) {
        self.left_mouse_pressed = false;
        self.left_mouse_released = false;
    }

    #[allow(dead_code)]
    pub fn left_mouse_pressed(&self) -> bool {
        self.left_mouse_pressed
    }

    #[allow(dead_code)]
    pub fn left_mouse_down(&self) -> bool {
        self.left_mouse_down
    }

    #[allow(dead_code)]
    pub fn left_mouse_released(&self) -> bool {
        self.left_mouse_released
    }
}
