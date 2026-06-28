use macroquad::prelude::*;
use std::collections::VecDeque;
use crate::camera::{Camera, TILE_SIZE, MAP_W, MAP_H};
use crate::sprites::Sprites;
use crate::loot::{Item, LootSpot};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum TileKind {
    Grass,
    Pavement,
    Road,
    WoodFloor,
    ConcreteFloor,
    ExtWall,
    IntWall,
    ClosedDoor,
    OpenDoor,
    Window,
    BrokenWindow,
    Rubble,
}

impl TileKind {
    pub fn is_solid(self) -> bool {
        matches!(self, TileKind::ExtWall | TileKind::IntWall |
                 TileKind::ClosedDoor | TileKind::Window)
    }
    pub fn is_slow(self) -> bool {
        matches!(self, TileKind::Rubble | TileKind::Grass)
    }
    pub fn sprite_col(self) -> f32 {
        match self {
            TileKind::Grass                           => 0.0,
            TileKind::Pavement                        => 1.0,
            TileKind::Road                            => 2.0,
            TileKind::WoodFloor                       => 3.0,
            TileKind::ConcreteFloor | TileKind::OpenDoor => 6.0,
            TileKind::ExtWall | TileKind::IntWall |
            TileKind::ClosedDoor | TileKind::Window |
            TileKind::BrokenWindow                    => 4.0,
            TileKind::Rubble                          => 5.0,
        }
    }
}

#[derive(Clone)]
pub struct MapTile {
    pub kind: TileKind,
    pub blood: f32,
}

impl MapTile {
    fn new(kind: TileKind) -> Self { Self { kind, blood: 0.0 } }
}

pub struct Map {
    pub tiles: Vec<MapTile>,
    pub loot: Vec<LootSpot>,
    pub escape_tile: (i32, i32),
}

// ── helpers ──────────────────────────────────────────────────────────────────

fn idx(x: i32, y: i32) -> usize { y as usize * MAP_W + x as usize }
fn in_bounds(x: i32, y: i32) -> bool {
    x >= 0 && y >= 0 && x < MAP_W as i32 && y < MAP_H as i32
}

fn set(tiles: &mut Vec<MapTile>, x: i32, y: i32, k: TileKind) {
    if in_bounds(x, y) { tiles[idx(x, y)] = MapTile::new(k); }
}

fn fill(tiles: &mut Vec<MapTile>, x0: i32, y0: i32, x1: i32, y1: i32, k: TileKind) {
    for y in y0..=y1 { for x in x0..=x1 { set(tiles, x, y, k); } }
}

fn place_room(
    tiles: &mut Vec<MapTile>,
    x0: i32, y0: i32, w: i32, h: i32,
    floor: TileKind, wall: TileKind,
) {
    fill(tiles, x0, y0, x0+w-1, y0+h-1, floor);
    for x in x0..x0+w { set(tiles, x, y0, wall); set(tiles, x, y0+h-1, wall); }
    for y in y0..y0+h { set(tiles, x0, y, wall); set(tiles, x0+w-1, y, wall); }
}

fn world(tx: i32, ty: i32) -> Vec2 {
    vec2(tx as f32 * TILE_SIZE + TILE_SIZE * 0.5, ty as f32 * TILE_SIZE + TILE_SIZE * 0.5)
}

