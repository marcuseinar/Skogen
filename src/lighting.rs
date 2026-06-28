use macroquad::prelude::*;
use macroquad::miniquad::{BlendFactor, BlendState, BlendValue, Equation};

const VERT: &str = r#"
#version 100
attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;
varying lowp vec2 uv;
varying lowp vec4 color;
uniform mat4 Model;
uniform mat4 Projection;
void main() {
    gl_Position = Projection * Model * vec4(position, 1.0);
    color = color0 / 255.0;
    uv = texcoord;
}
"#;

const FRAG: &str = r#"
#version 100
precision mediump float;
varying lowp vec2 uv;
varying lowp vec4 color;

uniform vec2  resolution;
uniform vec2  player_pos;
uniform float player_angle;
uniform float flashlight_on;
uniform float flashlight_battery;
uniform float ambient;
uniform float time;

#define PI  3.14159265358979
#define TAU 6.28318530717959

float soft_circle(vec2 frag, vec2 center, float radius) {
    return 1.0 - smoothstep(radius * 0.55, radius, distance(frag, center));
}

float flashlight(vec2 frag, vec2 center, float angle, float radius) {
    vec2 d = frag - center;
    float dist = length(d);
    float fa = atan(d.y, d.x);
    float diff = fa - angle;
    diff = diff - TAU * floor((diff + PI) / TAU);
    float half_cone = 0.52;
    float cone    = 1.0 - smoothstep(half_cone * 0.65, half_cone, abs(diff));
    float falloff = 1.0 - smoothstep(radius * 0.40, radius, dist);
    float hot     = (1.0 - smoothstep(0.0, radius * 0.15, dist)) * 0.55;
    float spill   = (1.0 - smoothstep(radius * 0.12, radius * 0.42, dist))
                  * (1.0 - smoothstep(PI * 0.5, PI, abs(diff))) * 0.12;
    return cone * falloff + hot + spill;
}

void main() {
    vec2 frag = vec2(gl_FragCoord.x, resolution.y - gl_FragCoord.y);

    float light = soft_circle(frag, player_pos, 38.0) * 0.13;

    if (flashlight_on > 0.5) {
        float batt = flashlight_battery;
        float batt_flicker = 1.0;
        if (batt < 0.25) {
            batt_flicker = 0.6 + 0.4 * sin(time * 28.0 * (1.0 - batt * 3.0));
        }
        float flicker = 1.0 + sin(time * 17.3) * 0.014 + sin(time * 7.1) * 0.008;
        light += flashlight(frag, player_pos, player_angle, 285.0 * flicker) * batt * batt_flicker;
    }

    light = clamp(light + ambient, 0.0, 1.0);
    float dark = 1.0 - light;
    gl_FragColor = vec4(0.01, 0.01, 0.055, dark * 0.975);
}
"#;

pub struct LightSystem {
    material: Material,
}

impl LightSystem {
    pub fn new() -> Self {
        let material = load_material(
            ShaderSource::Glsl { vertex: VERT, fragment: FRAG },
            MaterialParams {
                pipeline_params: PipelineParams {
                    color_blend: Some(BlendState::new(
                        Equation::Add,
                        BlendFactor::Value(BlendValue::SourceAlpha),
                        BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
                    )),
                    ..Default::default()
                },
                uniforms: vec![
                    UniformDesc::new("resolution",         UniformType::Float2),
                    UniformDesc::new("player_pos",         UniformType::Float2),
                    UniformDesc::new("player_angle",       UniformType::Float1),
                    UniformDesc::new("flashlight_on",      UniformType::Float1),
                    UniformDesc::new("flashlight_battery", UniformType::Float1),
                    UniformDesc::new("ambient",            UniformType::Float1),
                    UniformDesc::new("time",               UniformType::Float1),
                ],
                ..Default::default()
            },
        ).expect("lighting shader compile failed");
        Self { material }
    }

    pub fn draw(
        &self,
        player_sp: Vec2,
        player_angle: f32,
        flashlight_on: bool,
        battery_frac: f32,
        ambient: f32,
        time: f32,
    ) {
        let sw = screen_width();
        let sh = screen_height();
        self.material.set_uniform("resolution",         (sw, sh));
        self.material.set_uniform("player_pos",         (player_sp.x, player_sp.y));
        self.material.set_uniform("player_angle",       player_angle);
        self.material.set_uniform("flashlight_on",      if flashlight_on { 1.0f32 } else { 0.0f32 });
        self.material.set_uniform("flashlight_battery", battery_frac);
        self.material.set_uniform("ambient",            ambient);
        self.material.set_uniform("time",               time);
        gl_use_material(&self.material);
        draw_rectangle(0.0, 0.0, sw, sh, WHITE);
        gl_use_default_material();
    }
}
