use bevy_ecs::{
    entity::Entity,
    query::Changed,
    system::{Commands, EntityCommands, Query, Res, Resource},
    world::{Mut, World},
};

use crate::{
    point::Point,
    position::Position,
    primitives,
    selection::{Connection, Draggable, Hoverable},
};

fn bezier(
    start_point: Position,
    start_handle: Position,
    end_handle: Position,
    end_point: Position,
    t: f64,
) -> Position {
    let t_inv = 1.0 - t;
    let t1 = t_inv.powi(3);
    let t2 = 3.0 * t_inv.powi(2) * t;
    let t3 = 3.0 * t_inv * t.powi(2);
    let t4 = t.powi(3);

    t1 * start_point + t2 * start_handle + t3 * end_handle + t4 * end_point
}

pub fn generate_bezier_points(control_points: &[Position]) -> Vec<Position> {
    generate_bezier_points_with_offset(control_points, None, None)
}

pub fn generate_bezier_points_with_offset(
    control_points: &[Position],
    subdivisions: Option<usize>,
    offset: Option<f64>,
) -> Vec<Position> {
    let subdivisions = subdivisions.unwrap_or(60);
    let mut shape_points = Vec::with_capacity(subdivisions);
    let offset = offset.unwrap_or_default();

    for i in 0..subdivisions {
        let t = if offset > 0.0 {
            let t = i as f64 / subdivisions as f64 + offset;
            t.fract()
        } else {
            i as f64 / subdivisions as f64
        };

        let point = bezier(
            control_points[0],
            control_points[1],
            control_points[2],
            control_points[3],
            t,
        );
        shape_points.push(point);
    }

    shape_points
}

#[derive(Resource, Clone)]
pub struct BezierCurve {
    pub start_point: Entity,
    pub start_handle: Entity,
    pub end_handle: Entity,
    pub end_point: Entity,

    pub handles: Entity,
    pub curve: Entity,
}

impl BezierCurve {
    pub fn get_points<'a>(&self, world: &mut World) -> Vec<Position> {
        world
            .query::<&Position>()
            .get_many(
                world,
                [
                    self.start_point,
                    self.start_handle,
                    self.end_handle,
                    self.end_point,
                ],
            )
            .unwrap()
            .into_iter()
            .cloned()
            .collect()
    }

    fn update_handles(&self, world: &mut World, points: &[Position]) {
        let mut handles: Mut<primitives::Primatives> = world.get_mut(self.handles).unwrap();
        handles.set_positions(points);
    }
    fn update_curve(&self, world: &mut World, points: &[Position]) {
        let mut curve: Mut<primitives::Primatives> = world.get_mut(self.curve).unwrap();
        let curve_points: Vec<_> = generate_bezier_points(points);
        curve.set_positions(&curve_points);
    }

    pub fn update(&self, world: &mut World) {
        let points = self.get_points(world);

        self.update_handles(world, &points);
        self.update_curve(world, &points);
    }
}

fn create_control_point<'c>(commands: &'c mut Commands, x: f32, y: f32) -> EntityCommands<'c> {
    commands.spawn((
        Position::new(x, y),
        Point { size: 15.0 },
        Hoverable { radius: 20.0 },
        Draggable,
    ))
}

pub fn initialize_bezier_curve(mut commands: Commands) {
    let start_handle = create_control_point(&mut commands, 400.0, 456.0).id();
    let end_handle = create_control_point(&mut commands, 400.0, 24.0).id();

    let start_point = create_control_point(&mut commands, 200.0, 240.0)
        .insert(Connection(start_handle))
        .id();
    let end_point = create_control_point(&mut commands, 600.0, 240.0)
        .insert(Connection(end_handle))
        .id();

    let handles = primitives::Primatives::new(&[], primitives::Type::Line, 2.0);
    let handles = commands.spawn(handles).id();

    let curve = primitives::Primatives::new(&[], primitives::Type::LineStrip, 2.0);
    let curve = commands.spawn(curve).id();

    let resource = BezierCurve {
        start_point,
        start_handle,
        end_handle,
        end_point,
        handles,
        curve,
    };
    commands.add(move |world: &mut World| {
        resource.update(world);
        world.insert_resource(resource);
    });
}

pub fn update_bezier_curve(
    mut commands: Commands,
    query: Query<(), Changed<Position>>,
    bezier_curve: Res<BezierCurve>,
) {
    let bezier_curve = bezier_curve.clone();

    // Look at each point if any of them have a position that has changed
    let curve_changed = query
        .iter_many([
            bezier_curve.start_point,
            bezier_curve.start_handle,
            bezier_curve.end_handle,
            bezier_curve.end_point,
        ])
        .next();

    if curve_changed.is_some() {
        commands.add(move |world: &mut World| {
            bezier_curve.update(world);
        });
    }
}
