use macroquad::prelude::*;
use crate::camera::Camera;
use crate::loot::Item;
use crate::sprites::Sprites;

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

    pub fn draw(&self, cam: &Camera, sprites: &Sprites) {
        if !self.alive { return; }
        if !cam.is_visible(self.pos, 40.0) { return; }
        let sp = cam.world_to_screen(self.pos);
        sprites.draw_entity(self.kind, sp);
    }
}
