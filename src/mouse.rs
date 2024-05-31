use bevy::{
    app::{First, Plugin},
    ecs::{
        event::EventReader,
        system::{ResMut, Resource},
    },
    window::CursorMoved,
};

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

fn update_mouse_position(
    mut mouse_position: ResMut<MousePosition>,
    mut cursor_evr: EventReader<CursorMoved>,
) {
    let last_update = cursor_evr.read().last();

    if let Some(mouse_moved) = last_update {
        mouse_position.previous_position = mouse_position.position;
        mouse_position.position = Position::from([mouse_moved.position.x, mouse_moved.position.y]);
    }
}

pub struct MousePlugin;

impl Plugin for MousePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<MousePosition>();

        app.add_systems(First, update_mouse_position);
    }
}
