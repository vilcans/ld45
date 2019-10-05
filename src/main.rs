#![windows_subsystem = "windows"]

use serde::Deserialize;
use std::io::Read;

//use cgmath;
use ggez::nalgebra::Point2;
use ggez::nalgebra::Vector2;

use ggez;
use ggez::conf;
use ggez::event;
use ggez::filesystem::File;
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
    world_mesh: graphics::Mesh,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let ship = Ship {
            position: Point2::new(0.0, 0.0),
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

        let f = ggez::filesystem::open(ctx, "/mesh.dat")?;
        let world_mesh = load_meshes(ctx, f)?;

        let s = MainState { ship, world_mesh };
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
            MOVE_AMOUNT
        } else if input::keyboard::is_key_pressed(ctx, KeyCode::S) {
            -MOVE_AMOUNT
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
        let camera_position = self.ship.position;
        //let camera_position = Vector2::new(0.0, 0.0);
        {
            let (win_width, win_height) = graphics::drawable_size(ctx);
            let aspect = if win_height != 0.0 {
                win_width / win_height
            } else {
                1.0
            };
            let height = 500.0; // visible meters
            let width = height * aspect;
            let mut rect = graphics::Rect::new(-width * 0.5, height * 0.5, width, -height);
            rect.translate(Vector2::new(camera_position.x, camera_position.y));
            graphics::set_screen_coordinates(ctx, rect)?;
            println!("camera {:?} rect {:?}", camera_position, rect);
        }

        let draw_param = graphics::DrawParam::default();

        graphics::draw(ctx, &self.world_mesh, draw_param)?;
        graphics::draw(ctx, &self.ship.mesh, draw_param.dest(self.ship.position))?;
        graphics::present(ctx)?;
        Ok(())
    }
}

#[derive(Deserialize, Debug)]
struct RawMeshes {
    points: Vec<(f32, f32)>,
}

fn load_meshes(ctx: &mut Context, mut file: File) -> GameResult<graphics::Mesh> {
    let mut encoded = Vec::<u8>::new();
    file.read_to_end(&mut encoded).unwrap();

    let raw_meshes: RawMeshes = bincode::deserialize(&encoded[..]).unwrap();
    println!(
        "Loaded {} points: {:?}",
        raw_meshes.points.len(),
        raw_meshes
    );

    let points: Vec<Point2<f32>> = raw_meshes
        .points
        .into_iter()
        .map(|(x, y)| Point2::new(x, y))
        .collect();
    let mesh = graphics::MeshBuilder::new()
        .line(&points[..], 3.0, graphics::WHITE)?
        .build(ctx)?;
    Ok(mesh)
}

pub fn main() -> GameResult {
    let mut builder = ggez::ContextBuilder::new("Ludum Dare 45", "Martin Vilcans");

    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let base = path::PathBuf::from(manifest_dir.clone());
        builder = builder
            .add_resource_path(base.join("resources"))
            .add_resource_path(base.join("gen-resources"));
    } else {
        builder = builder.add_resource_path("./resources");
    }

    builder = builder.window_setup(conf::WindowSetup::default().title("Ludum Dare 45"));

    let (ctx, event_loop) = &mut builder.build()?;
    let state = &mut MainState::new(ctx)?;
    event::run(ctx, event_loop, state)
}
