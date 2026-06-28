use macroquad::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub enum Item {
    Food(u8),
    Water(u8),
    Medicine,
    Wood,
    Stone,
    Axe,
    Crowbar,
    Pistol,
    Ammo(u8),
}

impl Item {
    pub fn name(&self) -> &str {
        match self {
            Item::Food(_) => "Mat",
            Item::Water(_) => "Vatten",
            Item::Medicine => "Medicin",
            Item::Wood => "Trä",
            Item::Stone => "Sten",
            Item::Axe => "Yxa",
            Item::Crowbar => "Kofot",
            Item::Pistol => "Pistol",
            Item::Ammo(_) => "Ammo",
        }
    }

    pub fn color(&self) -> Color {
        match self {
            Item::Food(_) => ORANGE,
            Item::Water(_) => SKYBLUE,
            Item::Medicine => PINK,
            Item::Wood => Color::new(0.5, 0.3, 0.1, 1.0),
            Item::Stone => GRAY,
            Item::Axe | Item::Crowbar => DARKGRAY,
            Item::Pistol => Color::new(0.2, 0.2, 0.2, 1.0),
            Item::Ammo(_) => YELLOW,
        }
    }

    pub fn is_weapon(&self) -> bool {
        matches!(self, Item::Axe | Item::Crowbar | Item::Pistol)
    }

    pub fn is_ranged(&self) -> bool {
        matches!(self, Item::Pistol)
    }

    pub fn damage(&self) -> f32 {
        match self {
            Item::Axe => 34.0,
            Item::Crowbar => 26.0,
            Item::Pistol => 100.0,
            _ => 8.0,
        }
    }

    pub fn attack_range(&self) -> f32 {
        match self {
            Item::Pistol => 280.0,
            Item::Axe => 64.0,
            Item::Crowbar => 56.0,
            _ => 40.0,
        }
    }

    pub fn hunger_restore(&self) -> f32 {
        match self {
            Item::Food(n) => *n as f32,
            _ => 0.0,
        }
    }

    pub fn thirst_restore(&self) -> f32 {
        match self {
            Item::Water(n) => *n as f32,
            _ => 0.0,
        }
    }

    pub fn health_restore(&self) -> f32 {
        match self {
            Item::Medicine => 40.0,
            _ => 0.0,
        }
    }
}

pub struct LootContainer {
    pub pos: Vec2,
    pub items: Vec<Item>,
    pub looted: bool,
    pub label: &'static str,
}

impl LootContainer {
    pub fn new(pos: Vec2, items: Vec<Item>, label: &'static str) -> Self {
        Self { pos, items, looted: false, label }
    }
}
