#![windows_subsystem = "windows"]

//use cgmath;
use ggez::nalgebra::Point2;
use ggez::nalgebra::Vector2;

use ggez;
use ggez::conf;
use ggez::event;
use ggez::graphics;
use ggez::input;
use ggez::input::keyboard::KeyCode;
use ggez::timer;
use ggez::{Context, GameResult};

use std::env;
use std::path;

const TICKS_PER_SECOND: u32 = 60;
const TICK_TIME: f32 = 1.0 / TICKS_PER_SECOND as f32;

const SHIP_COLOR: u32 = 0x00ff00;
const SHIP_WIDTH: f32 = 20.0;
const SHIP_HEIGHT: f32 = 15.0;

struct Ship {
    position: Point2<f32>,
    velocity: Vector2<f32>,
    mesh: graphics::Mesh,
}

struct MainState {
    ship: Ship,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let ship = Ship {
            position: Point2::new(50.0, 50.0),
            velocity: Vector2::new(0.0, 0.0),
            mesh: graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::Stroke(graphics::StrokeOptions::DEFAULT.with_line_width(3.0)),
                graphics::Rect::new(
                    -SHIP_WIDTH * 0.5,
                    -SHIP_HEIGHT * 0.5,
                    SHIP_WIDTH * 0.5,
                    SHIP_HEIGHT * 0.5,
                ),
                graphics::Color::from_rgb_u32(SHIP_COLOR),
            )?,
        };

        let s = MainState { ship };
        Ok(s)
    }
}

impl Ship {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        const MOVE_AMOUNT: f32 = 100.0f32;
        self.velocity.x = if input::keyboard::is_key_pressed(ctx, KeyCode::A) {
            -MOVE_AMOUNT
        } else if input::keyboard::is_key_pressed(ctx, KeyCode::D) {
            MOVE_AMOUNT
        } else {
            0.0
        };
        self.velocity.y = if input::keyboard::is_key_pressed(ctx, KeyCode::W) {
            -MOVE_AMOUNT
        } else if input::keyboard::is_key_pressed(ctx, KeyCode::S) {
            MOVE_AMOUNT
        } else {
            0.0
        };
        Ok(())
    }

    fn tick(&mut self, _ctx: &mut Context) -> GameResult {
        self.position += self.velocity * TICK_TIME;
        Ok(())
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        self.ship.update(ctx)?;
        while timer::check_update_time(ctx, TICKS_PER_SECOND) {
            self.ship.tick(ctx)?;
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, [0.1, 0.2, 0.3, 1.0].into());
        graphics::draw(ctx, &self.ship.mesh, (self.ship.position,))?;
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
