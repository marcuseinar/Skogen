use macroquad::prelude::*;
use crate::camera::{Camera, TILE_SIZE, MAP_W, MAP_H};
use crate::buildings::{Building, BuildingKind};
use crate::entities::{Entity, EntityKind};
use crate::zombie::Zombie;

#[derive(Clone, Copy, PartialEq)]
pub enum TileKind {
    Grass,
    Forest,
    Road,
    Water,
    Sand,
    BuildingFloor,
    BuildingWall,
}

impl TileKind {
    pub fn is_solid(self) -> bool {
        matches!(self, TileKind::Forest | TileKind::Water | TileKind::BuildingWall)
    }

    pub fn color(self) -> Color {
        match self {
            TileKind::Grass       => Color::new(0.22, 0.48, 0.18, 1.0),
            TileKind::Forest      => Color::new(0.08, 0.28, 0.08, 1.0),
            TileKind::Road        => Color::new(0.38, 0.38, 0.36, 1.0),
            TileKind::Water       => Color::new(0.15, 0.38, 0.72, 1.0),
            TileKind::Sand        => Color::new(0.72, 0.66, 0.46, 1.0),
            TileKind::BuildingFloor => Color::new(0.72, 0.58, 0.38, 1.0),
            TileKind::BuildingWall  => Color::new(0.42, 0.28, 0.18, 1.0),
        }
    }
}

pub struct World {
    pub tiles: Vec<TileKind>,
    pub buildings: Vec<Building>,
    pub entities: Vec<Entity>,
    pub zombies: Vec<Zombie>,
}

impl World {
    fn idx(x: i32, y: i32) -> usize {
        (y as usize) * MAP_W + (x as usize)
    }

    pub fn get_tile(&self, x: i32, y: i32) -> TileKind {
        if x < 0 || y < 0 || x >= MAP_W as i32 || y >= MAP_H as i32 {
            return TileKind::Water;
        }
        self.tiles[Self::idx(x, y)]
    }

    pub fn set_tile(&mut self, x: i32, y: i32, k: TileKind) {
        if x < 0 || y < 0 || x >= MAP_W as i32 || y >= MAP_H as i32 { return; }
        self.tiles[Self::idx(x, y)] = k;
    }

    pub fn is_solid_at(&self, wx: f32, wy: f32) -> bool {
        let tx = (wx / TILE_SIZE) as i32;
        let ty = (wy / TILE_SIZE) as i32;
        if self.get_tile(tx, ty).is_solid() { return true; }
        // also check entity solidity
        for e in &self.entities {
            if e.is_solid() && (e.pos - vec2(wx, wy)).length() < 18.0 {
                return true;
            }
        }
        false
    }

    pub fn spawn_pos(&self) -> Vec2 {
        // Inside the starting stuga at tile (30,40) near the lake — door centre at tx=32,ty=43
        vec2(32.0 * TILE_SIZE + 16.0, 42.0 * TILE_SIZE + 8.0)
    }

