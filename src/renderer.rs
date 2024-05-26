use bevy_ecs::world::World;
use glium::{glutin::surface::WindowSurface, Display, Frame, Surface};

use crate::{point, position::Position, primitives};

pub struct RenderParams<'a> {
    pub display: &'a Display<WindowSurface>,
    pub target: &'a mut Frame,
    pub screen_size: &'a Position,
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

    pub fn draw(&mut self, world: &mut World, window_size: &Position) {
        let mut target = self.display.draw();
        target.clear_color(0.0, 0.0, 1.0, 1.0);

        let mut render_params = RenderParams {
            display: &self.display,
            target: &mut target,
            screen_size: &window_size,
        };

        // let mut query = world.query::<&mut primitives::Data>();
        // for mut data in query.iter_mut(world) {
        // self.primitives_renderer.draw(&mut render_params, &mut data);
        // }

        // let mut query = world.query::<&mut point::Data>();
        // for mut data in query.iter_mut(world) {
        // self.points_renderer.draw(&mut render_params, &mut data);
        // }

        self.points_renderer
            .draw_from_world(&mut render_params, world);

        target.finish().unwrap();
    }
}
