@group(0) @binding(0) var world: texture_storage_3d<r32uint, write>;

@compute
@workgroup_size(1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    if global_id.y < 8 {
        textureStore(world, global_id, vec4<u32>(255, 0, 0, 255));
    }
}