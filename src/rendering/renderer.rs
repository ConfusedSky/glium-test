use std::{num::NonZeroU32, time::Instant};

use bevy::{
    app::Plugin,
    ecs::{
        event::EventReader,
        system::{Commands, Resource},
        world::World,
    },
    prelude::{FromWorld, IntoSystemConfigs, Local, Query, Res, ResMut, With},
    window::{PrimaryWindow, RequestRedraw, Window, WindowResized},
    winit::WinitWindows,
};
use glium::{glutin::surface::WindowSurface, Display, Frame, Surface};
use glutin::{
    context::NotCurrentGlContext,
    display::{GetGlDisplay, GlDisplay},
};

use super::{
    point::{self, PointsData},
    primitives::{self, LinesData},
};
use crate::matrix::Mat3;
use crate::position::Position;

pub struct RenderParams<'a> {
    pub display: &'a Display<WindowSurface>,
    pub target: &'a mut Frame,
    pub world_to_view: &'a Mat3,
}

struct Renderer<'a> {
    display: Display<WindowSurface>,
    primitives_renderer: primitives::Renderer,
    points_renderer: point::Renderer<'a>,
}

#[derive(Resource)]
struct WindowSize(pub Position);

#[derive(Resource)]
pub struct WorldToView(pub Mat3);

#[derive(Default, Resource)]
pub struct CameraPosition(pub Position);

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

    pub fn draw(&mut self, world: &mut World, window_size: &Mat3) {
        let mut target = self.display.draw();
        target.clear_color(0.0, 0.0, 1.0, 1.0);

        let mut render_params = RenderParams {
            display: &self.display,
            target: &mut target,
            world_to_view: &window_size,
        };

        let mut query = world.query::<&mut primitives::Primatives>();
        for mut data in query.iter_mut(world) {
            self.primitives_renderer.draw(&mut render_params, &mut data);
        }

        let mut data = world.resource_mut::<LinesData>();
        self.primitives_renderer
            .draw_immediate(&mut render_params, &mut data);

        self.points_renderer
            .draw_from_world(&mut render_params, world);

        target.finish().unwrap();
    }
}

fn initialize_renderer(world: &mut World) {
    use raw_window_handle::HasRawWindowHandle;

    let event_loop = world.non_send_resource::<winit::event_loop::EventLoop<RequestRedraw>>();

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
    let world_to_view = world.resource::<WorldToView>().0.clone();
    let camera_position = world.resource::<CameraPosition>().0.clone();
    let world_to_view =
        world_to_view.multiply(&Mat3::translate(-camera_position.x(), -camera_position.y()));
    let mut renderer = world.remove_non_send_resource::<Renderer>().unwrap();
    renderer.draw(world, &world_to_view);
    world.insert_non_send_resource(renderer);
}

fn update_window_size(mut commands: Commands, mut window_resized: EventReader<WindowResized>) {
    let last_event = window_resized.read().last();
    let Some(last_event) = last_event else {
        return;
    };

    let window_size = Position::from([last_event.width, last_event.height]);
    println!("Window size set to: {:?}", window_size);

    commands.insert_resource(WorldToView(Mat3::world_to_view(window_size)));
    commands.insert_resource(WindowSize(window_size));
}

struct UpdateCameraPositionData {
    old_time: Instant,
}

impl FromWorld for UpdateCameraPositionData {
    fn from_world(_world: &mut World) -> Self {
        Self {
            old_time: Instant::now(),
        }
    }
}

fn update_camera_position(
    q_windows: Query<&Window, With<PrimaryWindow>>,
    mut camera_position: ResMut<CameraPosition>,
    window_size: Res<WindowSize>,
    mut data: Local<UpdateCameraPositionData>,
) {
    let now = Instant::now();
    let delta = now.duration_since(data.old_time).as_secs_f32();
    data.old_time = now;

    let Some(new_position) = q_windows.single().cursor_position() else {
        return;
    };
    let mouse_position = Position::from([new_position.x, new_position.y]);
    let margin = 50.0;
    let speed = 150.0;
    let distance = speed * delta;

    if mouse_position.x() < margin {
        camera_position.0 = camera_position.0 + Position::from([-distance, 0.0]);
    } else if mouse_position.x() > window_size.0.x() - margin {
        camera_position.0 = camera_position.0 + Position::from([distance, 0.0]);
    }

    if mouse_position.y() < margin {
        camera_position.0 = camera_position.0 + Position::from([0.0, -distance]);
    } else if mouse_position.y() > window_size.0.y() - margin {
        camera_position.0 = camera_position.0 + Position::from([0.0, distance]);
    }
}

pub struct RenderingPlugin;

impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<PointsData>();
        app.init_resource::<LinesData>();
        app.init_resource::<CameraPosition>();
        app.add_systems(
            bevy::app::First,
            (update_window_size, update_camera_position).chain(),
        );
        app.add_systems(bevy::app::Last, render_system);
    }

    // [initialize_renderer] needs to happen in finish so that we can use the eventloop created in
    // the in the winit plugin
    fn finish(&self, app: &mut bevy::prelude::App) {
        initialize_renderer(&mut app.world);
    }
}