    pub fn generate() -> Self {
        let mut world = World {
            tiles: vec![TileKind::Grass; MAP_W * MAP_H],
            buildings: Vec::new(),
            entities: Vec::new(),
            zombies: Vec::new(),
        };

        // Roads: horizontal at y=58-60, vertical at x=58-60
        let road_cx = (MAP_W / 2) as i32;
        let road_cy = (MAP_H / 2) as i32;

        for x in 0..MAP_W as i32 {
            for dy in -1..=1i32 {
                world.set_tile(x, road_cy + dy, TileKind::Road);
            }
        }
        for y in 0..MAP_H as i32 {
            for dx in -1..=1i32 {
                world.set_tile(road_cx + dx, y, TileKind::Road);
            }
        }

        // Branch roads (eastward from intersection, westward)
        for x in (road_cx - 25)..(road_cx - 8) {
            world.set_tile(x, road_cy - 15, TileKind::Road);
            world.set_tile(x, road_cy - 14, TileKind::Road);
        }
        for y in (road_cy - 15)..road_cy {
            world.set_tile(road_cx - 25, y, TileKind::Road);
            world.set_tile(road_cx - 24, y, TileKind::Road);
        }

        // Eastern branch
        for x in (road_cx + 8)..(road_cx + 28) {
            world.set_tile(x, road_cy + 15, TileKind::Road);
            world.set_tile(x, road_cy + 16, TileKind::Road);
        }
        for y in road_cy..(road_cy + 16) {
            world.set_tile(road_cx + 28, y, TileKind::Road);
            world.set_tile(road_cx + 29, y, TileKind::Road);
        }

        // Forest clusters
        let forest_centers: &[(i32, i32, i32)] = &[
            (15, 15, 12), (30, 10, 8), (80, 20, 10), (100, 15, 9),
            (10, 70, 11), (20, 85, 8), (90, 80, 13), (105, 90, 10),
            (40, 40, 7), (75, 45, 9), (55, 85, 8), (15, 45, 6),
            (95, 55, 7), (35, 100, 9), (85, 100, 8), (110, 55, 10),
        ];
        for &(fx, fy, r) in forest_centers {
            for dy in -r..=r {
                for dx in -r..=r {
                    if dx * dx + dy * dy <= r * r {
                        let tx = fx + dx;
                        let ty = fy + dy;
                        if world.get_tile(tx, ty) == TileKind::Grass {
                            world.set_tile(tx, ty, TileKind::Forest);
                        }
                    }
                }
            }
        }

        // Lakes
        let lakes: &[(i32, i32, i32)] = &[(22, 33, 12), (88, 72, 11)];
        for &(lx, ly, r) in lakes {
            for dy in -(r + 2)..=(r + 2) {
                for dx in -(r + 2)..=(r + 2) {
                    let d2 = dx * dx + dy * dy;
                    if d2 <= r * r {
                        world.set_tile(lx + dx, ly + dy, TileKind::Water);
                    } else if d2 <= (r + 2) * (r + 2) {
                        if world.get_tile(lx + dx, ly + dy) != TileKind::Water {
                            world.set_tile(lx + dx, ly + dy, TileKind::Sand);
                        }
                    }
                }
            }
        }

        // Place buildings
        let building_specs: &[(BuildingKind, i32, i32)] = &[
            (BuildingKind::Macken,  road_cx + 3, road_cy - 8),
            (BuildingKind::IcaNara, road_cx - 20, road_cy + 3),
            (BuildingKind::Stuga,   road_cx + 12, road_cy + 3),
            (BuildingKind::Stuga,   road_cx + 20, road_cy + 3),
            (BuildingKind::Stuga,   road_cx - 30, road_cy - 20),
            (BuildingKind::Forrad,  road_cx - 29, road_cy - 26),
            (BuildingKind::Stuga,   road_cx + 30, road_cy + 18),
            (BuildingKind::Forrad,  road_cx + 30, road_cy + 24),
            (BuildingKind::Camping, road_cx - 25, road_cy - 8),
            (BuildingKind::Stuga,   road_cx + 12, road_cy - 10),
        ];

        for &(kind, btx, bty) in building_specs {
            let b = Building::new(kind, btx, bty);
            world.place_building_tiles(&b);
            // Spawn 2-3 zombies near each building
            let zcount = if matches!(kind, BuildingKind::IcaNara) { 4 } else { 2 };
            for _ in 0..zcount {
                let zx = (btx as f32 + macroquad::rand::gen_range(-3.0f32, b.tw as f32 + 3.0)) * TILE_SIZE;
                let zy = (bty as f32 + macroquad::rand::gen_range(-3.0f32, b.th as f32 + 3.0)) * TILE_SIZE;
                world.zombies.push(Zombie::new(vec2(zx, zy)));
            }
            world.buildings.push(b);
        }

        // Scatter harvestable entities on grass
        for _ in 0..60 {
            let tx = macroquad::rand::gen_range(2.0f32, (MAP_W - 2) as f32) as i32;
            let ty = macroquad::rand::gen_range(2.0f32, (MAP_H - 2) as f32) as i32;
            if world.get_tile(tx, ty) == TileKind::Grass {
                let kind_r = macroquad::rand::gen_range(0u32, 10);
                let kind = if kind_r < 4 { EntityKind::Tree }
                           else if kind_r < 7 { EntityKind::Rock }
                           else { EntityKind::Bush };
                let pos = vec2(tx as f32 * TILE_SIZE + 16.0, ty as f32 * TILE_SIZE + 16.0);
                world.entities.push(Entity::new(kind, pos));
            }
        }

        // Extra wandering zombies on roads
        for _ in 0..12 {
            let tx = macroquad::rand::gen_range(5.0f32, (MAP_W - 5) as f32) as i32;
            let ty = macroquad::rand::gen_range(5.0f32, (MAP_H - 5) as f32) as i32;
            if !world.get_tile(tx, ty).is_solid() {
                let pos = vec2(tx as f32 * TILE_SIZE + 16.0, ty as f32 * TILE_SIZE + 16.0);
                world.zombies.push(Zombie::new(pos));
            }
        }

        world
    }

    fn place_building_tiles(&mut self, b: &Building) {
        for dy in 0..b.th {
            for dx in 0..b.tw {
                let tx = b.tx + dx;
                let ty = b.ty + dy;
                let on_perimeter = dx == 0 || dy == 0 || dx == b.tw - 1 || dy == b.th - 1;
                // Door gap: bottom-center, 2 tiles wide
                let door = dy == b.th - 1 && (dx == b.tw / 2 || dx == b.tw / 2 - 1);
                let kind = if on_perimeter && !door {
                    TileKind::BuildingWall
                } else {
                    TileKind::BuildingFloor
                };
                self.set_tile(tx, ty, kind);
            }
        }
    }