// ── Map generation ─────────────────────────────────────────────────────────────
impl Map {
    pub fn generate() -> Self {
        let mut tiles = vec![MapTile::new(TileKind::Grass); MAP_W * MAP_H];
        let mut loot: Vec<LootSpot> = Vec::new();

        // ── Roads ─────────────────────────────────────────────────────────────
        // N-S road cols 16-17
        for y in 0..MAP_H as i32 {
            set(&mut tiles, 16, y, TileKind::Road);
            set(&mut tiles, 17, y, TileKind::Road);
        }
        // E-W road rows 16-17
        for x in 0..MAP_W as i32 {
            set(&mut tiles, x, 16, TileKind::Road);
            set(&mut tiles, x, 17, TileKind::Road);
        }
        // Pavement shoulders
        for y in 0..MAP_H as i32 {
            if tiles[idx(15, y)].kind == TileKind::Grass { set(&mut tiles, 15, y, TileKind::Pavement); }
            if tiles[idx(18, y)].kind == TileKind::Grass { set(&mut tiles, 18, y, TileKind::Pavement); }
        }
        for x in 0..MAP_W as i32 {
            if tiles[idx(x, 15)].kind == TileKind::Grass { set(&mut tiles, x, 15, TileKind::Pavement); }
            if tiles[idx(x, 18)].kind == TileKind::Grass { set(&mut tiles, x, 18, TileKind::Pavement); }
        }

        // Escape point — south end of N-S road (row 34)
        let escape_tile = (16, 34);

        // ── SAFE HOUSE (NW) — cols 1-12, rows 1-12 ────────────────────────────
        // Outer shell
        place_room(&mut tiles, 1, 1, 12, 12, TileKind::WoodFloor, TileKind::ExtWall);
        // Interior partition — row 7 dividing bedrooms from living area
        for x in 2..12 { set(&mut tiles, x, 7, TileKind::IntWall); }
        set(&mut tiles, 6, 7, TileKind::ClosedDoor);   // interior door
        // Doors and windows
        set(&mut tiles, 6, 12, TileKind::ClosedDoor);  // south exit
        set(&mut tiles, 5,  1, TileKind::Window);      // north window
        set(&mut tiles, 8,  1, TileKind::Window);      // north window
        set(&mut tiles, 1,  5, TileKind::Window);      // west window
        set(&mut tiles, 12, 9, TileKind::Window);      // east window
        // Rubble path from house to road
        for x in 13..15 { set(&mut tiles, x, 12, TileKind::Pavement); }
        for x in 13..15 { set(&mut tiles, x, 11, TileKind::Pavement); }

        // Safe house loot (limited supplies — you need to scavenge)
        loot.push(LootSpot::new(world(4, 4),
            vec![Item::Flashlight { battery: 55.0 }, Item::Bandage],
            "Soffbord"));
        loot.push(LootSpot::new(world(9, 3),
            vec![Item::Food(20), Item::Water(15)],
            "Kylskåp"));
        loot.push(LootSpot::new(world(3, 9),
            vec![Item::Knife, Item::Food(10)],
            "Kökslåda"));

        // ── KONSUM STORE (NE) — cols 20-33, rows 1-11 ─────────────────────────
        place_room(&mut tiles, 20, 1, 14, 11, TileKind::ConcreteFloor, TileKind::ExtWall);
        // Interior partition — storage in back
        for x in 21..33 { set(&mut tiles, x, 7, TileKind::IntWall); }
        set(&mut tiles, 26, 7, TileKind::ClosedDoor);
        // Doors/windows
        set(&mut tiles, 26, 11, TileKind::ClosedDoor); // south exit
        set(&mut tiles, 23,  1, TileKind::Window);
        set(&mut tiles, 29,  1, TileKind::Window);
        set(&mut tiles, 33,  5, TileKind::Window);
        // Counter (obstacle row)
        for x in 22..31 { set(&mut tiles, x, 4, TileKind::IntWall); }
        set(&mut tiles, 22, 4, TileKind::ConcreteFloor); // gap in counter
        // Pavement access
        for y in 12..15 { set(&mut tiles, 26, y, TileKind::Pavement); }

        loot.push(LootSpot::new(world(24, 3),
            vec![Item::Food(35), Item::Food(30), Item::Water(40)],
            "Hylla"));
        loot.push(LootSpot::new(world(29, 3),
            vec![Item::Water(30), Item::Medkit],
            "Kyldisk"));
        loot.push(LootSpot::new(world(25, 9),
            vec![Item::Food(20), Item::Bandage],
            "Förråd"));

        // ── GARAGE (SW) — cols 1-12, rows 20-32 ───────────────────────────────
        place_room(&mut tiles, 1, 20, 12, 13, TileKind::ConcreteFloor, TileKind::ExtWall);
        // Large garage opening south (2 tiles)
        set(&mut tiles, 5, 32, TileKind::ConcreteFloor);
        set(&mut tiles, 6, 32, TileKind::ConcreteFloor);
        set(&mut tiles, 7, 32, TileKind::ConcreteFloor);
        // Small side door
        set(&mut tiles, 12, 26, TileKind::ClosedDoor);
        // Windows
        set(&mut tiles, 5, 20, TileKind::Window);
        set(&mut tiles, 1, 25, TileKind::Window);
        // Interior shelves (obstacles)
        for y in 21..27 { set(&mut tiles, 10, y, TileKind::IntWall); }

        loot.push(LootSpot::new(world(4, 23),
            vec![Item::TireIron, Item::Crowbar],
            "Verktygshylla"));
        loot.push(LootSpot::new(world(7, 28),
            vec![Item::Ammo(12), Item::Bandage],
            "Verktygslåda"));

        // ── GAS STATION (SE) — cols 20-33, rows 20-33 ─────────────────────────
        // Canopy area (just pavement outside)
        fill(&mut tiles, 20, 20, 33, 33, TileKind::Pavement);
        // Office/shop building inside
        place_room(&mut tiles, 25, 20, 9, 8, TileKind::ConcreteFloor, TileKind::ExtWall);
        set(&mut tiles, 29, 27, TileKind::ClosedDoor); // door south
        set(&mut tiles, 33, 24, TileKind::ClosedDoor); // door east
        set(&mut tiles, 26, 20, TileKind::Window);
        set(&mut tiles, 30, 20, TileKind::Window);
        // Pump islands (rubble as placeholder)
        fill(&mut tiles, 21, 30, 22, 31, TileKind::Rubble);
        fill(&mut tiles, 21, 24, 22, 25, TileKind::Rubble);

        // THE KEY LOOT — in a back office corner
        loot.push(LootSpot::new(world(27, 22),
            vec![Item::CarKeys],
            "Nyckelskåp"));
        loot.push(LootSpot::new(world(31, 23),
            vec![Item::Pistol, Item::Ammo(9)],
            "Kassaskåp"));
        loot.push(LootSpot::new(world(26, 25),
            vec![Item::Food(15), Item::Water(20)],
            "Disk"));

        // ── Fence lines (between safe house and road) ──────────────────────────
        // Small fences on grass to channel movement
        for y in 13..15 { set(&mut tiles, 0, y, TileKind::ExtWall); }
        for y in 19..21 { set(&mut tiles, 0, y, TileKind::ExtWall); }

        // ── Rubble / debris scatter ────────────────────────────────────────────
        // Crashed car rubble near intersection
        fill(&mut tiles, 14, 14, 14, 15, TileKind::Rubble);
        fill(&mut tiles, 14, 18, 14, 19, TileKind::Rubble);

        Self { tiles, loot, escape_tile }
    }

