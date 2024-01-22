struct Sphere {
    position: vec3<f32>,
    radius: f32
}

struct Camera {
    position: vec3<f32>,
    size: vec2<f32>,
    focal_length: f32,
}

struct Ray {
    origin: vec3<f32>,
    direction: vec3<f32>
}

fn ray_at(ray: Ray, t: f32) -> vec3<f32> {
    return ray.origin + t * ray.direction;
}

@group(0) @binding(0) var world: texture_3d<u32>;
@group(0) @binding(1) var output_texture: texture_storage_2d<rgba8uint, write>;
@group(0) @binding(2) var<uniform> camera : Camera;

@compute
@workgroup_size(1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let ratio = camera.size.x / camera.size.y;
    let viewport = vec2(2.0 * ratio, 2.0);
    let pixel_pos = vec2(f32(global_id.x), f32(global_id.y));
    var uvw = vec3((pixel_pos / camera.size) * viewport.y - 1.0, camera.focal_length);
    uvw.x *= ratio;

    let ray = Ray(camera.position, uvw);

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

    var color = vec3(255u);
    if (!hit) {
        color = vec3(0u);
    }
    else {
        if (mask.x) {
		    color = vec3(128u);
        }
        if (mask.y) {
            color = vec3(255u);
        }
        if (mask.z) {
            color = vec3(192u);
        }
    }
	

    textureStore(output_texture, vec2(i32(global_id.x), i32(global_id.y)), vec4<u32>(color, 256u));
}