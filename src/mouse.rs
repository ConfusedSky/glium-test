use bevy::{
    app::{First, Plugin},
    ecs::{
        event::EventReader,
        system::{ResMut, Resource},
    },
    window::CursorMoved,
};
use winit::event::{ElementState, MouseButton};

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

    #[allow(dead_code)]
    pub fn update(&mut self, new_position: Position) {
        self.previous_position = self.position;
        self.position = new_position;
    }
}

#[derive(Resource, Default, Debug)]
pub struct MouseButtons {
    left_mouse_pressed: bool,
    left_mouse_down: bool,
    left_mouse_released: bool,
}

impl MouseButtons {
    #[allow(dead_code)]
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

    #[allow(dead_code)]
    pub fn needs_end_frame(&self) -> bool {
        self.left_mouse_pressed || self.left_mouse_released
    }

    #[allow(dead_code)]
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

fn update_mouse_position(
    mut mouse_position: ResMut<MousePosition>,
    mut cursor_evr: EventReader<CursorMoved>,
) {
    let last_update = cursor_evr.read().last();

    if let Some(mouse_moved) = last_update {
        mouse_position.previous_position = mouse_position.previous_position;
        mouse_position.position = Position::from([mouse_moved.position.x, mouse_moved.position.y]);
    }
}

pub struct MousePlugin;

impl Plugin for MousePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<MousePosition>();
        app.init_resource::<MouseButtons>();

        app.add_systems(First, update_mouse_position);
    }
}
