mod camera;
mod loot;
mod map;
mod noise;
mod walker;
mod particles;
mod player;
mod ui;
mod lighting;
mod sprites;

use macroquad::prelude::*;
use camera::{Camera, TILE_SIZE};
use map::Map;
use noise::NoiseSystem;
use walker::Walker;
use particles::Particles;
use player::Player;
use lighting::LightSystem;
use sprites::Sprites;

#[derive(PartialEq)]
enum GameState { Playing, Dead, Escaped }

fn spawn_walkers() -> Vec<Walker> {
    let p = |tx: i32, ty: i32| {
        vec2(tx as f32 * TILE_SIZE + TILE_SIZE * 0.5,
             ty as f32 * TILE_SIZE + TILE_SIZE * 0.5)
    };
    [
        p(19,  8), p(27,  6), p(31,  9), p(23, 13),
        p( 7, 18), p(25, 18), p(14, 20), p( 3, 24),
        p(10, 28), p(22, 26), p(29, 29), p(18, 32),
        p(12, 34), p( 5, 35),
    ].iter().map(|&pos| Walker::new(pos)).collect()
}

fn new_game() -> (Map, Vec<Walker>, Player, Camera, NoiseSystem, Particles) {
    let map = Map::generate();
    let spawn = vec2(6.5 * TILE_SIZE, 5.5 * TILE_SIZE);
    let player = Player::new(spawn);
    let walkers = spawn_walkers();
    let mut cam = Camera::new();
    cam.update(spawn, 1.0); // snap camera to player
    (map, walkers, player, cam, NoiseSystem::new(), Particles::new())
}

