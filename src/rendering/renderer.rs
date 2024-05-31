use std::num::NonZeroU32;

use bevy::{
    app::Plugin, ecs::{system::Resource, world::World}, window::RequestRedraw, winit::WinitWindows
};
use glium::{glutin::surface::WindowSurface, Display, Frame, Surface};
use glutin::{context::NotCurrentGlContext, display::{GetGlDisplay, GlDisplay}};

use super::{point::{self, PointsData}, primitives};
use crate::position::Position;

pub struct RenderParams<'a> {
    pub display: &'a Display<WindowSurface>,
    pub target: &'a mut Frame,
    pub screen_size: &'a Position,
}

struct Renderer<'a> {
    display: Display<WindowSurface>,
    primitives_renderer: primitives::Renderer,
    points_renderer: point::Renderer<'a>,
}

#[derive(Resource)]
pub struct WindowSize(pub Position);

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

        let mut query = world.query::<&mut primitives::Primatives>();
        for mut data in query.iter_mut(world) {
            self.primitives_renderer.draw(&mut render_params, &mut data);
        }

        self.points_renderer
            .draw_from_world(&mut render_params, world);

        target.finish().unwrap();
    }
}

fn initialize_renderer(world: &mut World) {
    use raw_window_handle::HasRawWindowHandle;

    let event_loop =
        world.non_send_resource::<winit::event_loop::EventLoop<RequestRedraw>>();

    // First we start by opening a new Window
    let display_builder = glutin_winit::DisplayBuilder::new();
    let config_template_builder = glutin::config::ConfigTemplateBuilder::new();
    let (_, gl_config) = display_builder
        .build(&event_loop, config_template_builder, |mut configs| {
            // Just use the first configuration since we don't have any special preferences here
            configs.next().unwrap()
        })
        .unwrap();

    let winit_data = world.non_send_resource::<WinitWindows>();
    assert_eq!(winit_data.windows.len(), 1);

    let window = winit_data.windows.values().next().unwrap();
    let (width, height): (u32, u32) = window.inner_size().into();

    let attrs = glutin::surface::SurfaceAttributesBuilder::<glutin::surface::WindowSurface>::new()
        .build(
            window.raw_window_handle(),
            NonZeroU32::new(width).unwrap(),
            NonZeroU32::new(height).unwrap(),
        );

    let surface = unsafe {
        gl_config
            .display()
            .create_window_surface(&gl_config, &attrs)
            .unwrap()
    };

    let context_attributes =
        glutin::context::ContextAttributesBuilder::new().build(Some(window.raw_window_handle()));
    let current_context = Some(unsafe {
        gl_config
            .display()
            .create_context(&gl_config, &context_attributes)
            .expect("failed to create context")
    })
    .unwrap()
    .make_current(&surface)
    .unwrap();
    let display = Display::from_context_surface(current_context, surface).unwrap();

    let renderer = Renderer::new(display);

    world.insert_non_send_resource(renderer);
}

fn render_system(world: &mut World) {
    let window_size = world.resource::<WindowSize>().0.clone();
    let mut renderer = world.remove_non_send_resource::<Renderer>().unwrap();
    renderer.draw(world, &window_size);
    world.insert_non_send_resource(renderer);
}

pub struct RenderingPlugin;

impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<PointsData>();
        app.add_systems(bevy::app::Last, render_system);
    }

    // This needs to happen in finish so that we can use the eventloop created in
    // the in the winit plugin
    fn finish(&self, app: &mut bevy::prelude::App) {
        initialize_renderer(&mut app.world);
    }
}