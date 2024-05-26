use std::time::SystemTime;

use bevy_ecs::world::World;
use glium::{glutin::surface::WindowSurface, Display, Frame, Surface};

use crate::{
    bezier::{self, BezierCurve},
    point,
    position::Position,
    primitives,
};

pub struct RenderParams<'a> {
    pub display: &'a Display<WindowSurface>,
    pub target: &'a mut Frame,
    pub screen_size: &'a Position,
    pub timer: &'a SystemTime,
}

pub struct Renderer<'a> {
    display: Display<WindowSurface>,
    primitives_renderer: primitives::Renderer,
    points_renderer: point::Renderer<'a>,
}

impl Renderer<'_> {
    pub fn new(display: Display<WindowSurface>) -> Self {
        let primitives_renderer = primitives::Renderer::new(&display);
        let points_renderer = point::Renderer::new(&display);
        Self {
            display,
            primitives_renderer,
            points_renderer,
        }
    }

    pub fn draw(&mut self, world: &mut World, window_size: &Position, timer: &SystemTime) {
        let mut target = self.display.draw();
        target.clear_color(0.0, 0.0, 1.0, 1.0);

        let mut render_params = RenderParams {
            display: &self.display,
            target: &mut target,
            screen_size: &window_size,
            timer,
        };

        {
            let elapsed = timer.elapsed().unwrap().as_secs_f64() / 4.0;
            let points = world
                .get_resource::<BezierCurve>()
                .unwrap()
                .clone()
                .get_points(world);

            let p = bezier::generate_bezier_points_with_offset(&points, Some(10), Some(elapsed));
            let mut follow_points = world.query::<&mut point::Collection>().single_mut(world);
            follow_points.set_points(&p);
        }

        let mut query = world.query::<&mut primitives::Primatives>();
        for mut data in query.iter_mut(world) {
            self.primitives_renderer.draw(&mut render_params, &mut data);
        }

        let mut query = world.query::<&mut point::Collection>();
        for mut data in query.iter_mut(world) {
            self.points_renderer.draw(&mut render_params, &mut data);
        }

        self.points_renderer
            .draw_from_world(&mut render_params, world);

        target.finish().unwrap();
    }
}
