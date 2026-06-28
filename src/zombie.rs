use macroquad::prelude::*;
use crate::camera::Camera;

#[derive(PartialEq, Clone, Copy)]
pub enum ZombieState {
    Wandering,
    Chasing,
    Attacking,
}

pub struct Zombie {
    pub pos: Vec2,
    pub health: f32,
    pub state: ZombieState,
    pub wander_timer: f32,
    pub wander_dir: Vec2,
    pub alive: bool,
}

const SIGHT_RANGE: f32 = 200.0;
const ATTACK_RANGE: f32 = 22.0;
const CHASE_SPEED: f32 = 62.0;
const WANDER_SPEED: f32 = 22.0;

impl Zombie {
    pub fn new(pos: Vec2) -> Self {
        let angle = macroquad::rand::gen_range(0.0f32, std::f32::consts::TAU);
        Self {
            pos,
            health: 100.0,
            state: ZombieState::Wandering,
            wander_timer: macroquad::rand::gen_range(1.0f32, 3.0f32),
            wander_dir: vec2(angle.cos(), angle.sin()),
            alive: true,
        }
    }

    pub fn take_damage(&mut self, dmg: f32) {
        self.health -= dmg;
        if self.health <= 0.0 {
            self.alive = false;
        }
    }

    pub fn update(&mut self, dt: f32, player_pos: Vec2, is_solid: impl Fn(Vec2) -> bool) {
        if !self.alive { return; }

        let to_player = player_pos - self.pos;
        let dist = to_player.length();

        self.state = if dist < ATTACK_RANGE {
            ZombieState::Attacking
        } else if dist < SIGHT_RANGE {
            ZombieState::Chasing
        } else {
            ZombieState::Wandering
        };

        let movement = match self.state {
            ZombieState::Attacking => Vec2::ZERO,
            ZombieState::Chasing => {
                if dist > 0.0 { to_player / dist * CHASE_SPEED * dt } else { Vec2::ZERO }
            }
            ZombieState::Wandering => {
                self.wander_timer -= dt;
                if self.wander_timer <= 0.0 {
                    let angle = macroquad::rand::gen_range(0.0f32, std::f32::consts::TAU);
                    self.wander_dir = vec2(angle.cos(), angle.sin());
                    self.wander_timer = macroquad::rand::gen_range(1.5f32, 4.0f32);
                }
                self.wander_dir * WANDER_SPEED * dt
            }
        };

        if movement.length() > 0.0 {
            let next = self.pos + movement;
            if !is_solid(next) {
                self.pos = next;
            } else {
                // try sliding on axes
                let next_x = self.pos + vec2(movement.x, 0.0);
                if !is_solid(next_x) {
                    self.pos = next_x;
                } else {
                    let next_y = self.pos + vec2(0.0, movement.y);
                    if !is_solid(next_y) {
                        self.pos = next_y;
                    }
                }
            }
        }
    }

    pub fn draw(&self, cam: &Camera) {
        if !self.alive { return; }
        if !cam.is_visible(self.pos, 30.0) { return; }
        let sp = cam.world_to_screen(self.pos);

        // Body
        let body_color = Color::new(0.55, 0.1, 0.1, 1.0);
        draw_circle(sp.x, sp.y, 14.0, body_color);

        // Eyes
        draw_circle(sp.x - 5.0, sp.y - 4.0, 3.5, WHITE);
        draw_circle(sp.x + 5.0, sp.y - 4.0, 3.5, WHITE);
        draw_circle(sp.x - 5.0, sp.y - 4.0, 1.5, Color::new(0.8, 0.0, 0.0, 1.0));
        draw_circle(sp.x + 5.0, sp.y - 4.0, 1.5, Color::new(0.8, 0.0, 0.0, 1.0));

        // Health bar
        if self.health < 100.0 {
            draw_rectangle(sp.x - 14.0, sp.y - 22.0, 28.0, 4.0, Color::new(0.3, 0.0, 0.0, 0.8));
            draw_rectangle(sp.x - 14.0, sp.y - 22.0, 28.0 * (self.health / 100.0), 4.0, RED);
        }
    }
}
