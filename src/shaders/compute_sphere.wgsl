@group(0) @binding(0) var world: texture_storage_3d<r32uint, write>;

@compute
@workgroup_size(1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let offset = vec3<u32>(0);
    let voxel_idx = global_id + offset;
    if distance(vec3<f32>(voxel_idx), vec3<f32>(16.0)) < 16 {
        textureStore(world, voxel_idx, vec4<u32>(255, 0, 0, 255));
    }
}