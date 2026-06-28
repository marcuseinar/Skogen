use macroquad::prelude::*;

pub struct NoiseEvent {
    pub pos:  Vec2,
    pub radius: f32,
    pub intensity: f32,
    age: f32,
    max_age: f32,
}

pub struct NoiseSystem {
    pub events: Vec<NoiseEvent>,
}

impl NoiseSystem {
    pub fn new() -> Self { Self { events: Vec::new() } }

    pub fn emit(&mut self, pos: Vec2, radius: f32, intensity: f32) {
        self.events.push(NoiseEvent { pos, radius, intensity, age: 0.0, max_age: 2.5 });
    }

    pub fn update(&mut self, dt: f32) {
        for e in &mut self.events { e.age += dt; }
        self.events.retain(|e| e.age < e.max_age);
    }

    // Returns the position of the loudest recent noise within `hearing_radius` of `listener`,
    // or None if silent.
    pub fn loudest_near(&self, listener: Vec2, hearing_radius: f32) -> Option<Vec2> {
        let mut best: Option<(f32, Vec2)> = None;
        for e in &self.events {
            let dist = (e.pos - listener).length();
            if dist > hearing_radius + e.radius { continue; }
            let effective = e.intensity * (1.0 - (dist / (hearing_radius + e.radius)).min(1.0));
            if effective < 0.05 { continue; }
            if best.map_or(true, |(b, _)| effective > b) {
                best = Some((effective, e.pos));
            }
        }
        best.map(|(_, p)| p)
    }

    // Current total noise intensity at a position (for UI indicator)
    pub fn intensity_at(&self, pos: Vec2) -> f32 {
        let mut total = 0.0f32;
        for e in &self.events {
            let dist = (e.pos - pos).length();
            if dist <= e.radius {
                let fade = 1.0 - e.age / e.max_age;
                total = total.max(e.intensity * fade * (1.0 - dist / e.radius));
            }
        }
        total
    }
}
