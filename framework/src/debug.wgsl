struct ColoredVertex {
    @location(0)
    position: vec3<f32>,
    @location(1)
    color: vec3<f32>,
}

struct CameraData {
    view: mat4x4<f32>,
    proj: mat4x4<f32>,
}

struct VsOut {
    @builtin(position)
    clip_position: vec4<f32>,
    @location(0)
    color: vec3<f32>,
}

@group(0)
@binding(0)
var<uniform> camera: CameraData;

@vertex
fn vs_main(vertex: ColoredVertex) -> VsOut {
    let clip_position = camera.proj * camera.view * vec4(vertex.position, 1.0);
    return VsOut(clip_position, vertex.color);
}

@fragment
fn draw_colored(vs: VsOut) -> @location(0) vec4<f32> {
    return vec4(vs.color, 1.0);
}