    pub fn update_zombies(&mut self, dt: f32, player_pos: Vec2) {
        let tiles = self.tiles.clone();
        let get_tile = |x: i32, y: i32| -> TileKind {
            if x < 0 || y < 0 || x >= MAP_W as i32 || y >= MAP_H as i32 {
                return TileKind::Water;
            }
            tiles[(y as usize) * MAP_W + (x as usize)]
        };

        for z in &mut self.zombies {
            let is_solid = |pos: Vec2| {
                let tx = (pos.x / TILE_SIZE) as i32;
                let ty = (pos.y / TILE_SIZE) as i32;
                get_tile(tx, ty).is_solid()
            };
            z.update(dt, player_pos, is_solid);
        }
    }

    pub fn try_attack(&mut self, pos: Vec2, facing: Vec2, damage: f32, range: f32) {
        for z in &mut self.zombies {
            if !z.alive { continue; }
            let d = z.pos - pos;
            if d.length() > range { continue; }
            // Check roughly in facing direction
            if facing.length() > 0.1 {
                let dot = d.normalize().dot(facing.normalize());
                if dot < 0.3 { continue; }
            }
            z.take_damage(damage);
        }
    }

    pub fn interact_near(&mut self, pos: Vec2, has_axe: bool) -> Vec<crate::loot::Item> {
        let mut items = Vec::new();
        const RANGE: f32 = 55.0;

        for b in &mut self.buildings {
            for c in &mut b.containers {
                if !c.looted && (c.pos - pos).length() < RANGE {
                    items.extend(c.items.drain(..));
                    c.looted = true;
                }
            }
        }

        for e in &mut self.entities {
            if !e.alive { continue; }
            if (e.pos - pos).length() > RANGE { continue; }
            let can_harvest = has_axe || matches!(e.kind, EntityKind::Bush);
            if !can_harvest { continue; }
            e.health -= 1;
            if e.health <= 0 {
                e.alive = false;
                items.extend(e.loot());
            } else {
                items.push(e.harvest_item());
            }
            break;
        }

        items
    }

    pub fn update_roofs(&mut self, player_pos: Vec2, dt: f32) {
        for b in &mut self.buildings {
            let inside = b.contains_player(player_pos, TILE_SIZE);
            let target = if inside { 0.0 } else { 1.0 };
            let speed  = if inside { 6.0 } else { 3.0 };
            b.roof_alpha += (target - b.roof_alpha) * dt * speed;
        }
    }

    pub fn draw_roofs(&self, cam: &Camera) {
        for b in &self.buildings {
            b.draw_roof(cam, TILE_SIZE);
        }
    }

    pub fn draw(&self, cam: &Camera) {
        let sw = screen_width();
        let sh = screen_height();

        let x0 = ((cam.offset.x / TILE_SIZE) as i32 - 1).max(0);
        let y0 = ((cam.offset.y / TILE_SIZE) as i32 - 1).max(0);
        let x1 = ((cam.offset.x + sw) / TILE_SIZE) as i32 + 2;
        let y1 = ((cam.offset.y + sh) / TILE_SIZE) as i32 + 2;
        let x1 = x1.min(MAP_W as i32);
        let y1 = y1.min(MAP_H as i32);

        for ty in y0..y1 {
            for tx in x0..x1 {
                let tile = self.get_tile(tx, ty);
                let sp = cam.world_to_screen(vec2(tx as f32 * TILE_SIZE, ty as f32 * TILE_SIZE));
                draw_rectangle(sp.x, sp.y, TILE_SIZE + 0.5, TILE_SIZE + 0.5, tile.color());

                // Road markings
                if tile == TileKind::Road {
                    if tx % 6 == 0 {
                        draw_rectangle(sp.x + TILE_SIZE * 0.45, sp.y, TILE_SIZE * 0.1, TILE_SIZE, Color::new(0.85, 0.82, 0.2, 0.5));
                    }
                }

                // Forest texture dots
                if tile == TileKind::Forest {
                    draw_circle(sp.x + 10.0, sp.y + 8.0, 5.0, Color::new(0.05, 0.22, 0.05, 1.0));
                    draw_circle(sp.x + 22.0, sp.y + 20.0, 4.0, Color::new(0.05, 0.22, 0.05, 1.0));
                }
            }
        }

        // Draw entities
        for e in &self.entities {
            e.draw(cam);
        }

        // Draw building signs and containers
        for b in &self.buildings {
            b.draw_sign(cam, TILE_SIZE);
            b.draw_containers(cam);
        }

        // Draw zombies
        for z in &self.zombies {
            z.draw(cam);
        }
    }
}

