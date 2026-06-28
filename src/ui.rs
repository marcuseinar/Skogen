use macroquad::prelude::*;
use crate::player::Player;
use crate::loot::Item;
use crate::sprites::Sprites;

const BTN_W: f32 = 78.0;
const BTN_H: f32 = 58.0;
const BTN_GAP: f32 = 6.0;

fn btn_rects(sw: f32, sh: f32) -> [Rect; 3] {
    let bx = sw - BTN_W - 8.0;
    let by = sh * 0.42;
    [
        Rect::new(bx, by,                         BTN_W, BTN_H),
        Rect::new(bx, by + BTN_H + BTN_GAP,       BTN_W, BTN_H),
        Rect::new(bx, by + (BTN_H + BTN_GAP)*2.0, BTN_W, BTN_H),
    ]
}

pub struct TouchInput {
    pub interact:  bool,
    pub use_item:  bool,
    pub toggle_fl: bool,
    pub has_touch: bool,
}

/// Read touch button state (no drawing). Call at top of frame.
pub fn read_touch_input(sw: f32, sh: f32) -> TouchInput {
    let ts = touches();
    let has_touch = !ts.is_empty();
    if !has_touch {
        return TouchInput { interact: false, use_item: false, toggle_fl: false, has_touch: false };
    }
    let rects = btn_rects(sw, sh);
    let mut out = TouchInput { interact: false, use_item: false, toggle_fl: false, has_touch: true };
    for t in ts {
        if t.phase == TouchPhase::Started {
            let p = t.position;
            if rects[0].contains(p) { out.interact  = true; }
            if rects[1].contains(p) { out.use_item  = true; }
            if rects[2].contains(p) { out.toggle_fl = true; }
        }
    }
    out
}

/// Draw touch action buttons (visual only). Call during draw phase.
pub fn draw_touch_buttons(sw: f32, sh: f32) {
    if touches().is_empty() { return; }
    let r = btn_rects(sw, sh);
    touch_btn(r[0], "PLOCKA", Color::new(0.9, 0.85, 0.0, 1.0));
    touch_btn(r[1], "ANVÄND", Color::new(0.8, 0.45, 0.0, 1.0));
    touch_btn(r[2], "LAMPA",  Color::new(0.6, 0.60, 0.2, 1.0));
}

fn touch_btn(r: Rect, label: &str, col: Color) {
    draw_rectangle(r.x, r.y, r.w, r.h,
                   Color::new(col.r * 0.18, col.g * 0.18, col.b * 0.18, 0.38));
    draw_rectangle_lines(r.x, r.y, r.w, r.h, 2.0,
                         Color::new(col.r, col.g, col.b, 0.72));
    let lw = label.chars().count() as f32 * 5.5;
    draw_text(label, r.x + (r.w - lw) * 0.5, r.y + r.h * 0.55 + 5.0, 16.0, WHITE);
}

pub fn draw_hud(player: &Player, sprites: &Sprites, time_alive: f32) {
    let sw = screen_width();
    let sh = screen_height();

    // Status panel top-left
    draw_rectangle(6.0, 6.0, 170.0, 68.0, Color::new(0.0, 0.0, 0.0, 0.52));
    stat_bar(10.0, 12.0, 130.0, 12.0, player.health / 100.0,
             Color::new(0.78, 0.07, 0.07, 1.0), "HP ");
    stat_bar(10.0, 29.0, 130.0, 12.0, player.hunger / 100.0,
             Color::new(0.68, 0.42, 0.0,  1.0), "MAT");
    let batt_col = if player.flashlight_battery < 20.0 {
        Color::new(0.9, 0.15, 0.15, 1.0)
    } else {
        Color::new(0.82, 0.76, 0.15, 1.0)
    };
    let fl_lbl = if player.flashlight_on { "FL>" } else { "FL " };
    stat_bar(10.0, 46.0, 130.0, 12.0, player.flashlight_battery / 100.0,
             batt_col, fl_lbl);

    if player.has_keys {
        draw_text("[BILNYCKEL]", 10.0, 74.0, 14.0,
                  Color::new(0.88, 0.78, 0.15, 1.0));
    }

    // Noise flash (top-center)
    if player.current_noise > 0.2 {
        let msg = if player.current_noise > 0.8 { "!! BULLER !!" } else { "BULLER" };
        let alpha = (player.current_noise * 1.3).min(1.0);
        draw_text(msg, sw * 0.5 - 44.0, 22.0, 17.0,
                  Color::new(1.0, 0.45, 0.05, alpha));
    }

    // Timer top-right
    let mins = (time_alive / 60.0) as u32;
    let secs = (time_alive % 60.0) as u32;
    draw_text(&format!("{:02}:{:02}", mins, secs), sw - 66.0, 26.0, 24.0,
              Color::new(0.9, 0.9, 0.9, 0.9));

    // Inventory bar bottom
    let count = player.inventory.len().min(8);
    if count == 0 { return; }
    let slot = 46.0f32;
    let inv_w = slot * count as f32;
    let inv_x = (sw - inv_w) * 0.5;
    let inv_y = sh - slot - 6.0;

    draw_rectangle(inv_x - 3.0, inv_y - 3.0, inv_w + 6.0, slot + 6.0,
                   Color::new(0.0, 0.0, 0.0, 0.48));

    for i in 0..count {
        let sx = inv_x + i as f32 * slot;
        let active = i == player.active_slot;
        let bg = if active { Color::new(0.48, 0.42, 0.0, 0.85) }
                 else       { Color::new(0.07, 0.07, 0.07, 0.72) };
        draw_rectangle(sx, inv_y, slot - 2.0, slot - 2.0, bg);
        draw_rectangle_lines(sx, inv_y, slot - 2.0, slot - 2.0, 2.0,
            if active { YELLOW } else { Color::new(0.28, 0.28, 0.28, 0.8) });

        let item = &player.inventory[i];
        sprites.draw_item_hud(item.sprite_col(), sx + 7.0, inv_y + 5.0, slot - 16.0);

        let cnt = match item {
            Item::Food(n) | Item::Water(n) | Item::Ammo(n) => format!("{}", n),
            _ => String::new(),
        };
        if !cnt.is_empty() {
            draw_text(&cnt, sx + slot - 17.0, inv_y + 14.0, 13.0, WHITE);
        }
        draw_text(&(i + 1).to_string(), sx + 2.0, inv_y + 11.0, 11.0,
                  Color::new(0.5, 0.5, 0.5, 0.65));
    }

    if let Some(item) = player.inventory.get(player.active_slot) {
        draw_text(item.name(), inv_x, inv_y - 7.0, 15.0,
                  Color::new(0.9, 0.84, 0.6, 0.95));
    }
}

fn stat_bar(x: f32, y: f32, w: f32, h: f32, frac: f32, col: Color, label: &str) {
    draw_rectangle(x + 24.0, y, w, h, Color::new(0.1, 0.1, 0.1, 0.9));
    let fw = w * frac.clamp(0.0, 1.0);
    if fw > 0.0 { draw_rectangle(x + 24.0, y, fw, h, col); }
    draw_text(label, x, y + h - 1.0, 13.0, Color::new(0.78, 0.78, 0.78, 0.9));
}
