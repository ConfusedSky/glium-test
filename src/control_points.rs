use bevy_ecs::{
    component::Component,
    entity::Entity,
    system::{Commands, Resource},
};

use crate::{
    position::Position,
    selection::{Draggable, Hoverable},
};

#[derive(Resource)]
pub struct ControlPoints {
    pub start_point: Entity,
    pub start_handle: Entity,
    pub end_handle: Entity,
    pub end_point: Entity,
}

#[derive(Resource)]
pub struct ControlPointArray(pub [Position; 4]);

// Move this back over to point.rs after the refactor is complete
#[derive(Component)]
pub struct Point {
    pub size: f32,
}

fn create_control_point(mut commands: Commands, x: f32, y: f32) -> Entity {
    commands
        .spawn((
            Position::new(x, y),
            Point { size: 15.0 },
            Hoverable { radius: 20.0 },
            Draggable,
        ))
        .id()
}

pub fn initialize_points(mut commands: Commands) {
    let start_handle = create_control_point(commands.reborrow(), 400.0, 456.0);
    let end_handle = create_control_point(commands.reborrow(), 400.0, 24.0);

    let start_point = create_control_point(commands.reborrow(), 200.0, 240.0);
    let end_point = create_control_point(commands.reborrow(), 600.0, 240.0);

    commands.insert_resource(ControlPoints {
        start_point,
        start_handle,
        end_handle,
        end_point,
    });
}
