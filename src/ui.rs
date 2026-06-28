use macroquad::prelude::*;
use crate::player::{Player, InputState, INVENTORY_SLOTS};
use crate::loot::Item;

pub struct TouchControls {
    pub enabled: bool,
    joystick_anchor: Option<Vec2>,
    joystick_current: Option<Vec2>,
    joy_touch_id: Option<u64>,
    pub atk_pressed: bool,
    pub interact_pressed: bool,
    pub use_pressed: bool,
}

impl TouchControls {
    pub fn new() -> Self {
        Self {
            enabled: false,
            joystick_anchor: None,
            joystick_current: None,
            joy_touch_id: None,
            atk_pressed: false,
            interact_pressed: false,
            use_pressed: false,
        }
    }

    pub fn gather_input(&mut self, cam: &crate::camera::Camera) -> InputState {
        // Keyboard
        let mut kx = 0.0f32;
        let mut ky = 0.0f32;
        if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) { ky -= 1.0; }
        if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down) { ky += 1.0; }
        if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) { kx -= 1.0; }
        if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) { kx += 1.0; }

        let kb_attack = is_mouse_button_pressed(MouseButton::Left) || is_key_pressed(KeyCode::Space);
        let kb_interact = is_key_pressed(KeyCode::E);
        let kb_use = is_key_pressed(KeyCode::F);

        // Touch
        let sw = screen_width();
        let sh = screen_height();
        let left_boundary = sw * 0.45;

        let atk_rect = Rect::new(sw - 100.0, sh * 0.5, 90.0, 70.0);
        let interact_rect = Rect::new(sw - 100.0, sh * 0.5 + 80.0, 90.0, 70.0);
        let use_rect = Rect::new(sw - 100.0, sh * 0.5 + 160.0, 90.0, 70.0);

        let touches = touches();
        if !touches.is_empty() {
            self.enabled = true;
        }

        self.atk_pressed = false;
        self.interact_pressed = false;
        self.use_pressed = false;

        // Clear joy if its touch ended
        if let Some(jid) = self.joy_touch_id {
            let still_active = touches.iter().any(|t| {
                t.id == jid && t.phase != TouchPhase::Ended && t.phase != TouchPhase::Cancelled
            });
            if !still_active {
                self.joystick_anchor = None;
                self.joystick_current = None;
                self.joy_touch_id = None;
            }
        }

        for touch in &touches {
            let tp = touch.position;
            match touch.phase {
                TouchPhase::Started => {
                    if tp.x < left_boundary {
                        if self.joy_touch_id.is_none() {
                            self.joystick_anchor = Some(tp);
                            self.joystick_current = Some(tp);
                            self.joy_touch_id = Some(touch.id);
                        }
                    } else {
                        if atk_rect.contains(tp) { self.atk_pressed = true; }
                        if interact_rect.contains(tp) { self.interact_pressed = true; }
                        if use_rect.contains(tp) { self.use_pressed = true; }
                    }
                }
                TouchPhase::Moved | TouchPhase::Stationary => {
                    if Some(touch.id) == self.joy_touch_id {
                        self.joystick_current = Some(tp);
                    } else if tp.x >= left_boundary {
                        if atk_rect.contains(tp) { self.atk_pressed = true; }
                        if interact_rect.contains(tp) { self.interact_pressed = true; }
                        if use_rect.contains(tp) { self.use_pressed = true; }
                    }
                }
                _ => {}
            }
        }

        // Compute joystick direction
        let mut touch_dir = Vec2::ZERO;
        if let (Some(anchor), Some(current)) = (self.joystick_anchor, self.joystick_current) {
            let delta = current - anchor;
            const DEAD_ZONE: f32 = 10.0;
            if delta.length() > DEAD_ZONE {
                touch_dir = delta.normalize();
            }
        }

        // Facing from mouse (desktop)
        let _ = cam; // used by callers for screen_to_world if needed

        let move_x = if kx != 0.0 { kx } else { touch_dir.x };
        let move_y = if ky != 0.0 { ky } else { touch_dir.y };
        let raw_dir = vec2(move_x, move_y);
        let move_dir = if raw_dir.length() > 1.0 { raw_dir.normalize() } else { raw_dir };

        InputState {
            move_dir,
            attack: kb_attack || self.atk_pressed,
            interact: kb_interact || self.interact_pressed,
            use_item: kb_use || self.use_pressed,
        }
    }

    pub fn draw(&self) {
        if !self.enabled { return; }

        let sw = screen_width();
        let sh = screen_height();

        // Joystick
        if let Some(anchor) = self.joystick_anchor {
            draw_circle_lines(anchor.x, anchor.y, 55.0, 2.0, Color::new(1.0, 1.0, 1.0, 0.3));
            let inner = self.joystick_current.unwrap_or(anchor);
            let raw = inner - anchor;
            let delta = if raw.length() > 55.0 { raw.normalize() * 55.0 } else { raw };
            draw_circle(anchor.x + delta.x, anchor.y + delta.y, 22.0, Color::new(1.0, 1.0, 1.0, 0.4));
        }

        // Action buttons
        let btn = |x: f32, y: f32, w: f32, h: f32, label: &str, col: Color| {
            draw_rectangle(x, y, w, h, Color::new(col.r, col.g, col.b, 0.35));
            draw_rectangle_lines(x, y, w, h, 2.0, Color::new(col.r, col.g, col.b, 0.7));
            draw_text(label, x + w / 2.0 - label.len() as f32 * 5.0, y + h / 2.0 + 6.0, 20.0, WHITE);
        };

        btn(sw - 100.0, sh * 0.5,        90.0, 70.0, "ATK",  RED);
        btn(sw - 100.0, sh * 0.5 + 80.0, 90.0, 70.0, "E",    YELLOW);
        btn(sw - 100.0, sh * 0.5 + 160.0, 90.0, 70.0, "F",   ORANGE);
    }

    pub fn any_touch_new(&self) -> bool {
        touches().iter().any(|t| t.phase == TouchPhase::Started)
    }
}

