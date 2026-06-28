use macroquad::prelude::*;
use crate::world::TileKind;
use crate::entities::EntityKind;

pub struct Sprites {
    texture: Texture2D,
}

impl Sprites {
    pub async fn load() -> Self {
        let texture = load_texture("sprites.png").await
            .expect("sprites.png not found");
        texture.set_filter(FilterMode::Nearest);
        Self { texture }
    }

    fn draw_sprite_raw(&self, src_x: f32, src_y: f32, src_w: f32, src_h: f32,
                       dst_x: f32, dst_y: f32, dst_w: f32, dst_h: f32,
                       tint: Color, flip_x: bool) {
        draw_texture_ex(
            &self.texture,
            dst_x, dst_y,
            tint,
            DrawTextureParams {
                dest_size: Some(vec2(dst_w, dst_h)),
                source: Some(Rect::new(src_x, src_y, src_w, src_h)),
                flip_x,
                ..Default::default()
            },
        );
    }

    // sp = iso screen position of the tile's NW corner (= top vertex of diamond)
    pub fn draw_tile(&self, tile: TileKind, sp: Vec2) {
        let col = match tile {
            TileKind::Grass         => 0,
            TileKind::Forest        => 1,
            TileKind::Road          => 2,
            TileKind::Water         => 3,
            TileKind::Sand          => 4,
            TileKind::BuildingFloor => 5,
            TileKind::BuildingWall  => 6,
        } as f32;
        // Row 0: 64×64 cells. Top vertex of diamond at sprite pixel (32, 0).
        self.draw_sprite_raw(col * 64.0, 0.0, 64.0, 64.0,
                             sp.x - 32.0, sp.y, 64.0, 64.0, WHITE, false);
    }

    // sp = iso ground point of the character
    pub fn draw_player(&self, sp: Vec2, facing: Vec2, hurt: bool, attack_anim: f32) {
        let tint = if hurt { Color::new(1.0, 0.35, 0.35, 1.0) } else { WHITE };
        let col = if facing.x.abs() >= facing.y.abs() {
            if facing.x < 0.0 { 2 } else { 3 }
        } else if facing.y > 0.0 {
            0
        } else {
            1
        };
        // Row 1: 32×64 cells at y=64. Shadow center at ~y=59 in sprite, center x=16.
        self.draw_sprite_raw(col as f32 * 32.0, 64.0, 32.0, 64.0,
                             sp.x - 16.0, sp.y - 59.0, 32.0, 64.0, tint, false);
        if attack_anim > 0.0 {
            let swing = sp + facing * 30.0 * attack_anim;
            draw_circle(swing.x, swing.y, 8.0, Color::new(1.0, 0.8, 0.0, attack_anim * 0.7));
        }
    }

    // sp = iso ground point of the zombie
    pub fn draw_zombie(&self, sp: Vec2, health: f32, _facing_left: bool) {
        // Zombie is col 4 in row 1 (y=64), 32×64 cell
        self.draw_sprite_raw(4.0 * 32.0, 64.0, 32.0, 64.0,
                             sp.x - 16.0, sp.y - 59.0, 32.0, 64.0, WHITE, false);
        if health < 100.0 {
            draw_rectangle(sp.x - 14.0, sp.y - 72.0, 28.0, 4.0,
                           Color::new(0.25, 0.0, 0.0, 0.85));
            draw_rectangle(sp.x - 14.0, sp.y - 72.0, 28.0 * (health / 100.0), 4.0, RED);
        }
    }

    // sp = iso ground point of the entity (center of its tile)
    pub fn draw_entity(&self, kind: EntityKind, sp: Vec2) {
        // Row 2: 64×96 cells at y=128
        let col = match kind {
            EntityKind::Tree => 0,
            EntityKind::Rock => 1,
            EntityKind::Bush => 2,
        } as f32;
        // base_y: how many pixels above sp the sprite's bottom edge sits
        let base_y = match kind {
            EntityKind::Tree => 10.0,  // ground shadow bottom at y≈92
            EntityKind::Rock => 10.0,
            EntityKind::Bush => 10.0,
        };
        self.draw_sprite_raw(col * 64.0, 128.0, 64.0, 96.0,
                             sp.x - 32.0, sp.y - 96.0 + base_y, 64.0, 96.0, WHITE, false);
    }
}
