use macroquad::prelude::*;
use crate::camera::Camera;
use crate::loot::Item;
use crate::world::World;

pub const INVENTORY_SLOTS: usize = 8;
const PLAYER_SPEED: f32 = 130.0;
const PLAYER_RADIUS: f32 = 12.0;

pub struct Inventory {
    pub slots: [Option<Item>; INVENTORY_SLOTS],
    pub active: usize,
}

impl Inventory {
    pub fn new() -> Self {
        Self { slots: Default::default(), active: 0 }
    }

    pub fn add(&mut self, item: Item) -> bool {
        // Stack ammo/food/water
        for slot in self.slots.iter_mut() {
            if let Some(existing) = slot {
                match (existing, &item) {
                    (Item::Food(a), Item::Food(b)) => { *a = a.saturating_add(*b); return true; }
                    (Item::Water(a), Item::Water(b)) => { *a = a.saturating_add(*b); return true; }
                    (Item::Ammo(a), Item::Ammo(b)) => { *a = a.saturating_add(*b); return true; }
                    _ => {}
                }
            }
        }
        for slot in self.slots.iter_mut() {
            if slot.is_none() {
                *slot = Some(item);
                return true;
            }
        }
        false
    }

    pub fn active_item(&self) -> Option<&Item> {
        self.slots[self.active].as_ref()
    }
}

pub struct InputState {
    pub move_dir: Vec2,
    pub attack: bool,
    pub interact: bool,
    pub use_item: bool,
}

pub struct Player {
    pub pos: Vec2,
    pub health: f32,
    pub hunger: f32,
    pub thirst: f32,
    pub inventory: Inventory,
    pub facing: Vec2,
    pub alive: bool,
    pub damage_cooldown: f32,
    pub attack_cooldown: f32,
    pub attack_anim: f32,
}

impl Player {
    pub fn new(pos: Vec2) -> Self {
        Self {
            pos,
            health: 100.0,
            hunger: 100.0,
            thirst: 100.0,
            inventory: Inventory::new(),
            facing: vec2(0.0, 1.0),
            alive: true,
            damage_cooldown: 0.0,
            attack_cooldown: 0.0,
            attack_anim: 0.0,
        }
    }

    pub fn take_damage(&mut self, dmg: f32) {
        if self.damage_cooldown > 0.0 { return; }
        self.health -= dmg;
        self.damage_cooldown = 1.0;
        if self.health <= 0.0 {
            self.health = 0.0;
            self.alive = false;
        }
    }

    pub fn update(&mut self, dt: f32, world: &mut World, input: &InputState) {
        if !self.alive { return; }

        // Cooldowns
        self.damage_cooldown = (self.damage_cooldown - dt).max(0.0);
        self.attack_cooldown = (self.attack_cooldown - dt).max(0.0);
        self.attack_anim = (self.attack_anim - dt * 4.0).max(0.0);

        // Movement
        let dir = input.move_dir;
        if dir.length() > 0.01 {
            self.facing = dir.normalize();
            let speed = PLAYER_SPEED * dt;

            let next_x = self.pos + vec2(dir.x * speed, 0.0);
            if !world.is_solid_at(next_x.x - PLAYER_RADIUS, next_x.y)
                && !world.is_solid_at(next_x.x + PLAYER_RADIUS, next_x.y)
                && !world.is_solid_at(next_x.x, next_x.y - PLAYER_RADIUS)
                && !world.is_solid_at(next_x.x, next_x.y + PLAYER_RADIUS)
            {
                self.pos.x = next_x.x;
            }

            let next_y = self.pos + vec2(0.0, dir.y * speed);
            if !world.is_solid_at(next_y.x - PLAYER_RADIUS, next_y.y)
                && !world.is_solid_at(next_y.x + PLAYER_RADIUS, next_y.y)
                && !world.is_solid_at(next_y.x, next_y.y - PLAYER_RADIUS)
                && !world.is_solid_at(next_y.x, next_y.y + PLAYER_RADIUS)
            {
                self.pos.y = next_y.y;
            }
        }

        // Survival drain
        self.thirst -= dt / 6.0;
        self.hunger -= dt / 10.0;
        self.thirst = self.thirst.max(0.0);
        self.hunger = self.hunger.max(0.0);

        if self.thirst <= 0.0 { self.take_damage(dt * 4.0); }
        if self.hunger <= 0.0 { self.take_damage(dt * 2.0); }

        // Use item (consume)
        if input.use_item {
            let active = self.inventory.active;
            if let Some(item) = &self.inventory.slots[active].clone() {
                let hr = item.hunger_restore();
                let tr = item.thirst_restore();
                let heal = item.health_restore();
                if hr > 0.0 || tr > 0.0 || heal > 0.0 {
                    self.hunger = (self.hunger + hr).min(100.0);
                    self.thirst = (self.thirst + tr).min(100.0);
                    self.health = (self.health + heal).min(100.0);
                    self.inventory.slots[active] = None;
                }
            }
        }

        // Attack
        if input.attack && self.attack_cooldown <= 0.0 {
            let (dmg, range) = self.inventory.active_item().map(|i| (i.damage(), i.attack_range())).unwrap_or((8.0, 40.0));
            world.try_attack(self.pos, self.facing, dmg, range);
            self.attack_cooldown = 0.4;
            self.attack_anim = 1.0;
        }

        // Interact (loot / harvest)
        if input.interact {
            let has_axe = self.inventory.slots.iter().any(|s| matches!(s, Some(Item::Axe)));
            let items = world.interact_near(self.pos, has_axe);
            for item in items {
                self.inventory.add(item);
            }
        }

        // Check zombie contact
        for z in &world.zombies {
            if z.alive && (z.pos - self.pos).length() < 22.0 {
                self.take_damage(10.0);
                break;
            }
        }
    }

    pub fn draw(&self, cam: &Camera) {
        let sp = cam.world_to_screen(self.pos);

        // Shadow
        draw_ellipse(sp.x, sp.y + 10.0, 12.0, 5.0, 0.0, Color::new(0.0, 0.0, 0.0, 0.25));

        // Body
        let hurt = self.damage_cooldown > 0.0;
        let body_col = if hurt { Color::new(1.0, 0.3, 0.3, 1.0) } else { Color::new(0.85, 0.78, 0.6, 1.0) };
        draw_circle(sp.x, sp.y, 12.0, body_col);

        // Shirt (blue-ish)
        draw_rectangle(sp.x - 8.0, sp.y, 16.0, 10.0, Color::new(0.3, 0.45, 0.7, 1.0));

        // Head
        draw_circle(sp.x, sp.y - 10.0, 9.0, Color::new(0.88, 0.72, 0.55, 1.0));
        // Eyes in facing direction
        let eye_off = self.facing * 4.0;
        draw_circle(sp.x + eye_off.x + self.facing.y * 3.0, sp.y + eye_off.y - self.facing.x * 3.0 - 10.0, 2.0, Color::new(0.1, 0.1, 0.1, 1.0));

        // Weapon swing indicator
        if self.attack_anim > 0.0 {
            let swing_pos = sp + self.facing * 30.0 * self.attack_anim;
            draw_circle(swing_pos.x, swing_pos.y, 8.0, Color::new(1.0, 0.8, 0.0, self.attack_anim * 0.7));
        }
    }

    pub fn active_weapon_name(&self) -> &str {
        self.inventory.active_item().map(|i| i.name()).unwrap_or("Händer")
    }
}
