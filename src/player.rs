use macroquad::prelude::*;
use crate::camera::{Camera, TILE_SIZE};
use crate::map::Map;
use crate::noise::NoiseSystem;
use crate::loot::Item;
use crate::sprites::Sprites;

const WALK_SPEED: f32 = 85.0;
const RUN_SPEED:  f32 = 155.0;
const PLAYER_R:   f32 = 10.0;

pub struct Player {
    pub pos:     Vec2,
    pub facing:  Vec2,
    pub health:  f32,
    pub hunger:  f32,
    pub alive:   bool,

    pub flashlight_on:      bool,
    pub flashlight_battery: f32,   // 0-100

    pub inventory:   Vec<Item>,
    pub active_slot: usize,

    pub has_keys:    bool,

    damage_cooldown: f32,
    attack_cooldown: f32,
    pub attack_anim: f32,

    // Click-to-move
    path:       Vec<Vec2>,
    running:    bool,

    // Noise emitted this frame
    pub current_noise: f32,
}

impl Player {
    pub fn new(pos: Vec2) -> Self {
        Self {
            pos,
            facing: vec2(0.0, 1.0),
            health: 100.0,
            hunger: 100.0,
            alive:  true,
            flashlight_on:      true,
            flashlight_battery: 55.0,
            inventory: vec![
                Item::Knife,
                Item::Bandage,
                Item::Food(20),
                Item::Water(15),
            ],
            active_slot: 0,
            has_keys: false,
            damage_cooldown: 0.0,
            attack_cooldown: 0.0,
            attack_anim:     0.0,
            path:    Vec::new(),
            running: false,
            current_noise: 0.0,
        }
    }

    pub fn active_item(&self) -> Option<&Item> {
        self.inventory.get(self.active_slot)
    }

    pub fn take_damage(&mut self, dmg: f32) {
        if self.damage_cooldown > 0.0 { return; }
        self.health = (self.health - dmg).max(0.0);
        self.damage_cooldown = 0.9;
        if self.health <= 0.0 { self.alive = false; }
    }

    // Called when a touch/click destination is set
    pub fn set_destination(&mut self, world_pos: Vec2, map: &Map, running: bool) {
        self.path = map.find_path(self.pos, world_pos);
        self.running = running;
    }

    pub fn stop(&mut self) { self.path.clear(); }

