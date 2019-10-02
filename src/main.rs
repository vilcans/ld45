#![windows_subsystem = "windows"]

//use cgmath;
use ggez::nalgebra::Point2;

use ggez;
use ggez::conf;
use ggez::event;
use ggez::graphics;
use ggez::input;
use ggez::input::keyboard::KeyCode;
use ggez::nalgebra as na;
use ggez::timer;
use ggez::{Context, GameResult};

use std::env;
use std::path;

struct MainState {
    image1: graphics::Image,
    circle: graphics::Mesh,
    circle_position: Point2<f32>,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let image1 = graphics::Image::new(ctx, "/ld-logo.png")?;

        let circle = graphics::Mesh::new_circle(
            ctx,
            graphics::DrawMode::fill(),
            na::Point2::new(0.0, 0.0),
            100.0,
            2.0,
            graphics::WHITE,
        )?;

        let s = MainState {
            image1: image1,
            circle: circle,
            circle_position: na::Point2::new(0.0, 50.0),
        };
        Ok(s)
    }
}

impl MainState {
    fn tick(&mut self, ctx: &mut Context) -> GameResult {
        const MOVE_SPEED: f32 = 5.0f32;
        if input::keyboard::is_key_pressed(ctx, KeyCode::A) {
            self.circle_position.x -= MOVE_SPEED;
        }
        if input::keyboard::is_key_pressed(ctx, KeyCode::D) {
            self.circle_position.x += MOVE_SPEED;
        }
        if input::keyboard::is_key_pressed(ctx, KeyCode::W) {
            self.circle_position.y -= MOVE_SPEED;
        }
        if input::keyboard::is_key_pressed(ctx, KeyCode::S) {
            self.circle_position.y += MOVE_SPEED;
        }
        Ok(())
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        while timer::check_update_time(ctx, 60) {
            self.tick(ctx)?;
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, [0.1, 0.2, 0.3, 1.0].into());

        graphics::draw(ctx, &self.circle, (self.circle_position,))?;

        graphics::draw(ctx, &self.image1, (na::Point2::new(10.0, 10.0),))?;

        graphics::present(ctx)?;
        Ok(())
    }
}

pub fn main() -> GameResult {
    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        path::PathBuf::from("./resources")
    };

    let window_setup = conf::WindowSetup::default().title("Ludum Dare 45");

    let builder = ggez::ContextBuilder::new("Ludum Dare 45", "Martin Vilcans")
        .add_resource_path(resource_dir)
        .window_setup(window_setup);

    let (ctx, event_loop) = &mut builder.build()?;
    let state = &mut MainState::new(ctx)?;
    event::run(ctx, event_loop, state)
}