#[macroquad::main("Skogen")]
async fn main() {
    macroquad::rand::srand((get_time() * 1_000_000.0) as u64 ^ 0xdeadbeef);

    let sprites = Sprites::load().await;
    let light_system = LightSystem::new();

    let (mut map, mut walkers, mut player, mut camera,
         mut noise, mut particles) = new_game();
    let mut state = GameState::Playing;
    let mut time_alive = 0.0f32;

    let num_keys = [
        KeyCode::Key1, KeyCode::Key2, KeyCode::Key3, KeyCode::Key4,
        KeyCode::Key5, KeyCode::Key6, KeyCode::Key7, KeyCode::Key8,
    ];

    loop {
        let dt = get_frame_time().min(0.05);
        let time = get_time() as f32;
        let sw = screen_width();
        let sh = screen_height();

        match state {
            GameState::Playing => {
                // ── Input ───────────────────────────────────────────────────────

                // Inventory slot selection
                for (i, &key) in num_keys.iter().enumerate() {
                    if is_key_pressed(key) && i < player.inventory.len() {
                        player.active_slot = i;
                    }
                }
                let (_, wheel_y) = mouse_wheel();
                if wheel_y > 0.1 && player.active_slot > 0 {
                    player.active_slot -= 1;
                }
                if wheel_y < -0.1 && !player.inventory.is_empty() {
                    player.active_slot = (player.active_slot + 1)
                        .min(player.inventory.len().saturating_sub(1));
                }

                // Touch button input (read before drawing)
                let touch = ui::read_touch_input(sw, sh);

                // Actions
                if touch.interact || is_key_pressed(KeyCode::E) {
                    player.interact(&mut map, &mut noise);
                }
                if touch.use_item || is_key_pressed(KeyCode::Q) {
                    player.use_active_item();
                }
                if touch.toggle_fl || is_key_pressed(KeyCode::F) {
                    player.flashlight_on = !player.flashlight_on;
                }

                // Mouse click-to-move
                if is_mouse_button_pressed(MouseButton::Left) {
                    let mp = vec2(mouse_position().0, mouse_position().1);
                    let on_inv   = mp.y > sh - 52.0;
                    let on_panel = mp.y < 80.0 && mp.x < 185.0;
                    if !on_inv && !on_panel {
                        let wp    = camera.screen_to_world(mp);
                        let shift = is_key_down(KeyCode::LeftShift)
                                 || is_key_down(KeyCode::RightShift);
                        player.set_destination(wp, &map, shift);
                    }
                }

                // Touch tap-to-move (left/center of screen)
                for t in touches() {
                    if t.phase == TouchPhase::Started {
                        let p = t.position;
                        let on_btn = p.x > sw - 90.0 && p.y > sh * 0.40;
                        let on_inv = p.y > sh - 52.0;
                        if !on_btn && !on_inv {
                            player.set_destination(
                                camera.screen_to_world(p), &map, false);
                        }
                    }
                }

                // ── Update ──────────────────────────────────────────────────────
                noise.update(dt);
                player.update(dt, &mut map, &mut noise, &mut walkers, &mut particles);
                for w in &mut walkers {
                    w.update(dt, player.pos, player.flashlight_on, &map, &mut noise);
                }
                // Gradually despawn dead walkers
                walkers.retain(|w| w.alive || macroquad::rand::gen_range(0.0f32, 1.0) > 0.997);
                particles.update(dt);
                camera.update(player.pos, dt);

                if !player.alive {
                    state = GameState::Dead;
                }

                // Escape win condition
                let esc_world = vec2(
                    map.escape_tile.0 as f32 * TILE_SIZE + TILE_SIZE * 0.5,
                    map.escape_tile.1 as f32 * TILE_SIZE + TILE_SIZE * 0.5,
                );
                if player.has_keys && (player.pos - esc_world).length() < TILE_SIZE * 1.5 {
                    state = GameState::Escaped;
                }

                time_alive += dt;

                // ── Draw ────────────────────────────────────────────────────────
                clear_background(Color::new(0.10, 0.18, 0.08, 1.0));

                map.draw_tiles(&camera, &sprites);

                // Depth-sorted walkers + player (iso painter's order)
                let pd = player.pos.x + player.pos.y;
                let mut wo: Vec<usize> = (0..walkers.len())
                    .filter(|&i| walkers[i].alive)
                    .collect();
                wo.sort_unstable_by(|&a, &b| {
                    let da = walkers[a].pos.x + walkers[a].pos.y;
                    let db = walkers[b].pos.x + walkers[b].pos.y;
                    da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
                });
                let pi = wo.partition_point(|&i| {
                    walkers[i].pos.x + walkers[i].pos.y < pd
                });
                for i in 0..=wo.len() {
                    if i == pi { player.draw(&camera, &sprites); }
                    if i < wo.len() { walkers[wo[i]].draw(&camera, &sprites); }
                }

                particles.draw(&camera);

                // Lighting overlay (directional flashlight cone)
                let player_sp = camera.world_to_screen(player.pos);
                let fx = player.facing.x;
                let fy = player.facing.y;
                // Project world-space facing into iso screen space, compute angle
                let player_angle = ((fx + fy) * 0.5_f32).atan2(fx - fy);

                let day_frac = (time_alive % 240.0) / 240.0;
                let ambient = if day_frac < 0.5 {
                    0.85
                } else if day_frac < 0.75 {
                    0.85 - 0.80 * ((day_frac - 0.5) / 0.25)
                } else {
                    0.05 + 0.80 * ((day_frac - 0.75) / 0.25)
                };

                light_system.draw(
                    player_sp,
                    player_angle,
                    player.flashlight_on,
                    player.flashlight_battery / 100.0,
                    ambient,
                    time,
                );

                // HUD + touch buttons
                ui::draw_hud(&player, &sprites, time_alive);
                ui::draw_touch_buttons(sw, sh);

                // Contextual hints
                let near_loot = map.loot.iter().any(|ls| {
                    !ls.looted && (ls.world_pos - player.pos).length() < 52.0
                });
                if near_loot {
                    draw_text("Tryck E för att plocka",
                              sw * 0.5 - 105.0, sh * 0.5 - 60.0, 20.0,
                              Color::new(1.0, 1.0, 0.6, 0.9));
                }

                let near_esc = (player.pos - esc_world).length() < TILE_SIZE * 3.0;
                if near_esc && !player.has_keys {
                    draw_text("Hitta bilnycklarna vid bensinstationen!",
                              sw * 0.5 - 195.0, sh * 0.5 - 60.0, 19.0,
                              Color::new(1.0, 0.7, 0.2, 0.9));
                }
            }

            GameState::Dead => {
                clear_background(Color::new(0.06, 0.0, 0.0, 1.0));
                draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.35, 0.0, 0.0, 0.3));

                let mins = (time_alive / 60.0) as u32;
                let secs = (time_alive % 60.0) as u32;
                draw_text("DU DOG",
                          sw * 0.5 - 90.0, sh * 0.5 - 50.0, 72.0, RED);
                draw_text(&format!("Du överlevde {:02}:{:02}", mins, secs),
                          sw * 0.5 - 120.0, sh * 0.5 + 12.0, 28.0, WHITE);
                draw_text("Tryck R eller peka för att börja om",
                          sw * 0.5 - 165.0, sh * 0.5 + 52.0, 21.0, LIGHTGRAY);

                let restart = is_key_pressed(KeyCode::R)
                    || touches().iter().any(|t| t.phase == TouchPhase::Started);
                if restart {
                    macroquad::rand::srand((get_time() * 1_000_000.0) as u64 ^ 0xcafe);
                    let (m, w, p, c, n, par) = new_game();
                    map = m; walkers = w; player = p; camera = c;
                    noise = n; particles = par;
                    state = GameState::Playing;
                    time_alive = 0.0;
                }
            }

            GameState::Escaped => {
                clear_background(Color::new(0.03, 0.08, 0.02, 1.0));
                draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.0, 0.15, 0.05, 0.4));

                let mins = (time_alive / 60.0) as u32;
                let secs = (time_alive % 60.0) as u32;
                draw_text("DU KLARADE DET!",
                          sw * 0.5 - 180.0, sh * 0.5 - 50.0, 62.0,
                          Color::new(0.3, 0.9, 0.3, 1.0));
                draw_text(&format!("Du tog dig ut på {:02}:{:02}", mins, secs),
                          sw * 0.5 - 135.0, sh * 0.5 + 15.0, 26.0, WHITE);
                draw_text("Tryck R eller peka för att spela igen",
                          sw * 0.5 - 175.0, sh * 0.5 + 55.0, 21.0, LIGHTGRAY);

                let restart = is_key_pressed(KeyCode::R)
                    || touches().iter().any(|t| t.phase == TouchPhase::Started);
                if restart {
                    macroquad::rand::srand((get_time() * 1_000_000.0) as u64 ^ 0xbabe);
                    let (m, w, p, c, n, par) = new_game();
                    map = m; walkers = w; player = p; camera = c;
                    noise = n; particles = par;
                    state = GameState::Playing;
                    time_alive = 0.0;
                }
            }
        }

        next_frame().await;
    }
}