pub fn draw_hud(player: &Player, time_alive: f32) {
    let sw = screen_width();
    let sh = screen_height();
    let bar_w = sw.min(320.0) * 0.28;
    let bar_h = 14.0;
    let x0 = 10.0;

    // Background panel
    draw_rectangle(x0 - 4.0, 8.0, bar_w + 80.0, 60.0, Color::new(0.0, 0.0, 0.0, 0.45));

    draw_stat_bar(x0, 14.0, bar_w, bar_h, player.health / 100.0, RED, "Hälsa");
    draw_stat_bar(x0, 30.0, bar_w, bar_h, player.hunger / 100.0, ORANGE, "Mat");
    draw_stat_bar(x0, 46.0, bar_w, bar_h, player.thirst / 100.0, SKYBLUE, "Vatten");

    // Timer top-right
    let mins = (time_alive / 60.0) as u32;
    let secs = (time_alive % 60.0) as u32;
    let time_str = format!("{:02}:{:02}", mins, secs);
    draw_text(&time_str, sw - 70.0, 28.0, 26.0, WHITE);

    // Inventory bar at bottom
    let slot_size = (sw / INVENTORY_SLOTS as f32).min(60.0);
    let inv_w = slot_size * INVENTORY_SLOTS as f32;
    let inv_x = (sw - inv_w) / 2.0;
    let inv_y = sh - slot_size - 8.0;

    for i in 0..INVENTORY_SLOTS {
        let sx = inv_x + i as f32 * slot_size;
        let active = i == player.inventory.active;
        let bg = if active { Color::new(0.6, 0.6, 0.1, 0.8) } else { Color::new(0.1, 0.1, 0.1, 0.7) };
        draw_rectangle(sx, inv_y, slot_size - 2.0, slot_size - 2.0, bg);
        draw_rectangle_lines(sx, inv_y, slot_size - 2.0, slot_size - 2.0, 2.0,
            if active { YELLOW } else { DARKGRAY });

        if let Some(item) = &player.inventory.slots[i] {
            let ic = item.color();
            draw_rectangle(sx + 6.0, inv_y + 6.0, slot_size - 14.0, slot_size - 20.0, ic);
            let name = item.name();
            let fs = 11.0;
            draw_text(name, sx + 3.0, inv_y + slot_size - 8.0, fs, WHITE);

            // Show count for stackable items
            let count_str = match item {
                Item::Food(n) => format!("{}", n),
                Item::Water(n) => format!("{}", n),
                Item::Ammo(n) => format!("{}", n),
                _ => String::new(),
            };
            if !count_str.is_empty() {
                draw_text(&count_str, sx + slot_size - 18.0, inv_y + 18.0, 14.0, WHITE);
            }
        }

        // Slot number hint
        draw_text(&(i + 1).to_string(), sx + 3.0, inv_y + 14.0, 12.0, Color::new(0.6, 0.6, 0.6, 0.6));
    }

    // Active weapon name
    let weapon = player.active_weapon_name();
    draw_text(weapon, inv_x, inv_y - 6.0, 16.0, Color::new(0.9, 0.9, 0.9, 0.9));

    // Control hints (bottom-left, small)
    if !touches().is_empty() { return; } // skip on touch
    let hint_y = sh - 10.0;
    draw_text("WASD=rörelse  E=plocka  F=använd  Klick/Space=attack", 10.0, hint_y, 13.0, Color::new(0.6, 0.6, 0.6, 0.7));
}

fn draw_stat_bar(x: f32, y: f32, w: f32, h: f32, frac: f32, col: Color, label: &str) {
    draw_rectangle(x + 52.0, y, w, h, Color::new(0.15, 0.15, 0.15, 0.8));
    draw_rectangle(x + 52.0, y, w * frac.clamp(0.0, 1.0), h, col);
    draw_text(label, x, y + h - 2.0, 14.0, WHITE);
}
