use macroquad::prelude::*;
use crate::camera::Camera;

pub struct Particle {
    pub pos:  Vec2,
    vel:      Vec2,
    life:     f32,
    max_life: f32,
    radius:   f32,
    color:    Color,
    kind:     ParticleKind,
}

enum ParticleKind {
    Blood,
    Dust,
    NoiseRing { start_r: f32 },
}

pub struct Particles {
    list: Vec<Particle>,
}

impl Particles {
    pub fn new() -> Self { Self { list: Vec::new() } }

    pub fn emit_blood(&mut self, pos: Vec2, count: usize) {
        for _ in 0..count {
            let angle = macroquad::rand::gen_range(0.0f32, std::f32::consts::TAU);
            let speed = macroquad::rand::gen_range(18.0f32, 55.0f32);
            let life  = macroquad::rand::gen_range(0.4f32, 0.9f32);
            self.list.push(Particle {
                pos,
                vel: vec2(angle.cos(), angle.sin()) * speed,
                life, max_life: life,
                radius: macroquad::rand::gen_range(1.5f32, 3.5f32),
                color: Color::new(0.62, 0.04, 0.04, 1.0),
                kind: ParticleKind::Blood,
            });
        }
    }

    pub fn emit_noise_ring(&mut self, pos: Vec2, max_r: f32) {
        self.list.push(Particle {
            pos,
            vel: Vec2::ZERO,
            life: 0.7,
            max_life: 0.7,
            radius: max_r,
            color: Color::new(1.0, 0.85, 0.4, 0.0),
            kind: ParticleKind::NoiseRing { start_r: 4.0 },
        });
    }

    pub fn update(&mut self, dt: f32) {
        for p in &mut self.list {
            p.life -= dt;
            p.pos += p.vel * dt;
            p.vel *= 1.0 - (dt * 6.0).min(1.0);
        }
        self.list.retain(|p| p.life > 0.0);
    }

    pub fn draw(&self, cam: &Camera) {
        for p in &self.list {
            if !cam.is_visible(p.pos, p.radius + 16.0) { continue; }
            let sp = cam.world_to_screen(p.pos);
            let frac = p.life / p.max_life;
            match &p.kind {
                ParticleKind::Blood => {
                    let alpha = frac * p.color.a;
                    draw_circle(sp.x, sp.y, p.radius, Color { a: alpha, ..p.color });
                }
                ParticleKind::Dust => {
                    draw_circle(sp.x, sp.y, p.radius * frac,
                                Color::new(0.5, 0.45, 0.38, frac * 0.4));
                }
                ParticleKind::NoiseRing { start_r } => {
                    let t = 1.0 - frac;
                    let r = start_r + (p.radius - start_r) * t;
                    draw_circle_lines(sp.x, sp.y, r, 1.5,
                        Color::new(1.0, 0.85, 0.4, frac * 0.55));
                }
            }
        }
    }
}
