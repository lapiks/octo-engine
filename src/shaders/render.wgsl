struct VertexOutput {
  @builtin(position) Position : vec4<f32>,
  @location(0) fragUV : vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_idx : u32) -> VertexOutput {
    var out: VertexOutput;
    let x = -1.0f + f32(i32(vertex_idx & 1u) << 2u);
    let y = -1.0f + f32(i32(vertex_idx & 2u) << 1u);
    out.Position = vec4<f32>(x, y, 0.0, 1.0);
    return out;
}

@group(0) @binding(0) var t_color : texture_2d<u32>;

@fragment
fn fs_main(@location(0) fragUV : vec2<f32>) -> @location(0) vec4<f32> {
    let tex = textureLoad(t_color, vec2<i32>(fragUV * 256.0), 0);
    let r = f32(tex.x) / 255.0;
    let g = f32(tex.y) / 255.0;
    let b = f32(tex.z) / 255.0;
    return vec4<f32>(r, g, b, 1.0);
}
