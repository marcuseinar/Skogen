use macroquad::prelude::*;

pub const TILE_SIZE: f32 = 32.0;
pub const MAP_W: usize = 120;
pub const MAP_H: usize = 120;

pub struct Camera {
    pub offset: Vec2,
}

impl Camera {
    pub fn new() -> Self {
        Self { offset: Vec2::ZERO }
    }

    pub fn update(&mut self, player_pos: Vec2) {
        let sw = screen_width();
        let sh = screen_height();
        let world_w = MAP_W as f32 * TILE_SIZE;
        let world_h = MAP_H as f32 * TILE_SIZE;
        let mut off = player_pos - vec2(sw / 2.0, sh / 2.0);
        off.x = off.x.clamp(0.0, (world_w - sw).max(0.0));
        off.y = off.y.clamp(0.0, (world_h - sh).max(0.0));
        self.offset = off;
    }

    pub fn world_to_screen(&self, pos: Vec2) -> Vec2 {
        pos - self.offset
    }

    pub fn screen_to_world(&self, pos: Vec2) -> Vec2 {
        pos + self.offset
    }

    pub fn is_visible(&self, pos: Vec2, margin: f32) -> bool {
        let sw = screen_width();
        let sh = screen_height();
        let sp = self.world_to_screen(pos);
        sp.x > -margin && sp.y > -margin && sp.x < sw + margin && sp.y < sh + margin
    }
}
