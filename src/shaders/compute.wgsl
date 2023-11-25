@group(0)
@binding(0)
var output_texture: texture_storage_2d<rgba8uint, write>;

@compute
@workgroup_size(1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    textureStore(output_texture, vec2(i32(global_id.x), i32(global_id.y)), vec4<u32>(256u, 0u, 0u, 256u));
}