    fn tile(&self, x: i32, y: i32) -> TileKind {
        if !in_bounds(x, y) { return TileKind::ExtWall; }
        self.tiles[idx(x, y)].kind
    }

    pub fn is_solid(&self, x: i32, y: i32) -> bool { self.tile(x, y).is_solid() }

    pub fn is_solid_at_world(&self, wx: f32, wy: f32) -> bool {
        let tx = (wx / TILE_SIZE) as i32;
        let ty = (wy / TILE_SIZE) as i32;
        self.is_solid(tx, ty)
    }

    pub fn tile_at_world(&self, wx: f32, wy: f32) -> TileKind {
        self.tile((wx / TILE_SIZE) as i32, (wy / TILE_SIZE) as i32)
    }

    pub fn open_door(&mut self, x: i32, y: i32) {
        if in_bounds(x, y) && self.tiles[idx(x,y)].kind == TileKind::ClosedDoor {
            self.tiles[idx(x,y)].kind = TileKind::OpenDoor;
        }
    }

    pub fn break_window(&mut self, x: i32, y: i32) {
        if in_bounds(x, y) && self.tiles[idx(x,y)].kind == TileKind::Window {
            self.tiles[idx(x,y)].kind = TileKind::BrokenWindow;
        }
    }

    pub fn splat_blood(&mut self, wx: f32, wy: f32) {
        let tx = (wx / TILE_SIZE) as i32;
        let ty = (wy / TILE_SIZE) as i32;
        if in_bounds(tx, ty) {
            self.tiles[idx(tx,ty)].blood = (self.tiles[idx(tx,ty)].blood + 0.45).min(1.0);
        }
    }

