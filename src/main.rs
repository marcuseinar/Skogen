mod camera;
mod loot;
mod entities;
mod zombie;
mod buildings;
mod world;
mod player;
mod ui;
mod lighting;

use macroquad::prelude::*;
use camera::Camera;
use world::World;
use player::Player;
use ui::{draw_hud, TouchControls};
use lighting::LightSystem;

#[derive(PartialEq)]
enum GameState {
    Playing,
    Dead,
}

fn new_game() -> (World, Player, Camera) {
    let world = World::generate();
    let spawn = world.spawn_pos();
    let player = Player::new(spawn);
    let camera = Camera::new();
    (world, player, camera)
}

#[macroquad::main("Skogen")]
async fn main() {
    let seed = (get_time() * 1_000_000.0) as u64 ^ 0xdeadbeef;
    macroquad::rand::srand(seed);

    let (mut world, mut player, mut camera) = new_game();
    let mut state = GameState::Playing;
    let mut time_alive = 0.0f32;
    let mut touch_controls = TouchControls::new();
    let light_system = LightSystem::new();

    // Inventory selection state
    let num_keys = [
        KeyCode::Key1, KeyCode::Key2, KeyCode::Key3, KeyCode::Key4,
        KeyCode::Key5, KeyCode::Key6, KeyCode::Key7, KeyCode::Key8,
    ];

    loop {
        let dt = get_frame_time().min(0.05);

        match state {
            GameState::Playing => {
                // Inventory slot selection
                for (i, &key) in num_keys.iter().enumerate() {
                    if is_key_pressed(key) {
                        player.inventory.active = i;
                    }
                }

                // Mouse-aimed facing (desktop)
                let mp = mouse_position();
                let mw = camera.screen_to_world(vec2(mp.0, mp.1));
                let to_mouse = mw - player.pos;
                if to_mouse.length() > 5.0 {
                    player.facing = to_mouse.normalize();
                }

                world.update_zombies(dt, player.pos);
                world.update_roofs(player.pos, dt);

                let input = touch_controls.gather_input(&camera);
                player.update(dt, &mut world, &input);

                // Remove dead zombies (cleanup pass)
                world.zombies.retain(|z| z.alive || macroquad::rand::gen_range(0.0f32, 1.0f32) > 0.995);

                camera.update(player.pos);

                if !player.alive {
                    state = GameState::Dead;
                }

                time_alive += dt;

                // --- Draw ---
                clear_background(Color::new(0.2, 0.38, 0.18, 1.0));
                world.draw(&camera);
                player.draw(&camera);
                world.draw_roofs(&camera);

                // Ambient: bright day → dark night → dawn
                let day_len = 240.0f32;
                let t = time_alive % day_len;
                let frac = t / day_len;
                let ambient = if frac < 0.5 {
                    0.85
                } else if frac < 0.75 {
                    let p = (frac - 0.5) / 0.25;
                    0.85 + (0.05 - 0.85) * p
                } else {
                    let p = (frac - 0.75) / 0.25;
                    0.05 + (0.85 - 0.05) * p
                };

                // Aux lights: screen-space centres of first 4 buildings
                let aux: Vec<Vec2> = world.buildings.iter().take(4).map(|b| {
                    let wx = (b.tx as f32 + b.tw as f32 * 0.5) * camera::TILE_SIZE;
                    let wy = (b.ty as f32 + b.th as f32 * 0.5) * camera::TILE_SIZE;
                    camera.world_to_screen(vec2(wx, wy))
                }).collect();

                let player_sp = camera.world_to_screen(player.pos);
                light_system.draw(player_sp, &aux, ambient, time_alive);

                draw_hud(&player, time_alive);
                touch_controls.draw();

                // "E to loot" proximity hint
                let has_nearby = world.buildings.iter().any(|b| {
                    b.containers.iter().any(|c| !c.looted && (c.pos - player.pos).length() < 60.0)
                });
                if has_nearby {
                    let sw = screen_width();
                    let sh = screen_height();
                    draw_text("Tryck E för att plocka", sw / 2.0 - 100.0, sh / 2.0 - 60.0, 20.0,
                        Color::new(1.0, 1.0, 0.6, 0.9));
                }
            }

            GameState::Dead => {
                clear_background(Color::new(0.08, 0.0, 0.0, 1.0));
                let sw = screen_width();
                let sh = screen_height();
                let mins = (time_alive / 60.0) as u32;
                let secs = (time_alive % 60.0) as u32;

                // Red vignette
                draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.4, 0.0, 0.0, 0.3));

                draw_text("DU DOG", sw / 2.0 - 90.0, sh / 2.0 - 50.0, 68.0, RED);
                draw_text(
                    &format!("Du överlevde {:02}:{:02}", mins, secs),
                    sw / 2.0 - 115.0, sh / 2.0 + 10.0, 28.0, WHITE,
                );
                draw_text("Tryck R eller peka för att börja om",
                    sw / 2.0 - 160.0, sh / 2.0 + 50.0, 22.0, LIGHTGRAY);

                let restart = is_key_pressed(KeyCode::R) || touch_controls.any_touch_new();
                if restart {
                    macroquad::rand::srand((get_time() * 1_000_000.0) as u64 ^ 0xcafe);
                    let (w, p, c) = new_game();
                    world = w;
                    player = p;
                    camera = c;
                    state = GameState::Playing;
                    time_alive = 0.0;
                    touch_controls = TouchControls::new();
                }
            }
        }

        next_frame().await;
    }
}
