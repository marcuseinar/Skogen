use macroquad::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub enum Item {
    TireIron,
    Flashlight { battery: f32 },
    Food(u32),
    Water(u32),
    Bandage,
    Medkit,
    CarKeys,
    Pistol,
    Ammo(u32),
    Crowbar,
    Knife,
}

impl Item {
    pub fn name(&self) -> &str {
        match self {
            Item::TireIron             => "Kofot",
            Item::Flashlight { .. }    => "Ficklampa",
            Item::Food(_)              => "Mat",
            Item::Water(_)             => "Vatten",
            Item::Bandage              => "Bandage",
            Item::Medkit               => "Sjuklåda",
            Item::CarKeys              => "Bilnycklar",
            Item::Pistol               => "Pistol",
            Item::Ammo(_)              => "Ammunition",
            Item::Crowbar              => "Kofot",
            Item::Knife                => "Kniv",
        }
    }

    pub fn sprite_col(&self) -> f32 {
        match self {
            Item::TireIron | Item::Crowbar | Item::Knife => 0.0,
            Item::Flashlight { .. }                      => 1.0,
            Item::Food(_)                                => 2.0,
            Item::Water(_)                               => 3.0,
            Item::Bandage                                => 4.0,
            Item::Medkit                                 => 5.0,
            Item::CarKeys                                => 6.0,
            Item::Pistol | Item::Ammo(_)                 => 7.0,
        }
    }

    pub fn melee_damage(&self) -> f32 {
        match self {
            Item::TireIron | Item::Crowbar => 45.0,
            Item::Knife                    => 35.0,
            _                              => 15.0,
        }
    }

    pub fn melee_range(&self) -> f32 {
        match self {
            Item::TireIron | Item::Crowbar => 52.0,
            Item::Knife                    => 42.0,
            _                              => 38.0,
        }
    }

    pub fn food_restore(&self) -> u32 { match self { Item::Food(v) => *v, _ => 0 } }
    pub fn water_restore(&self) -> u32 { match self { Item::Water(v) => *v, _ => 0 } }
    pub fn health_restore(&self) -> f32 {
        match self { Item::Bandage => 20.0, Item::Medkit => 60.0, _ => 0.0 }
    }
    pub fn is_consumable(&self) -> bool {
        matches!(self, Item::Food(_) | Item::Water(_) | Item::Bandage | Item::Medkit)
    }
    pub fn is_melee(&self) -> bool {
        matches!(self, Item::TireIron | Item::Crowbar | Item::Knife)
    }
}

pub struct LootSpot {
    pub world_pos: Vec2,
    pub items: Vec<Item>,
    pub looted: bool,
    pub label: &'static str,
}

impl LootSpot {
    pub fn new(world_pos: Vec2, items: Vec<Item>, label: &'static str) -> Self {
        Self { world_pos, items, looted: false, label }
    }

    pub fn draw(&self, cam: &crate::camera::Camera) {
        if self.looted { return; }
        if !cam.is_visible(self.world_pos, 40.0) { return; }
        let sp = cam.world_to_screen(self.world_pos);
        draw_rectangle(sp.x - 8.0, sp.y - 9.0, 16.0, 10.0,
                       Color::new(0.50, 0.34, 0.06, 1.0));
        draw_rectangle_lines(sp.x - 8.0, sp.y - 9.0, 16.0, 10.0, 1.5,
                       Color::new(0.80, 0.58, 0.10, 0.95));
        draw_rectangle(sp.x - 8.0, sp.y - 11.5, 16.0, 3.5,
                       Color::new(0.38, 0.24, 0.04, 1.0));
        draw_line(sp.x - 3.0, sp.y - 10.0, sp.x + 1.0, sp.y - 8.0,
                  1.0, Color::new(0.9, 0.78, 0.35, 0.65));
    }
}
