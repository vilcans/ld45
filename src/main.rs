#![windows_subsystem = "windows"]

use serde::Deserialize;
use std::io::Read;

use bit_vec::BitVec;

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

const TURN_SPEED: f32 = 2.5;
const THRUST: f32 = 120.0;
const GRAVITY: f32 = 40.0;
const ENERGY_CONSERVATION: f32 = 0.6;

const SHIP_COLOR: u32 = 0x91e2db;

const VISIBLE_HEIGHT: f32 = 200.0;

const STROKE_WIDTH: f32 = 1.0;

const FILL_COLOR: u32 = 0x000000;
const WALL_COLOR: u32 = 0x2ca693;
const BACKGROUND_COLOR: u32 = 0x023f3c;

const LEVEL_EXTENTS: graphics::Rect = graphics::Rect {
    x: -500.0,
    y: -500.0,
    w: 1000.0,
    h: 1000.0,
};

const COLLISION_MAP_WIDTH: u32 = 1024;
const COLLISION_MAP_HEIGHT: u32 = 1024;

struct Ship {
    position: Point2<f32>,
    velocity: Vector2<f32>,
    // Angle in radians, 0 = pointing to the right, pi/2 = pointing up
    angle: f32,
    angular_velocity: f32,
    thrust: f32,
    meshes: Vec<graphics::Mesh>,
    alive: bool,
}

struct MainState {
    ship: Ship,
    level_meshes: Vec<graphics::Mesh>,
    collision_map: BitVec,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        // Ship

        let f = ggez::filesystem::open(ctx, "/ship.dat")?;
        let raw_ship_meshes = load_meshes(ctx, f)?;
        let ship_meshes = create_drawables(
            ctx,
            &raw_ship_meshes,
            Color::from_rgb_u32(FILL_COLOR),
            Color::from_rgb_u32(SHIP_COLOR),
        )?;

        let ship = Ship {
            position: Point2::new(0.0, 0.0),
            velocity: Vector2::new(0.0, 0.0),
            angle: std::f32::consts::FRAC_PI_2,
            angular_velocity: 0.0,
            thrust: 0.0,
            meshes: ship_meshes,
            alive: true,
        };

        // Level

        let f = ggez::filesystem::open(ctx, "/mesh.dat")?;
        let raw_level_meshes = load_meshes(ctx, f)?;
        let level_meshes = create_drawables(
            ctx,
            &raw_level_meshes,
            Color::from_rgb_u32(FILL_COLOR),
            Color::from_rgb_u32(WALL_COLOR),
        )?;

        // Render collision map

        let canvas = graphics::Canvas::new(
            ctx,
            COLLISION_MAP_WIDTH as u16,
            COLLISION_MAP_HEIGHT as u16,
            conf::NumSamples::One,
        )?;

        graphics::set_canvas(ctx, Some(&canvas));
        graphics::clear(ctx, [0.0, 0.0, 0.0, 0.0].into());
        graphics::set_screen_coordinates(ctx, LEVEL_EXTENTS)?;

        let draw_param = graphics::DrawParam::default();
        for mesh in &level_meshes {
            graphics::draw(ctx, mesh, draw_param)?;
        }
        graphics::present(ctx)?;

        let image = canvas.into_inner();
        let pixels = image.to_rgba8(ctx)?;
        assert!(pixels.len() == (COLLISION_MAP_WIDTH * COLLISION_MAP_HEIGHT * 4) as usize);
        let collision_map = BitVec::from_fn(
            COLLISION_MAP_WIDTH as usize * COLLISION_MAP_HEIGHT as usize,
            |i| {
                let a = pixels[i * 4 + 3];
                a >= 0x80
            },
        );