    // BFS path on tile grid, returns world-space waypoints
    pub fn find_path(&self, start: Vec2, end: Vec2) -> Vec<Vec2> {
        let sx = (start.x / TILE_SIZE) as i32;
        let sy = (start.y / TILE_SIZE) as i32;
        let ex = (end.x / TILE_SIZE) as i32;
        let ey = (end.y / TILE_SIZE) as i32;
        if sx == ex && sy == ey { return vec![end]; }
        if !in_bounds(ex, ey) || self.is_solid(ex, ey) { return vec![]; }

        let cap = MAP_W * MAP_H;
        let mut parent = vec![(i16::MIN, i16::MIN); cap];
        let mut visited = vec![false; cap];
        let mut queue = VecDeque::new();
        let start_idx = idx(sx, sy);
        visited[start_idx] = true;
        queue.push_back((sx, sy));

        let dirs: &[(i32,i32)] = &[(0,1),(1,0),(0,-1),(-1,0),(1,1),(1,-1),(-1,1),(-1,-1)];
        let mut found = false;

        'outer: while let Some((cx, cy)) = queue.pop_front() {
            for &(dx, dy) in dirs {
                let nx = cx + dx;
                let ny = cy + dy;
                if !in_bounds(nx, ny) { continue; }
                let ni = idx(nx, ny);
                if visited[ni] { continue; }
                if self.is_solid(nx, ny) { continue; }
                if dx != 0 && dy != 0 && self.is_solid(cx+dx, cy) && self.is_solid(cx, cy+dy) { continue; }
                visited[ni] = true;
                parent[ni] = (cx as i16, cy as i16);
                if nx == ex && ny == ey { found = true; break 'outer; }
                queue.push_back((nx, ny));
            }
        }

        if !found { return vec![end]; }  // best effort: move toward destination

        let mut path = Vec::new();
        let (mut px, mut py) = (ex, ey);
        while !(px == sx && py == sy) {
            path.push(vec2(px as f32 * TILE_SIZE + TILE_SIZE * 0.5,
                           py as f32 * TILE_SIZE + TILE_SIZE * 0.5));
            let (ppx, ppy) = parent[idx(px, py)];
            if ppx == i16::MIN { break; }
            px = ppx as i32; py = ppy as i32;
        }
        path.reverse();
        path
    }

    // Draw tiles in diagonal order (correct iso painter's algorithm)
    pub fn draw_tiles(&self, cam: &Camera, sprites: &Sprites) {
        let sw = screen_width();
        let sh = screen_height();
        let mg = 96.0;
        let corners = [
            cam.screen_to_world(vec2(-mg, -mg)),
            cam.screen_to_world(vec2(sw+mg, -mg)),
            cam.screen_to_world(vec2(-mg, sh+mg)),
            cam.screen_to_world(vec2(sw+mg, sh+mg)),
        ];
        let min_tx = corners.iter().map(|c| (c.x/TILE_SIZE).floor() as i32 - 1)
            .min().unwrap_or(0).max(0);
        let max_tx = corners.iter().map(|c| (c.x/TILE_SIZE).ceil() as i32 + 1)
            .max().unwrap_or(0).min(MAP_W as i32 - 1);
        let min_ty = corners.iter().map(|c| (c.y/TILE_SIZE).floor() as i32 - 1)
            .min().unwrap_or(0).max(0);
        let max_ty = corners.iter().map(|c| (c.y/TILE_SIZE).ceil() as i32 + 1)
            .max().unwrap_or(0).min(MAP_H as i32 - 1);

        for sum in (min_tx+min_ty)..=(max_tx+max_ty) {
            for tx in (sum-max_ty).max(min_tx)..=(sum-min_ty).min(max_tx) {
                let ty = sum - tx;
                if ty < 0 || ty >= MAP_H as i32 { continue; }
                let tile = &self.tiles[idx(tx, ty)];
                let sp = cam.world_to_screen(vec2(tx as f32 * TILE_SIZE, ty as f32 * TILE_SIZE));
                sprites.draw_tile(tile.kind, sp);
                // Blood overlay
                if tile.blood > 0.0 {
                    let alpha = (tile.blood * 0.75).min(0.7);
                    sprites.draw_blood(sp, alpha);
                }
            }
        }

        // Draw escape marker
        let (ex, ey) = self.escape_tile;
        let esp = cam.world_to_screen(vec2(ex as f32 * TILE_SIZE, ey as f32 * TILE_SIZE));
        if cam.is_visible(vec2(ex as f32 * TILE_SIZE, ey as f32 * TILE_SIZE), 48.0) {
            // Subtle pulsing amber arrow
            let t = get_time() as f32;
            let pulse = 0.5 + 0.5 * (t * 2.0).sin();
            draw_triangle(
                vec2(esp.x, esp.y - 8.0),
                vec2(esp.x - 10.0, esp.y + 4.0),
                vec2(esp.x + 10.0, esp.y + 4.0),
                Color::new(0.85, 0.65, 0.1, 0.35 + 0.3 * pulse),
            );
        }

        // Draw loot spots
        for ls in &self.loot {
            ls.draw(cam);
        }
    }
}
