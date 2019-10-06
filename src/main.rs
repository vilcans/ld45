#![windows_subsystem = "windows"]

use serde::Deserialize;
use std::io::Read;

use bit_vec::BitVec;
use std::collections::{HashMap, HashSet};

//use cgmath;
use ggez::nalgebra;
use ggez::nalgebra::Point2;
use ggez::nalgebra::Vector2;

use ggez;
use ggez::audio;
use ggez::audio::SoundSource;
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

const DEAD_TIMEOUT: f32 = 1.5;

const TICKS_PER_SECOND: u32 = 60;
const TICK_TIME: f32 = 1.0 / TICKS_PER_SECOND as f32;

const TURN_SPEED: f32 = 2.5;
const THRUST: f32 = 120.0;
const GRAVITY: f32 = 40.0;
const ENERGY_CONSERVATION: f32 = 0.6;

const SHIP_COLOR: u32 = 0x91e2db;

const VISIBLE_HEIGHT: f32 = 300.0;

const STROKE_WIDTH: f32 = 1.0;

const FILL_COLOR: u32 = 0x000000;
const WALL_COLOR: u32 = 0x2ca693;
const BACKGROUND_COLOR: u32 = 0x023f3c;

const FONT_SIZE: f32 = 28.0;

const LEVEL_EXTENTS: graphics::Rect = graphics::Rect {
    x: -500.0,
    y: -1000.0,
    w: 1000.0,
    h: 2000.0,
};

const COLLISION_MAP_WIDTH: u32 = 1024;
const COLLISION_MAP_HEIGHT: u32 = 2048;

#[derive(Debug, Clone, Copy)]
enum Level {
    One,
}

struct Ship {
    position: Point2<f32>,
    velocity: Vector2<f32>,
    // Angle in radians, 0 = pointing to the right, pi/2 = pointing up
    angle: f32,
    angular_velocity: f32,
    thrust: f32,

    alive: bool,
    dead_time: f32,
    turning_enabled: bool,
    thrust_enabled: bool,

    polygons: RawMeshes,
    meshes: Vec<graphics::Mesh>,
}

impl Ship {
    fn reset(&mut self, position: Point2<f32>) {
        self.position = position;
        self.velocity = Vector2::new(0.0, 0.0);
        self.angular_velocity = 0.0;
        self.thrust = 0.0;
        self.angle = std::f32::consts::FRAC_PI_2;
        self.alive = true;
        self.dead_time = 0.0;
    }