    pub fn update(
        &mut self,
        dt: f32,
        map: &mut Map,
        noise: &mut NoiseSystem,
        walkers: &mut Vec<crate::walker::Walker>,
        particles: &mut crate::particles::Particles,
    ) {
        if !self.alive { return; }

        self.damage_cooldown = (self.damage_cooldown - dt).max(0.0);
        self.attack_cooldown = (self.attack_cooldown - dt).max(0.0);
        self.attack_anim     = (self.attack_anim - dt * 3.5).max(0.0);
        self.current_noise   = 0.0;

        // ── Flashlight battery ───────────────────────────────────────────────
        if self.flashlight_on {
            self.flashlight_battery = (self.flashlight_battery - dt * 0.35).max(0.0);
            if self.flashlight_battery <= 0.0 { self.flashlight_on = false; }
        }

        // ── Hunger drain ─────────────────────────────────────────────────────
        self.hunger = (self.hunger - dt / 18.0).max(0.0);
        if self.hunger <= 0.0 { self.take_damage(dt * 2.5); }

        // ── WASD / keyboard movement ─────────────────────────────────────────
        let mut kb_dir = Vec2::ZERO;
        if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up)    { kb_dir.y -= 1.0; }
        if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down)   { kb_dir.y += 1.0; }
        if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left)   { kb_dir.x -= 1.0; }
        if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right)  { kb_dir.x += 1.0; }
        let kb_run = is_key_down(KeyCode::LeftShift);

        if kb_dir.length() > 0.01 {
            self.path.clear();  // keyboard overrides click-to-move
            self.running = kb_run;
            let dir = kb_dir.normalize();
            self.facing = dir;
            self.move_in_dir(dir, if kb_run { RUN_SPEED } else { WALK_SPEED }, dt, map);

            let is_slow_tile = map.tile_at_world(self.pos.x, self.pos.y).is_slow();
            let noise_r = if kb_run { TILE_SIZE * 5.0 } else if is_slow_tile { TILE_SIZE * 2.5 } else { TILE_SIZE * 1.5 };
            let noise_i = if kb_run { 0.85 } else { 0.35 };
            noise.emit(self.pos, noise_r, noise_i);
            self.current_noise = noise_i;
        } else if !self.path.is_empty() {
            // ── Click-to-move path following ─────────────────────────────────
            let target = self.path[0];
            let to_target = target - self.pos;
            let dist = to_target.length();

            if dist < 6.0 {
                self.path.remove(0);
                // Auto-open doors when adjacent
                let tx = (target.x / TILE_SIZE) as i32;
                let ty = (target.y / TILE_SIZE) as i32;
                map.open_door(tx, ty);
            } else {
                let dir = to_target / dist;
                self.facing = dir;
                let spd = if self.running { RUN_SPEED } else { WALK_SPEED };
                self.move_in_dir(dir, spd, dt, map);

                let noise_r = if self.running { TILE_SIZE * 5.0 } else { TILE_SIZE * 1.5 };
                let noise_i = if self.running { 0.85 } else { 0.35 };
                noise.emit(self.pos, noise_r, noise_i);
                self.current_noise = noise_i;
            }
        }

        // ── Auto-attack nearby walker ────────────────────────────────────────
        if self.attack_cooldown <= 0.0 {
            let (dmg, range) = self.active_item()
                .map(|i| (i.melee_damage(), i.melee_range()))
                .unwrap_or((15.0, 38.0));

            let mut best_dist = range;
            let mut best_idx  = None;
            for (i, w) in walkers.iter().enumerate() {
                if !w.alive { continue; }
                let d = (w.pos - self.pos).length();
                if d < best_dist { best_dist = d; best_idx = Some(i); }
            }
            if let Some(i) = best_idx {
                walkers[i].take_damage(dmg);
                map.splat_blood(walkers[i].pos.x, walkers[i].pos.y);
                particles.emit_blood(walkers[i].pos, 6);
                self.attack_cooldown = 0.55;
                self.attack_anim     = 1.0;
                noise.emit(self.pos, TILE_SIZE * 5.0, 0.9);
                particles.emit_noise_ring(self.pos, TILE_SIZE * 5.0);
                self.current_noise = 0.9;
            }
        }

        // ── Walker damage to player ──────────────────────────────────────────
        for w in walkers.iter() {
            if w.alive && (w.pos - self.pos).length() < TILE_SIZE * 1.1 {
                self.take_damage(18.0);
                break;
            }
        }

        // ── Check for car keys in inventory ──────────────────────────────────
        self.has_keys = self.inventory.iter().any(|i| matches!(i, Item::CarKeys));
    }

    fn move_in_dir(&mut self, dir: Vec2, speed: f32, dt: f32, map: &Map) {
        let delta = dir * speed * dt;
        let r = PLAYER_R;
        let try_pos = |dx: f32, dy: f32, p: Vec2, m: &Map| {
            let np = p + vec2(dx, dy);
            !m.is_solid_at_world(np.x - r, np.y)
                && !m.is_solid_at_world(np.x + r, np.y)
                && !m.is_solid_at_world(np.x, np.y - r)
                && !m.is_solid_at_world(np.x, np.y + r)
        };
        if try_pos(delta.x, delta.y, self.pos, map) {
            self.pos += delta;
        } else if try_pos(delta.x, 0.0, self.pos, map) {
            self.pos.x += delta.x;
        } else if try_pos(0.0, delta.y, self.pos, map) {
            self.pos.y += delta.y;
        }
    }

    pub fn interact(&mut self, map: &mut Map, noise: &mut NoiseSystem) {
        const LOOT_R: f32 = 52.0;
        for ls in &mut map.loot {
            if ls.looted { continue; }
            if (ls.world_pos - self.pos).length() > LOOT_R { continue; }
            for item in ls.items.drain(..) {
                self.inventory.push(item);
            }
            ls.looted = true;
        }
        // Open nearby doors
        let tx = (self.pos.x / TILE_SIZE) as i32;
        let ty = (self.pos.y / TILE_SIZE) as i32;
        for (dx, dy) in [(0,1),(0,-1),(1,0),(-1,0)] {
            map.open_door(tx+dx, ty+dy);
        }
    }

    pub fn use_active_item(&mut self) {
        if self.active_slot >= self.inventory.len() { return; }
        let item = &self.inventory[self.active_slot].clone();
        if !item.is_consumable() { return; }
        let hr = item.health_restore();
        let fr = item.food_restore();
        let wr = item.water_restore();
        self.health = (self.health + hr).min(100.0);
        self.hunger = (self.hunger + fr as f32 * 0.8).min(100.0);
        // water not tracked separately, just hunger offset
        let _ = wr;
        self.inventory.remove(self.active_slot);
        if self.active_slot >= self.inventory.len() && !self.inventory.is_empty() {
            self.active_slot = self.inventory.len() - 1;
        }
    }

    pub fn draw(&self, cam: &Camera, sprites: &Sprites) {
        if !self.alive { return; }
        let sp = cam.world_to_screen(self.pos);

        let col = if self.facing.y.abs() >= self.facing.x.abs() {
            if self.facing.y >= 0.0 { 0 } else { 1 }
        } else {
            if self.facing.x < 0.0 { 2 } else { 3 }
        };

        let hurt = self.damage_cooldown > 0.0;
        let tint = if hurt { Color::new(1.0, 0.25, 0.25, 1.0) } else { WHITE };
        sprites.draw_character(col, sp, tint, false);

        // Attack arc
        if self.attack_anim > 0.0 {
            let swing_pos = sp + self.facing * 36.0 * self.attack_anim;
            let r = 9.0 * self.attack_anim;
            draw_circle(swing_pos.x, swing_pos.y, r,
                        Color::new(0.9, 0.8, 0.2, self.attack_anim * 0.65));
            draw_circle_lines(swing_pos.x, swing_pos.y, r + 2.0, 1.0,
                        Color::new(1.0, 0.9, 0.4, self.attack_anim * 0.4));
        }
    }
}
