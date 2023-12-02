struct VertexOutput {
  @builtin(position) Position : vec4<f32>,
  @location(0) fragUV : vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_idx : u32) -> VertexOutput {
    var out: VertexOutput;
    let x = -1.0 + f32(i32(vertex_idx & 1u) << 2u);
    let y = -1.0 + f32(i32(vertex_idx & 2u) << 1u);
    out.Position = vec4<f32>(x, y, 0.0, 1.0);
    out.fragUV = vec2<f32>((x + 1.0) / 2.0f, (y + 1.0) / 2.0);
    return out;
}

struct Globals {
    width: u32,
    height: u32
};

@group(0) @binding(0) var t_color : texture_2d<u32>;
@group(0) @binding(1) var<uniform> globals : Globals;

@fragment
fn fs_main(@location(0) fragUV : vec2<f32>) -> @location(0) vec4<f32> {
    let screen_size = vec2(f32(globals.width), f32(globals.height));
    let tex = textureLoad(t_color, vec2<i32>(fragUV * screen_size), 0);
    let r = f32(tex.x) / 256.0;
    let g = f32(tex.y) / 256.0;
    let b = f32(tex.z) / 256.0;
    return vec4<f32>(r, g, 1.0, 1.0);
}
