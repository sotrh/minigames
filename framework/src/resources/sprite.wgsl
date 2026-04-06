struct SpriteVertex {
    @location(0)
    positon: vec2<f32>,
    @location(1)
    uv: vec2<f32>,
}

struct Camera {
    view: mat4x4<f32>,
    proj: mat4x4<f32>,
}

struct VsOut {
    @builtin(position)
    clip_pos: vec4<f32>,
    @location(0)
    uv: vec2<f32>,
}

@group(0)
@binding(0)
var<uniform> camera: Camera;

@group(1)
@binding(0)
var sprite_tex: texture_2d<f32>;

@group(1)
@binding(1)
var sprite_sampler: sampler;

@vertex
fn position_sprite(v: SpriteVertex) -> VsOut {
    let clip_position = camera.proj * camera.view * vec4(v.positon, 0.0, 1.0);
    return VsOut(clip_position, v.uv);
}

@fragment
fn texture_sprite(vs: VsOut) -> @location(0) vec4<f32> {
    let color = textureSample(sprite_tex, sprite_sampler, vs.uv);
    return color;
}
