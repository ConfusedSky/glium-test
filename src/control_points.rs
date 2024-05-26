use bevy_ecs::{
    component::Component, entity::Entity, system::{Commands, Resource}
};

use crate::position::Position;

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

fn create_control_point(x: f32, y: f32) -> (Position, Point) {
    (Position::new(x, y), Point { size: 15.0 })
}

pub fn initialize_points(mut commands: Commands) {
    let start_point = commands.spawn(create_control_point(200.0, 240.0)).id();
    let start_handle = commands.spawn(create_control_point(400.0, 456.0)).id();
    let end_handle = commands.spawn(create_control_point(400.0, 24.0)).id();
    let end_point = commands.spawn(create_control_point(600.0, 240.0)).id();

    commands.insert_resource(ControlPoints {
        start_point,
        start_handle,
        end_handle,
        end_point,
    });
}
