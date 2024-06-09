use bevy::ecs::{component::Component, entity::Entity};

// Components that exist for reverse lookup of a curve from a point
#[derive(Component)]
#[allow(dead_code)]
pub struct BezierHandle(pub Entity);

// Start and end points are different components so a mid point
// of a spline can have both
#[derive(Component)]
#[allow(dead_code)]
pub struct BezierStartPoint(pub Entity);
#[derive(Component)]
#[allow(dead_code)]
pub struct BezierEndPoint(pub Entity);

#[derive(Component, Clone)]
pub struct BezierCurve {
    pub start_point: Entity,
    pub start_handle: Entity,
    pub end_handle: Entity,
    pub end_point: Entity,

    pub curve_primitives: Entity,
}
