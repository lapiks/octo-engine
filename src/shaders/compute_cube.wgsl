@group(0) @binding(0) var world: texture_storage_3d<r32uint, write>;

@compute
@workgroup_size(1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let offset = vec3<u32>(6);
    let voxel_idx = global_id + offset;
    if voxel_idx.y < 8 {
        textureStore(world, voxel_idx, vec4<u32>(255, 0, 0, 255));
    }
}