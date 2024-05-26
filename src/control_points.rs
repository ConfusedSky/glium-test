use bevy_ecs::{
    component::Component,
    entity::Entity,
    system::{Commands, EntityCommands, Resource},
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

fn create_control_point<'commands>(commands: &'commands mut Commands, x: f32, y: f32) -> EntityCommands<'commands> {
    commands.spawn((
        Position::new(x, y),
        Point { size: 15.0 },
        Hoverable { radius: 20.0 },
        Draggable,
    ))
}

pub fn initialize_points(mut commands: Commands) {
    let start_handle = create_control_point(&mut commands, 400.0, 456.0).id();
    let end_handle = create_control_point(&mut commands, 400.0, 24.0).id();

    let start_point = create_control_point(&mut commands, 200.0, 240.0).id();
    let end_point = create_control_point(&mut commands, 600.0, 240.0).id();

    commands.insert_resource(ControlPoints {
        start_point,
        start_handle,
        end_handle,
        end_point,
    });
}
