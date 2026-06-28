use macroquad::prelude::*;
use crate::camera::{Camera, TILE_SIZE};
use crate::noise::NoiseSystem;
use crate::sprites::Sprites;
use crate::map::Map;

const WANDER_SPEED:  f32 = 28.0;
const CHASE_SPEED:   f32 = 48.0;
const SIGHT_RANGE:   f32 = TILE_SIZE * 4.5;   // 4.5 tiles
const HEARING_RANGE: f32 = TILE_SIZE * 9.0;    // 9 tiles
const ATTACK_RANGE:  f32 = TILE_SIZE * 1.1;    // just over 1 tile

#[derive(Clone, Debug)]
enum WalkerState {
    Wandering,
    Investigating { target: Vec2, patience: f32 },
    Chasing,
}

pub struct Walker {
    pub pos:   Vec2,
    pub facing: Vec2,
    pub health: f32,
    pub alive:  bool,
    state: WalkerState,
    wander_timer: f32,
    wander_dir:   Vec2,
    groan_timer:  f32,
    pub alerted: bool,
}

impl Walker {
    pub fn new(pos: Vec2) -> Self {
        let angle = macroquad::rand::gen_range(0.0f32, std::f32::consts::TAU);
        Self {
            pos,
            facing: vec2(0.0, 1.0),
            health: 100.0,
            alive: true,
            state: WalkerState::Wandering,
            wander_timer: macroquad::rand::gen_range(1.0f32, 4.0f32),
            wander_dir: vec2(angle.cos(), angle.sin()),
            groan_timer: macroquad::rand::gen_range(3.0f32, 9.0f32),
            alerted: false,
        }
    }

    pub fn take_damage(&mut self, dmg: f32) {
        self.health -= dmg;
        if self.health <= 0.0 { self.alive = false; }
    }

    pub fn can_attack_player(&self, player_pos: Vec2) -> bool {
        (self.pos - player_pos).length() < ATTACK_RANGE
    }

    pub fn update(
        &mut self,
        dt: f32,
        player_pos: Vec2,
        player_flashlight_on: bool,
        map: &Map,
        noise: &mut NoiseSystem,
    ) {
        if !self.alive { return; }

        let to_player = player_pos - self.pos;
        let dist_to_player = to_player.length();

        // ── Groan (attracts nearby walkers) ────────────────────────────────
        self.groan_timer -= dt;
        if self.groan_timer <= 0.0 {
            self.groan_timer = macroquad::rand::gen_range(6.0f32, 14.0f32);
            noise.emit(self.pos, TILE_SIZE * 5.0, 0.5);
        }

        // ── Sight check (limited if flashlight off) ─────────────────────────
        let sight_range = if player_flashlight_on { SIGHT_RANGE } else { SIGHT_RANGE * 0.4 };
        let can_see_player = dist_to_player < sight_range
            && self.facing.dot(to_player.normalize_or_zero()) > 0.3;

        // ── State machine ───────────────────────────────────────────────────
        self.state = match &self.state {
            WalkerState::Chasing => {
                if can_see_player { WalkerState::Chasing }
                else { WalkerState::Investigating { target: player_pos, patience: 6.0 } }
            }
            WalkerState::Investigating { target, patience } => {
                if can_see_player {
                    self.alerted = true;
                    WalkerState::Chasing
                } else if *patience <= 0.0 || (self.pos - *target).length() < 24.0 {
                    WalkerState::Wandering
                } else {
                    WalkerState::Investigating { target: *target, patience: patience - dt }
                }
            }
            WalkerState::Wandering => {
                if can_see_player {
                    self.alerted = true;
                    WalkerState::Chasing
                } else if let Some(npos) = noise.loudest_near(self.pos, HEARING_RANGE) {
                    WalkerState::Investigating { target: npos, patience: 8.0 }
                } else {
                    WalkerState::Wandering
                }
            }
        };

        // ── Movement ────────────────────────────────────────────────────────
        let speed = match &self.state {
            WalkerState::Wandering          => WANDER_SPEED,
            WalkerState::Investigating { .. }=> WANDER_SPEED * 1.4,
            WalkerState::Chasing            => CHASE_SPEED,
        };

        let desired_dir = match &self.state {
            WalkerState::Chasing => to_player.normalize_or_zero(),
            WalkerState::Investigating { target, .. } => {
                (*target - self.pos).normalize_or_zero()
            }
            WalkerState::Wandering => {
                self.wander_timer -= dt;
                if self.wander_timer <= 0.0 {
                    let angle = macroquad::rand::gen_range(0.0f32, std::f32::consts::TAU);
                    self.wander_dir = vec2(angle.cos(), angle.sin());
                    self.wander_timer = macroquad::rand::gen_range(2.0f32, 5.0f32);
                }
                self.wander_dir
            }
        };

        if desired_dir.length() > 0.05 {
            self.facing = desired_dir;
            let move_delta = desired_dir * speed * dt;
            let next = self.pos + move_delta;
            let r = 10.0f32;

            if !map.is_solid_at_world(next.x, next.y)
                && !map.is_solid_at_world(next.x - r, next.y)
                && !map.is_solid_at_world(next.x + r, next.y)
                && !map.is_solid_at_world(next.x, next.y - r)
                && !map.is_solid_at_world(next.x, next.y + r)
            {
                self.pos = next;
            } else {
                let nx = self.pos + vec2(move_delta.x, 0.0);
                let ny = self.pos + vec2(0.0, move_delta.y);
                if !map.is_solid_at_world(nx.x, nx.y) { self.pos = nx; }
                else if !map.is_solid_at_world(ny.x, ny.y) { self.pos = ny; }
                else {
                    // Unstick: randomize wander dir
                    let angle = macroquad::rand::gen_range(0.0f32, std::f32::consts::TAU);
                    self.wander_dir = vec2(angle.cos(), angle.sin());
                }
            }
        }
    }

    pub fn is_chasing(&self) -> bool { matches!(self.state, WalkerState::Chasing) }

    pub fn draw(&self, cam: &Camera, sprites: &Sprites) {
        if !self.alive { return; }
        if !cam.is_visible(self.pos, 64.0) { return; }
        let sp = cam.world_to_screen(self.pos);
        let facing_left = self.facing.x < -0.1;
        let col = if self.facing.y.abs() >= self.facing.x.abs() {
            if self.facing.y >= 0.0 { 4 } else { 5 }
        } else {
            if facing_left { 6 } else { 7 }
        };
        sprites.draw_character(col, sp, WHITE, false);

        // Health bar
        if self.health < 100.0 {
            let bw = 24.0;
            draw_rectangle(sp.x - bw/2.0, sp.y - 70.0, bw, 3.0, Color::new(0.2,0.0,0.0,0.9));
            draw_rectangle(sp.x - bw/2.0, sp.y - 70.0, bw * (self.health/100.0), 3.0,
                           Color::new(0.75, 0.05, 0.05, 1.0));
        }

        // Alert flash
        if self.alerted {
            let t = get_time() as f32;
            let alpha = ((t * 4.0).sin().abs() * 0.7).min(0.7);
            draw_text("!", sp.x - 4.0, sp.y - 68.0, 20.0,
                      Color::new(1.0, 0.15, 0.05, alpha));
        }
    }
}