        // Print collision map
        if true {
            for y in 0..32 {
                for x in 0..32 {
                    let bit = collision_map
                        .get(
                            ((y * COLLISION_MAP_HEIGHT / 32) * COLLISION_MAP_WIDTH
                                + (x * COLLISION_MAP_WIDTH / 32))
                                as usize,
                        )
                        .unwrap();
                    print!("{}", if bit { 'X' } else { '.' });
                }
                println!();
            }
        }

        graphics::set_canvas(ctx, None);

        let s = MainState {
            ship,
            level_meshes,
            collision_map,
        };
        Ok(s)
    }

    fn get_collider_map_index(position: Point2<f32>) -> Option<usize> {
        let r = ((position.y - LEVEL_EXTENTS.top()) * COLLISION_MAP_HEIGHT as f32 / LEVEL_EXTENTS.h)
            .round() as i32;
        if r < 0 || r >= COLLISION_MAP_HEIGHT as i32 {
            println!("Point is outside: {:?} collision row {}", position, r);
            return None;
        }
        let r = r as u32;

        let c = ((position.x - LEVEL_EXTENTS.left()) * COLLISION_MAP_WIDTH as f32 / LEVEL_EXTENTS.w)
            .round() as i32;
        if c < 0 || c >= COLLISION_MAP_WIDTH as i32 {
            println!("Point is outside: {:?} collision column {}", position, c);
            return None;
        }
        let c = c as u32;

        return Some(((COLLISION_MAP_HEIGHT - 1 - r) * COLLISION_MAP_WIDTH + c) as usize);
    }

    fn get_collision(&self, position: Point2<f32>) -> bool {
        if let Some(i) = MainState::get_collider_map_index(position) {
            return self.collision_map[i];
        }
        return false;
    }
}

impl Ship {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        self.angular_velocity = 0.0;
        self.thrust = 0.0;

        if self.alive {
            if input::keyboard::is_key_pressed(ctx, KeyCode::A) {
                self.angular_velocity += TURN_SPEED;
            }
            if input::keyboard::is_key_pressed(ctx, KeyCode::D) {
                self.angular_velocity -= TURN_SPEED;
            }
            if input::keyboard::is_key_pressed(ctx, KeyCode::W) {
                self.thrust = THRUST;
            }
        }
        Ok(())
    }

    fn tick(&mut self, _ctx: &mut Context) -> GameResult {
        if !self.alive {
            return Ok(());
        }
        self.angle =
            (self.angle + self.angular_velocity * TICK_TIME) % (std::f32::consts::PI * 2.0);

        self.velocity *= ENERGY_CONSERVATION.powf(TICK_TIME);
        let direction = Vector2::new(self.angle.cos(), self.angle.sin());
        let mut acceleration = self.thrust * direction;
        acceleration.y -= GRAVITY;
        self.velocity += acceleration * TICK_TIME;
        self.position += self.velocity * TICK_TIME;
        Ok(())
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        self.ship.update(ctx)?;
        while timer::check_update_time(ctx, TICKS_PER_SECOND) {
            self.ship.tick(ctx)?;

            if self.ship.alive {
                let position = self.ship.position;
                let hit = self.get_collision(position.into());
                if hit {
                    self.ship.alive = false;
                    println!("Collided at {}", position);
                }
            }
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
        let ship_draw_param = draw_param
            .dest(self.ship.position)
            .rotation(self.ship.angle);
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

fn load_meshes(_ctx: &mut Context, mut file: File) -> GameResult<RawMeshes> {
    let mut encoded = Vec::<u8>::new();
    file.read_to_end(&mut encoded).unwrap();
    let m: RawMeshes = bincode::deserialize(&encoded[..]).unwrap();
    Ok(m)
}

fn create_drawables(
    ctx: &mut Context,
    raw_meshes: &RawMeshes,
    fill_color: graphics::Color,
    line_color: graphics::Color,
) -> GameResult<Vec<graphics::Mesh>> {
    let mut meshes = Vec::<graphics::Mesh>::new();
    for polygon in raw_meshes.polygons.iter() {
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
