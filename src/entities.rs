use macroquad::prelude::*;
use crate::camera::Camera;
use crate::loot::Item;

#[derive(Clone, Copy, PartialEq)]
pub enum EntityKind {
    Tree,
    Rock,
    Bush,
}

pub struct Entity {
    pub kind: EntityKind,
    pub pos: Vec2,
    pub health: i32,
    pub alive: bool,
}

impl Entity {
    pub fn new(kind: EntityKind, pos: Vec2) -> Self {
        let health = match kind {
            EntityKind::Tree => 3,
            EntityKind::Rock => 4,
            EntityKind::Bush => 1,
        };
        Self { kind, pos, health, alive: true }
    }

    pub fn loot(&self) -> Vec<Item> {
        match self.kind {
            EntityKind::Tree => vec![Item::Wood, Item::Wood],
            EntityKind::Rock => vec![Item::Stone, Item::Stone],
            EntityKind::Bush => vec![Item::Food(20)],
        }
    }

    pub fn harvest_item(&self) -> Item {
        match self.kind {
            EntityKind::Tree => Item::Wood,
            EntityKind::Rock => Item::Stone,
            EntityKind::Bush => Item::Food(10),
        }
    }

    pub fn is_solid(&self) -> bool {
        self.alive && !matches!(self.kind, EntityKind::Bush)
    }

    pub fn draw(&self, cam: &Camera) {
        if !self.alive { return; }
        if !cam.is_visible(self.pos, 40.0) { return; }
        let sp = cam.world_to_screen(self.pos);
        match self.kind {
            EntityKind::Tree => {
                draw_rectangle(sp.x - 5.0, sp.y + 4.0, 10.0, 12.0, Color::new(0.38, 0.22, 0.08, 1.0));
                draw_circle(sp.x, sp.y - 2.0, 18.0, Color::new(0.1, 0.42, 0.1, 1.0));
                draw_circle(sp.x - 6.0, sp.y + 2.0, 12.0, Color::new(0.08, 0.38, 0.08, 1.0));
                draw_circle(sp.x + 7.0, sp.y + 1.0, 10.0, Color::new(0.12, 0.45, 0.12, 1.0));
            }
            EntityKind::Rock => {
                draw_circle(sp.x, sp.y, 13.0, Color::new(0.52, 0.52, 0.52, 1.0));
                draw_circle(sp.x + 5.0, sp.y + 4.0, 9.0, Color::new(0.44, 0.44, 0.44, 1.0));
                draw_circle(sp.x - 4.0, sp.y + 3.0, 7.0, Color::new(0.48, 0.48, 0.48, 1.0));
            }
            EntityKind::Bush => {
                draw_circle(sp.x, sp.y, 11.0, Color::new(0.18, 0.52, 0.18, 1.0));
                draw_circle(sp.x - 6.0, sp.y + 3.0, 8.0, Color::new(0.14, 0.46, 0.14, 1.0));
                draw_circle(sp.x + 4.0, sp.y - 2.0, 3.5, RED);
                draw_circle(sp.x - 2.0, sp.y + 6.0, 2.5, RED);
            }
        }
    }
}