    fn update(&mut self, ctx: &mut Context) -> GameResult {
        self.angular_velocity = 0.0;
        self.thrust = 0.0;

        if self.alive {
            if self.turning_enabled
                && (input::keyboard::is_key_pressed(ctx, KeyCode::A)
                    || input::keyboard::is_key_pressed(ctx, KeyCode::Left))
            {
                self.angular_velocity += TURN_SPEED;
            }
            if self.turning_enabled
                && (input::keyboard::is_key_pressed(ctx, KeyCode::D)
                    || input::keyboard::is_key_pressed(ctx, KeyCode::Right))
            {
                self.angular_velocity -= TURN_SPEED;
            }
            if self.thrust_enabled
                && (input::keyboard::is_key_pressed(ctx, KeyCode::W)
                    || input::keyboard::is_key_pressed(ctx, KeyCode::Up))
            {
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

struct LevelState {
    level_meshes: Vec<graphics::Mesh>,
    collision_map: BitVec,
    triggers: HashMap<u32, Trigger>,
    shown_triggers: HashSet<u32>,
}

impl LevelState {
    fn get_spawn_position(&self) -> Point2<f32> {
        // The spawn position is a "trigger" with ID 0
        let spawn_trigger = self.triggers.get(&0u32).unwrap();
        Point2::new(
            (spawn_trigger.min_x + spawn_trigger.max_x) * 0.5,
            (spawn_trigger.min_y + spawn_trigger.max_y) * 0.5,
        )
    }

    fn get_collision(&self, position: Point2<f32>) -> bool {
        if let Some(i) = LevelState::get_collider_map_index(position) {
            return self.collision_map[i];
        }
        return false;
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
}

fn load_level(ctx: &mut Context, level: u32) -> GameResult<LevelState> {
    // Level

    let f = ggez::filesystem::open(ctx, format!("/level{:02}.dat", level))?;
    let raw_level_meshes = load_meshes(ctx, f)?;
    let level_meshes = create_drawables(
        ctx,
        &raw_level_meshes,
        Color::from_rgb_u32(FILL_COLOR),
        Color::from_rgb_u32(WALL_COLOR),
    )?;

    let triggers: HashMap<u32, Trigger> = raw_level_meshes
        .triggers
        .iter()
        .map(|t| (t.id, *t))
        .collect();

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
    if false {
        for y in 0..32 {
            for x in 0..32 {
                let bit = collision_map
                    .get(
                        ((y * COLLISION_MAP_HEIGHT / 32) * COLLISION_MAP_WIDTH
                            + (x * COLLISION_MAP_WIDTH / 32)) as usize,
                    )
                    .unwrap();
                print!("{}", if bit { 'X' } else { '.' });
            }
            println!();
        }
    }

    graphics::set_canvas(ctx, None);

    Ok(LevelState {
        level_meshes,
        collision_map,
        triggers,
        shown_triggers: HashSet::new(),
    })
}

struct MainState {
    ship: Ship,
    font: graphics::Font,
    ui_text: Option<graphics::Text>,
    level: Option<LevelState>,
    _ambient: audio::Source,
    ping: audio::Source,
    thrust_sound: audio::Source,
    explosion_sound: audio::Source,
}

impl MainState {
    fn new(ctx: &mut Context, starting_level: u32) -> GameResult<MainState> {
        // Audio
        let mut ambient = audio::Source::new(ctx, "/music.ogg").unwrap();
        let _ = ambient.play_detached();
        ambient.set_repeat(true);

        let ping = audio::Source::new(ctx, "/ping.ogg").unwrap();
        let mut thrust_sound = audio::Source::new(ctx, "/thrust.wav").unwrap();
        thrust_sound.set_volume(0.0);
        thrust_sound.set_repeat(true);
        let explosion_sound = audio::Source::new(ctx, "/explosion.ogg").unwrap();

        // Text

        let font = graphics::Font::new(ctx, "/font/font.ttf")?;

        // Ship

        let f = ggez::filesystem::open(ctx, "/ship.dat")?;
        let ship_polygons = load_meshes(ctx, f)?;
        let ship_meshes = create_drawables(
            ctx,
            &ship_polygons,
            Color::from_rgb_u32(FILL_COLOR),
            Color::from_rgb_u32(SHIP_COLOR),
        )?;

        let f = ggez::filesystem::open(ctx, "/ship-collider.dat")?;
        let collider_polygons = load_meshes(ctx, f)?;

        let level = load_level(ctx, starting_level)?;
        let ship = Ship {
            position: level.get_spawn_position(),
            velocity: Vector2::new(0.0, 0.0),
            angle: std::f32::consts::FRAC_PI_2,
            angular_velocity: 0.0,
            thrust: 0.0,
            polygons: collider_polygons,
            meshes: ship_meshes,
            alive: true,
            dead_time: 0.0,
            thrust_enabled: false,
            turning_enabled: false,
        };

        Ok(MainState {
            ship,
            font,
            ui_text: None,
            level: Some(level),
            _ambient: ambient,
            ping,
            thrust_sound,
            explosion_sound,
        })
    }

    fn update_during_play(&mut self, ctx: &mut Context) -> GameResult<Option<u32>> {
        let level = self.level.as_ref().unwrap();

        let mut hit_trigger = None;

        self.ship.update(ctx)?;
        while timer::check_update_time(ctx, TICKS_PER_SECOND) && hit_trigger.is_none() {
            self.ship.tick(ctx)?;

            if self.ship.alive {
                let ship_transform = nalgebra::Isometry2::new(
                    Vector2::new(self.ship.position.x, self.ship.position.y),
                    self.ship.angle,
                );
                //println!("Ship transform: {}", ship_transform);

                let mut collided = false;
                for poly in self.ship.polygons.polygons.iter() {
                    for &(x, y) in poly.iter() {
                        let point = ship_transform * Point2::new(x, y);
                        let hit = level.get_collision(point.into());
                        if hit {
                            //println!("Collided at {}", point);
                            collided = true;
                        }
                    }
                }

                if collided {
                    self.ship.alive = false;
                    let _ = self.explosion_sound.play();
                } else {
                    for (&trigger_id, trigger) in level.triggers.iter() {
                        if trigger.min_x <= self.ship.position.x
                            && self.ship.position.x < trigger.max_x
                            && trigger.min_y <= self.ship.position.y
                            && self.ship.position.y < trigger.max_y
                            && !level.shown_triggers.contains(&trigger_id)
                        {
                            println!("In trigger {}: {}", trigger_id, self.ship.position);
                            hit_trigger = Some(trigger_id);
                            break;
                        }
                    }
                }
            } else {
                self.ship.dead_time += TICK_TIME;
            }
        }
        if !self.ship.alive && self.ship.dead_time >= DEAD_TIMEOUT {
            self.show_text(ctx, "Ouch! ... I wonder why that felt familiar.");
        }
        Ok(hit_trigger)
    }

    fn execute_trigger(&mut self, ctx: &mut Context, trigger_id: u32) -> GameResult {
        self.level
            .as_mut()
            .unwrap()
            .shown_triggers
            .insert(trigger_id);

        let level = Level::One;
        match (level, trigger_id) {
            (_, 0) => {
                // ignore hitting the spawn point
            }
            (Level::One, 10) => {
                self.show_text(ctx, "What's this? What happened? Am I falling?");
            }
            (Level::One, 11) => {
                self.show_text(ctx, "I'm in some kind of aircraft. Can I control it?");
            }
            (Level::One, 12) => {
                self.show_text(ctx, "Nothing. I'm going to crash!");
            }
            (Level::One, 13) => {
                self.show_text(ctx, "Wait! I feel it... Go up!");
                self.ship.thrust_enabled = true;
            }
            (Level::One, 14) => {
                self.show_text(ctx, "Up! Up! Up!");
            }
            (Level::One, 20) => {
                self.show_text(ctx, "I think I know how to turn left and right...");
                self.ship.turning_enabled = true;
            }
            (Level::One, 21) => {
                self.show_text(ctx, "This feels stangely natural. I should be a pilot!");
            }
            (Level::One, 22) => {
                self.show_text(ctx, "Maybe I am a pilot? I don't remember anything.");
            }

            _ => {
                self.show_text(
                    ctx,
                    &format!(
                        "Hit unknown trigger {} on level {:?}. This is a bug.",
                        trigger_id, level
                    ),
                );
            }
        }

        Ok(())
    }

    fn show_text(&mut self, _ctx: &mut Context, t: &str) {
        let _ = self.ping.play();
        let mut text = graphics::Text::new(t);
        text.set_font(self.font, graphics::Scale::uniform(FONT_SIZE));
        self.ui_text = Some(text);
    }

    fn restart_level(&mut self) {
        self.ship
            .reset(self.level.as_ref().unwrap().get_spawn_position());
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        let ui_displayed = self.ui_text.is_some();
        if ui_displayed {
            if input::keyboard::is_key_pressed(ctx, KeyCode::Return) {
                self.ui_text = None;
                if !self.ship.alive {
                    // It's the game over text
                    self.restart_level();
                }
            } else {
                timer::sleep(timer::f64_to_duration(0.01));
            }
        }

        if !ui_displayed && self.level.is_some() {
            let r = self.update_during_play(ctx)?;
            if let Some(trigger_id) = r {
                self.execute_trigger(ctx, trigger_id)?;
            }
        }

        let thrust_volume = if self.ship.alive {
            if self.ui_text.is_none() {
                self.ship.thrust * 0.30 / THRUST
            } else {
                self.ship.thrust * 0.15 / THRUST
            }
        } else {
            0.0
        };
        self.thrust_sound.set_volume(thrust_volume);
        self.thrust_sound.play_later()?;

        // There must be a better way to make sure we waste the time?
        while timer::check_update_time(ctx, TICKS_PER_SECOND) {}
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, graphics::Color::from_rgb_u32(BACKGROUND_COLOR));

        let camera_position = self.ship.position;

        let (world_draw_rect, ui_draw_rect) = {
            let (win_width, win_height) = graphics::drawable_size(ctx);
            let aspect = if win_height != 0.0 {
                win_width / win_height
            } else {
                1.0
            };
            let height = VISIBLE_HEIGHT;
            let width = height * aspect;
            let mut world_rect = graphics::Rect::new(-width * 0.5, height * 0.5, width, -height);
            world_rect.translate(Vector2::new(camera_position.x, camera_position.y));

            let ui_rect = graphics::Rect::new(0.0, 0.0, 800.0, 800.0 * win_height / win_width);
            (world_rect, ui_rect)
        };

        let draw_param = graphics::DrawParam::default();

        graphics::set_screen_coordinates(ctx, world_draw_rect)?;

        // Draw level
        if let Some(level) = &self.level {
            for mesh in &level.level_meshes {
                graphics::draw(ctx, mesh, draw_param)?;
            }
        }

        // Draw ship
        if self.ship.alive {
            let ship_draw_param = draw_param
                .dest(self.ship.position)
                .rotation(self.ship.angle);
            for mesh in &self.ship.meshes {
                graphics::draw(ctx, mesh, ship_draw_param)?;
            }
        }

        // Draw UI

        graphics::set_screen_coordinates(ctx, ui_draw_rect)?;

        if let Some(text) = self.ui_text.as_ref() {
            graphics::draw(
                ctx,
                text,
                draw_param.color(graphics::Color::from_rgb_u32(0x00ff00)),
            )?;
        }

        graphics::present(ctx)?;
        Ok(())
    }
}

#[derive(Deserialize, Debug, Copy, Clone)]
struct Trigger {
    id: u32,
    min_x: f32,
    max_x: f32,
    min_y: f32,
    max_y: f32,
}

#[derive(Deserialize, Debug)]
struct RawMeshes {
    polygons: Vec<Vec<(f32, f32)>>,
    triggers: Vec<Trigger>,
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
    let args: Vec<String> = env::args().collect();
    let starting_level: u32 = if args.len() == 2 {
        args[1].parse::<u32>().unwrap_or(1)
    } else {
        1
    };

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
    let state = &mut MainState::new(ctx, starting_level)?;
    event::run(ctx, event_loop, state)
}
