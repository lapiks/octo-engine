struct Sphere {
    position: vec3<f32>,
    radius: f32
}

struct Camera {
    position: vec3<f32>,
    focal_length: f32,
}

struct Ray {
    origin: vec3<f32>,
    direction: vec3<f32>
}

fn ray_at(ray: Ray, t: f32) -> vec3<f32> {
    return ray.origin + t * ray.direction;
}

fn ray_hit_sphere(ray: Ray, sphere: Sphere) -> bool {
    let oc = ray.origin - sphere.position;
    let a = dot(ray.direction, ray.direction);
    let b = 2.0 * dot(oc, ray.direction);
    let c = dot(oc, oc) - sphere.radius * sphere.radius;
    let discriminant = b*b - 4.0*a*c;
    return discriminant >= 0.0;
}

fn ray_color(ray: Ray) -> vec4<f32> {
    let sphere = Sphere(vec3(0.0, 0.0, 0.0), 0.5); 
    if(ray_hit_sphere(ray, sphere)) {
        return vec4<f32>(1.0, 0.0, 0.0, 1.0);
    }

    let y = 0.5 * (normalize(ray.direction).y + 1.0);
    let color = mix(vec3(0.5, 0.7, 1.0), vec3(1.0), y);
    return vec4<f32>(color, 1.0);
} 

struct Globals {
    width: u32,
    height: u32
};

@group(0) @binding(0) var output_texture: texture_storage_2d<rgba8uint, write>;
@group(0) @binding(1) var<uniform> globals : Globals;
@group(0) @binding(2) var<uniform> camera : Camera;

@compute
@workgroup_size(1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let screen_size = vec2(f32(globals.width), f32(globals.height));
    let ratio = screen_size.x / screen_size.y;
    let viewport = vec2(2.0 * ratio, 2.0);
    let pixel_pos = vec2(f32(global_id.x), f32(global_id.y));
    var uvw = vec3((pixel_pos / screen_size) * viewport.y - 1.0, camera.focal_length);
    uvw.x *= ratio;

    let ray = Ray(camera.position, uvw);
    let color = ray_color(ray);

    textureStore(output_texture, vec2(i32(global_id.x), i32(global_id.y)), vec4<u32>(color * 256.0));
}