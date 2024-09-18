struct Globals {
    screen_size_recip: vec2f,
    scale_factor: f32,
}

@group(0) @binding(0) var<uniform> globals: Globals;

struct VertexInput {
    @builtin(vertex_index) vertex_index: u32,
    @location(0) color: vec4f,
    @location(1) pos: vec2f,
    @location(2) size: vec2f,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4f,
    @location(0) color: vec4f,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    let x = (f32(1 - i32(input.vertex_index)) + 1.0) / 2.0;
    let y = (f32(i32(input.vertex_index & 1u) * 2 - 1) + 1.0) / 2.0;

    let screen_pos: vec2f = (input.pos + (vec2f(x, y) * input.size)) * globals.scale_factor;
    out.clip_position = vec4<f32>(
        (screen_pos.x * globals.screen_size_recip.x) - 1.0,
        1.0 - (screen_pos.y * globals.screen_size_recip.y),
        0.0,
        1.0
    );

    out.color = input.color;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    return in.color;
}