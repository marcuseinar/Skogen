use macroquad::prelude::*;
use crate::camera::Camera;
use crate::loot::{Item, LootContainer};

#[derive(Clone, Copy, PartialEq)]
pub enum BuildingKind {
    Stuga,
    Macken,
    IcaNara,
    Camping,
    Forrad,
}

pub struct Building {
    pub kind: BuildingKind,
    pub tx: i32,
    pub ty: i32,
    pub tw: i32,
    pub th: i32,
    pub containers: Vec<LootContainer>,
    pub label: &'static str,
}

impl Building {
    pub fn new(kind: BuildingKind, tx: i32, ty: i32) -> Self {
        let (tw, th, label, containers) = match kind {
            BuildingKind::Stuga => (5, 4, "Stuga", stuga_loot(tx, ty)),
            BuildingKind::Macken => (6, 5, "MACKEN", macken_loot(tx, ty)),
            BuildingKind::IcaNara => (8, 6, "ICA Nära", ica_loot(tx, ty)),
            BuildingKind::Camping => (7, 7, "Camping", camping_loot(tx, ty)),
            BuildingKind::Forrad => (3, 3, "Förråd", forrad_loot(tx, ty)),
        };
        Self { kind, tx, ty, tw, th, containers, label }
    }

    pub fn draw_sign(&self, cam: &Camera, ts: f32) {
        let wx = self.tx as f32 * ts;
        let wy = self.ty as f32 * ts;
        let sp = cam.world_to_screen(vec2(wx + self.tw as f32 * ts / 2.0, wy - 6.0));
        if !cam.is_visible(vec2(wx, wy), 200.0) { return; }

        let sign_color = match self.kind {
            BuildingKind::IcaNara => Color::new(0.9, 0.1, 0.1, 1.0),
            BuildingKind::Macken => Color::new(0.9, 0.7, 0.0, 1.0),
            BuildingKind::Camping => Color::new(0.1, 0.6, 0.1, 1.0),
            _ => Color::new(0.6, 0.45, 0.2, 1.0),
        };

        let font_size = 14.0;
        let label_w = self.label.len() as f32 * 7.5;
        draw_rectangle(sp.x - label_w / 2.0 - 4.0, sp.y - font_size, label_w + 8.0, font_size + 4.0, sign_color);
        draw_text(self.label, sp.x - label_w / 2.0, sp.y - 2.0, font_size, WHITE);
    }

    pub fn draw_containers(&self, cam: &Camera) {
        for c in &self.containers {
            if c.looted { continue; }
            if !cam.is_visible(c.pos, 20.0) { continue; }
            let sp = cam.world_to_screen(c.pos);
            draw_rectangle(sp.x - 7.0, sp.y - 7.0, 14.0, 14.0, Color::new(0.55, 0.35, 0.1, 1.0));
            draw_rectangle_lines(sp.x - 7.0, sp.y - 7.0, 14.0, 14.0, 2.0, Color::new(0.3, 0.15, 0.0, 1.0));
            // small E prompt if nearby would be shown by UI
        }
    }
}

fn stuga_loot(tx: i32, ty: i32) -> Vec<LootContainer> {
    let ts = crate::camera::TILE_SIZE;
    let cx = (tx as f32 + 1.5) * ts;
    let cy = (ty as f32 + 1.5) * ts;
    vec![
        LootContainer::new(vec2(cx, cy), vec![
            Item::Food(40), Item::Food(30), Item::Water(30),
            if macroquad::rand::gen_range(0u32, 3) == 0 { Item::Axe } else { Item::Food(20) },
        ], "Skåp"),
        LootContainer::new(vec2(cx + ts, cy), vec![
            Item::Medicine, Item::Water(50),
        ], "Byrå"),
    ]
}

fn macken_loot(tx: i32, ty: i32) -> Vec<LootContainer> {
    let ts = crate::camera::TILE_SIZE;
    let cx = (tx as f32 + 1.5) * ts;
    let cy = (ty as f32 + 1.5) * ts;
    vec![
        LootContainer::new(vec2(cx, cy), vec![
            Item::Food(25), Item::Food(25), Item::Water(40), Item::Crowbar,
        ], "Hylla"),
        LootContainer::new(vec2(cx + ts * 2.0, cy), vec![
            Item::Food(20), Item::Ammo(12),
        ], "Kassalåda"),
    ]
}

fn ica_loot(tx: i32, ty: i32) -> Vec<LootContainer> {
    let ts = crate::camera::TILE_SIZE;
    let cx = (tx as f32 + 2.0) * ts;
    let cy = (ty as f32 + 2.0) * ts;
    vec![
        LootContainer::new(vec2(cx, cy), vec![
            Item::Food(50), Item::Food(50), Item::Food(40), Item::Water(60),
        ], "Hylla 1"),
        LootContainer::new(vec2(cx + ts * 2.0, cy), vec![
            Item::Water(60), Item::Water(40), Item::Food(30),
        ], "Hylla 2"),
        LootContainer::new(vec2(cx, cy + ts), vec![
            Item::Medicine, Item::Medicine, Item::Ammo(6),
        ], "Apotek"),
        LootContainer::new(vec2(cx + ts * 2.0, cy + ts), vec![
            Item::Pistol, Item::Ammo(18),
        ], "Kassaskåp"),
    ]
}

fn camping_loot(tx: i32, ty: i32) -> Vec<LootContainer> {
    let ts = crate::camera::TILE_SIZE;
    let cx = (tx as f32 + 1.0) * ts;
    let cy = (ty as f32 + 1.5) * ts;
    vec![
        LootContainer::new(vec2(cx, cy), vec![
            Item::Food(35), Item::Water(50), Item::Axe,
        ], "Tält 1"),
        LootContainer::new(vec2(cx + ts * 2.0, cy), vec![
            Item::Food(30), Item::Water(45), Item::Medicine,
        ], "Tält 2"),
        LootContainer::new(vec2(cx + ts, cy + ts * 2.0), vec![
            Item::Water(60), Item::Food(20),
        ], "Tält 3"),
    ]
}

fn forrad_loot(tx: i32, ty: i32) -> Vec<LootContainer> {
    let ts = crate::camera::TILE_SIZE;
    let cx = (tx as f32 + 1.0) * ts;
    let cy = (ty as f32 + 1.0) * ts;
    vec![
        LootContainer::new(vec2(cx, cy), vec![
            Item::Axe, Item::Wood, Item::Stone,
        ], "Förråd"),
    ]
}
