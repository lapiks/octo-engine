struct Ray {
    origin: vec3<f32>,
    direction: vec3<f32>
}

fn ray_at(ray: Ray, t: f32) -> vec3<f32> {
    return ray.origin + t * ray.direction;
}

struct Camera {
    model: mat4x4<f32>,
    size: vec2<f32>,
}

@group(0) @binding(0) var world: texture_3d<u32>;
@group(0) @binding(1) var output_texture: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(2) var<uniform> camera : Camera;

@compute
@workgroup_size(1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let pixel_pos = vec2(f32(global_id.x), f32(global_id.y));
    let screen_pos = (pixel_pos / camera.size) * 2.0 - 1.0;
	let camera_plane_u = vec3(camera.model[0][0], camera.model[0][1], camera.model[0][2]);
	let camera_plane_v = vec3(camera.model[1][0], camera.model[1][1], camera.model[1][2]) * camera.size.y / camera.size.x;
    let camera_direction = vec3(camera.model[2][0], camera.model[2][1], camera.model[2][2]);
    let camera_position = vec3(camera.model[3][0], camera.model[3][1], camera.model[3][2]);
	let ray_dir = camera_direction + screen_pos.x * camera_plane_u + screen_pos.y * camera_plane_v;
    
    let ray = Ray(camera_position, ray_dir);

    var map_pos = vec3(floor(ray.origin));
    let delta_dist = abs(vec3(length(ray.direction)) / ray.direction);
    let ray_step = vec3(sign(ray.direction));
    var side_dist = (sign(ray.direction) * (map_pos - ray.origin) + (sign(ray.direction) * 0.5) + 0.5) * delta_dist; 
    var mask: vec3<bool> = vec3(false);

    let MAX_RAY_STEPS = 64;
    var hit = false;
    for (var i: i32 = 0; i < MAX_RAY_STEPS; i++) {
        let voxel = textureLoad(world, vec3<i32>(map_pos), 0);
        if (voxel.x == 255u) {
            hit = true;
            break;
        }
        mask = side_dist.xyz < min(side_dist.yzx, side_dist.zxy);
        side_dist += vec3<f32>(mask) * delta_dist;
		map_pos += vec3<f32>(mask) * ray_step;
    }

    var color = vec3(1.0);
    if (!hit) {
        color = vec3(0.0);
    }
    else {
        if (mask.x) {
		    color = vec3(0.5);
        }
        if (mask.y) {
            color = vec3(1.0);
        }
        if (mask.z) {
            color = vec3(0.75);
        }
    }
	
    textureStore(output_texture, vec2(i32(global_id.x), i32(u32(camera.size.y) - global_id.y)), vec4<f32>(color, 1.0));
}