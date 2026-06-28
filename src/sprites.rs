use macroquad::prelude::*;
use crate::map::TileKind;

pub struct Sprites { texture: Texture2D }

impl Sprites {
    pub async fn load() -> Self {
        let tex = load_texture("sprites.png").await.expect("sprites.png not found");
        tex.set_filter(FilterMode::Nearest);
        Self { texture: tex }
    }

    fn blit(&self, src_x: f32, src_y: f32, src_w: f32, src_h: f32,
             dst_x: f32, dst_y: f32, dst_w: f32, dst_h: f32,
             tint: Color, flip_x: bool) {
        draw_texture_ex(&self.texture, dst_x, dst_y, tint, DrawTextureParams {
            dest_size: Some(vec2(dst_w, dst_h)),
            source:    Some(Rect::new(src_x, src_y, src_w, src_h)),
            flip_x,
            ..Default::default()
        });
    }

    // ── Tiles (row 0, 64×64 cells). sp = iso top-vertex of tile. ─────────────
    pub fn draw_tile(&self, kind: TileKind, sp: Vec2) {
        let col = kind.sprite_col();
        self.blit(col*64.0, 0.0, 64.0, 64.0, sp.x-32.0, sp.y, 64.0, 64.0, WHITE, false);
    }

    pub fn draw_blood(&self, sp: Vec2, alpha: f32) {
        // Blood splatter from row 3 (effects), col 0
        let tint = Color::new(1.0, 1.0, 1.0, alpha);
        self.blit(0.0, 160.0, 32.0, 32.0, sp.x-16.0, sp.y+4.0, 32.0, 16.0, tint, false);
    }

    // ── Characters (row 1, y=64, 32×64 cells, cols 0-7). ─────────────────────
    // sp = iso ground point of character
    pub fn draw_character(&self, col: usize, sp: Vec2, tint: Color, flip_x: bool) {
        self.blit(col as f32 * 32.0, 64.0, 32.0, 64.0,
                  sp.x - 16.0, sp.y - 59.0, 32.0, 64.0, tint, flip_x);
    }

    // ── Items (row 2, y=128, 32×32 cells). Drawn in world space. ─────────────
    pub fn draw_item(&self, col: f32, sp: Vec2, tint: Color) {
        let scale = 0.8;
        let dw = 32.0 * scale; let dh = 32.0 * scale;
        self.blit(col * 32.0, 128.0, 32.0, 32.0,
                  sp.x - dw*0.5, sp.y - dh, dw, dh, tint, false);
    }

    // Item in HUD slot
    pub fn draw_item_hud(&self, col: f32, x: f32, y: f32, size: f32) {
        self.blit(col * 32.0, 128.0, 32.0, 32.0, x, y, size, size, WHITE, false);
    }
}
