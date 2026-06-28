use macroquad::prelude::*;

pub const TILE_SIZE: f32 = 32.0;
pub const ISO_W: f32 = 64.0;
pub const ISO_H: f32 = 32.0;
pub const MAP_W: usize = 36;
pub const MAP_H: usize = 36;

pub struct Camera {
    pub offset: Vec2,
}

impl Camera {
    pub fn new() -> Self { Self { offset: Vec2::ZERO } }

    pub fn update(&mut self, player_pos: Vec2, dt: f32) {
        let tx = player_pos.x / TILE_SIZE;
        let ty = player_pos.y / TILE_SIZE;
        let iso_x = (tx - ty) * (ISO_W * 0.5);
        let iso_y = (tx + ty) * (ISO_H * 0.5);
        let target = vec2(iso_x - screen_width() * 0.5, iso_y - screen_height() * 0.5);
        // Exponential smoothing for cinematic camera follow
        let factor = 1.0 - (-10.0_f32 * dt).exp();
        self.offset = self.offset + (target - self.offset) * factor.min(1.0);
    }

    pub fn world_to_screen(&self, pos: Vec2) -> Vec2 {
        let tx = pos.x / TILE_SIZE;
        let ty = pos.y / TILE_SIZE;
        vec2((tx - ty) * (ISO_W * 0.5), (tx + ty) * (ISO_H * 0.5)) - self.offset
    }

    pub fn screen_to_world(&self, pos: Vec2) -> Vec2 {
        let iso = pos + self.offset;
        let hw = ISO_W * 0.5;
        let hh = ISO_H * 0.5;
        let tx = (iso.x / hw + iso.y / hh) * 0.5;
        let ty = (iso.y / hh - iso.x / hw) * 0.5;
        vec2(tx * TILE_SIZE, ty * TILE_SIZE)
    }

    pub fn is_visible(&self, pos: Vec2, margin: f32) -> bool {
        let sw = screen_width();
        let sh = screen_height();
        let sp = self.world_to_screen(pos);
        sp.x > -margin && sp.y > -margin && sp.x < sw + margin && sp.y < sh + margin
    }
}
