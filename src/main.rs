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
use ggez::graphics::Color;
use ggez::input;
use ggez::input::keyboard::KeyCode;
use ggez::timer;
use ggez::{Context, GameResult};

use std::env;
use std::path;

const TICKS_PER_SECOND: u32 = 60;
const TICK_TIME: f32 = 1.0 / TICKS_PER_SECOND as f32;

const SHIP_COLOR: u32 = 0x91e2db;

const VISIBLE_HEIGHT: f32 = 200.0;

const STROKE_WIDTH: f32 = 1.0;

const FILL_COLOR: u32 = 0x000000;
const WALL_COLOR: u32 = 0x2ca693;
const BACKGROUND_COLOR: u32 = 0x023f3c;

struct Ship {
    position: Point2<f32>,
    velocity: Vector2<f32>,
    meshes: Vec<graphics::Mesh>,
}

struct MainState {
    ship: Ship,
    level_meshes: Vec<graphics::Mesh>,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let f = ggez::filesystem::open(ctx, "/ship.dat")?;
        let ship_meshes = load_meshes(
            ctx,
            f,
            Color::from_rgb_u32(FILL_COLOR),
            Color::from_rgb_u32(SHIP_COLOR),
        )?;

        let ship = Ship {
            position: Point2::new(0.0, 0.0),
            velocity: Vector2::new(0.0, 0.0),
            meshes: ship_meshes,
        };

        let f = ggez::filesystem::open(ctx, "/mesh.dat")?;
        let level_meshes = load_meshes(
            ctx,
            f,
            Color::from_rgb_u32(FILL_COLOR),
            Color::from_rgb_u32(WALL_COLOR),
        )?;

        let s = MainState { ship, level_meshes };
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
        graphics::clear(ctx, graphics::Color::from_rgb_u32(BACKGROUND_COLOR));
        let camera_position = self.ship.position;
        //let camera_position = Vector2::new(0.0, 0.0);
        {
            let (win_width, win_height) = graphics::drawable_size(ctx);
            let aspect = if win_height != 0.0 {
                win_width / win_height
            } else {
                1.0
            };
            let height = VISIBLE_HEIGHT;
            let width = height * aspect;
            let mut rect = graphics::Rect::new(-width * 0.5, height * 0.5, width, -height);
            rect.translate(Vector2::new(camera_position.x, camera_position.y));
            graphics::set_screen_coordinates(ctx, rect)?;
        }

        let draw_param = graphics::DrawParam::default();

        // Draw level
        for mesh in &self.level_meshes {
            graphics::draw(ctx, mesh, draw_param)?;
        }

        // Draw ship
        let ship_draw_param = draw_param.dest(self.ship.position);
        for mesh in &self.ship.meshes {
            graphics::draw(ctx, mesh, ship_draw_param)?;
        }

        graphics::present(ctx)?;
        Ok(())
    }
}

#[derive(Deserialize, Debug)]
struct RawMeshes {
    polygons: Vec<Vec<(f32, f32)>>,
}

fn load_meshes(
    ctx: &mut Context,
    mut file: File,
    fill_color: graphics::Color,
    line_color: graphics::Color,
) -> GameResult<Vec<graphics::Mesh>> {
    let mut encoded = Vec::<u8>::new();
    file.read_to_end(&mut encoded).unwrap();

    let raw_meshes: RawMeshes = bincode::deserialize(&encoded[..]).unwrap();

    let mut meshes = Vec::<graphics::Mesh>::new();
    for polygon in raw_meshes.polygons {
        let points: Vec<Point2<f32>> = polygon
            .iter()
            .map(|(x, y)| Point2::<f32>::new(*x, *y))
            .collect();

        let filled_mesh = graphics::MeshBuilder::new()
            .polygon(
                graphics::DrawMode::Fill(graphics::FillOptions::default()),
                &points[..],
                fill_color,
            )?
            .build(ctx)?;
        meshes.push(filled_mesh);

        let wall_mesh = graphics::MeshBuilder::new()
            .polygon(
                graphics::DrawMode::Stroke(
                    graphics::StrokeOptions::default().with_line_width(STROKE_WIDTH),
                ),
                &points[..],
                line_color,
            )?
            .build(ctx)?;

        meshes.push(wall_mesh);
    }

    Ok(meshes)
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
