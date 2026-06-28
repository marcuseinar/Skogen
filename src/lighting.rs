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
uniform vec2  player_light;
uniform float ambient;
uniform float time;
uniform vec2  aux0;
uniform vec2  aux1;
uniform vec2  aux2;
uniform vec2  aux3;
uniform float num_aux;

float soft_light(vec2 frag, vec2 center, float radius) {
    return 1.0 - smoothstep(radius * 0.25, radius, distance(frag, center));
}

void main() {
    vec2 frag = vec2(gl_FragCoord.x, resolution.y - gl_FragCoord.y);

    float flicker = 1.0 + sin(time * 11.3) * 0.025 + sin(time * 7.1) * 0.015;
    float light = soft_light(frag, player_light, 260.0 * flicker);

    if (num_aux > 0.0) light += soft_light(frag, aux0, 70.0) * 0.55;
    if (num_aux > 1.0) light += soft_light(frag, aux1, 70.0) * 0.55;
    if (num_aux > 2.0) light += soft_light(frag, aux2, 70.0) * 0.55;
    if (num_aux > 3.0) light += soft_light(frag, aux3, 70.0) * 0.55;

    light = clamp(light + ambient, 0.0, 1.0);
    float darkness = 1.0 - light;

    gl_FragColor = vec4(0.0, 0.02, 0.1, darkness * 0.94);
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
                    UniformDesc::new("resolution",  UniformType::Float2),
                    UniformDesc::new("player_light", UniformType::Float2),
                    UniformDesc::new("ambient",     UniformType::Float1),
                    UniformDesc::new("time",        UniformType::Float1),
                    UniformDesc::new("aux0",        UniformType::Float2),
                    UniformDesc::new("aux1",        UniformType::Float2),
                    UniformDesc::new("aux2",        UniformType::Float2),
                    UniformDesc::new("aux3",        UniformType::Float2),
                    UniformDesc::new("num_aux",     UniformType::Float1),
                ],
                ..Default::default()
            },
        ).expect("failed to compile lighting shader");

        Self { material }
    }

    pub fn draw(&self, player_sp: Vec2, aux_lights: &[Vec2], ambient: f32, time: f32) {
        let sw = screen_width();
        let sh = screen_height();

        self.material.set_uniform("resolution",   (sw, sh));
        self.material.set_uniform("player_light", (player_sp.x, player_sp.y));
        self.material.set_uniform("ambient",      ambient);
        self.material.set_uniform("time",         time);

        let get = |i: usize| -> (f32, f32) {
            aux_lights.get(i).map(|v| (v.x, v.y)).unwrap_or((0.0, 0.0))
        };
        self.material.set_uniform("aux0", get(0));
        self.material.set_uniform("aux1", get(1));
        self.material.set_uniform("aux2", get(2));
        self.material.set_uniform("aux3", get(3));
        self.material.set_uniform("num_aux", aux_lights.len().min(4) as f32);

        gl_use_material(&self.material);
        draw_rectangle(0.0, 0.0, sw, sh, WHITE);
        gl_use_default_material();
    }
}
