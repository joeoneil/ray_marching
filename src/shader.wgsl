//#region constants
let EPSILON: f32 = 0.0001;
let ambient_light: vec3<f32> = vec3<f32>(0.5, 0.5, 0.5);
//#endregion

//#region typedefs
struct VertexInput {
    @location(0) position: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) position: vec3<f32>,
}

struct CameraUniform {
    view_pos: vec4<f32>,
    view_angle: mat4x4<f32>,
};

struct Config {
    time: f32,
    width: u32,
    height: u32,

    shape_count: u32,
    sphere_count: u32,
    prism_count: u32,
}

struct Shape {
    color: vec4<f32>,
    index: u32,
    shape_type: u32,
    bounding_box: vec4<f32>,
};

// type 0
struct Sphere {
    model: vec4<f32>, // vec3 pos, f32 radius
}

// type 1
struct RectPrism {
    model: vec3<f32>, // vec3 pos
    size: vec3<f32>, // vec3 size
    rot: vec4<f32>, // quaternion rotation
}
//#endregion

//#region bindings
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(1) @binding(1)
var<uniform> config: Config;

@group(2) @binding(0)
var<storage, read> shapes: array<Shape>;

@group(2) @binding(1)
var<storage, read> spheres: array<Sphere>;

@group(2) @binding(2)
var<storage, read> prisms: array<RectPrism>;

//#endregion

//#region helper functions
fn rgb_to_srgb(rgb: vec4<f32>) -> vec4<f32> {
    var x: f32 = pow((rgb.x + 0.055) / 1.055, 2.4);
    var y: f32 = pow((rgb.y + 0.055) / 1.055, 2.4);
    var z: f32 = pow((rgb.z + 0.055) / 1.055, 2.4);
    return vec4<f32>(x, y, z, rgb.w);
}

fn ray_direction(fov: f32, size: vec2<f32>, coord: vec2<f32>) -> vec3<f32> {
    var xy: vec2<f32> = coord - size / 2.0;
    var z: f32 = (size.y / 2.0) / tan(radians(fov) / 2.0);
    return normalize(vec3<f32>(xy, -z));
}

fn inv_ray_direction(fov: f32, size: vec2<f32>, world_coord: vec3<f32>) -> vec2<f32> {
    var z: f32 = (size.y / 2.0) / tan(radians(fov) / 2.0);
    let scale = z / world_coord.z;
    let scaled_coord = world_coord * scale;
    return world_coord.xy + size / 2.0;
}

fn map_screen_space(size: vec2<f32>, input: vec2<f32>) -> vec2<f32> {
    let xs = size.x / 2.0;
    let ys = size.y / 2.0;
    return vec2<f32>((input.x + 1.0) * xs, (input.y + 1.0) * ys);
}
//#endregion

//#region SDF

fn sphere_sdf(sample_point: vec3<f32>, sphere: Sphere) -> f32 {
    let center = sphere.model.xyz;
    let radius = sphere.model.w;
    return length(sample_point - center) - radius;
}

// says cube but is actually a rect prism
fn cube_sdf(sample_point: vec3<f32>, center: vec3<f32>, bounds: vec3<f32>, rot: vec4<f32>) -> f32 {
    if (max(bounds.x, max(bounds.y, bounds.z)) < EPSILON) {
        return 100.0; // arbitrary large number
    }
    let p = sample_point - center;
    // rotate point about center by quaternion rot
    let q = rot.xyz;
    let r = rot.w;
    let t = 2.0 * cross(q, p);
    let p = p + r * t + cross(q, t);
    // get distance from point to rectangular prism (NOT A CUBE)
    let d = abs(p) - bounds;
    return length(max(d, vec3<f32>(0.0))) + min(max(d.x, max(d.y, d.z)), 0.0);
}

fn scene_sdf(sample_point: vec3<f32>, pixel_coord: vec2<f32>) -> vec4<f32> {
    var min_dist: f32 = 100.0; // arbitrary large number
    var color: vec3<f32> = vec3<f32>(0.0, 0.0, 0.0); // object color
    for (var i: i32 = 0; i < i32(config.shape_count); i++) {
        var dist: f32 = 100.0;

        let min = shapes[i].bounding_box.xy;
        let max = shapes[i].bounding_box.zw;
        if (pixel_coord.x < min.x || pixel_coord.x > max.x || pixel_coord.y < min.y || pixel_coord.y > max.y) {
            continue;
        }

        // type 0 = sphere
        if (shapes[i].shape_type == u32(0)) {
            dist = sphere_sdf(sample_point, spheres[shapes[i].index]);
        }

        // type 1 = rect prism
        else if (shapes[i].shape_type == u32(1)) {
            dist = cube_sdf(sample_point, prisms[shapes[i].index].model.xyz, prisms[shapes[i].index].size.xyz, prisms[shapes[i].index].rot);
        }

        else {
            continue;
        }

        if (dist < min_dist) {
            min_dist = dist;
            color = shapes[i].color.xyz;
        }
    }
    return vec4<f32>(color, min_dist);
}

fn shortest_distance_to_surface(eye: vec3<f32>, dir: vec3<f32>, start: f32, MAX_DIST: f32, pixel_coord: vec2<f32>) -> vec4<f32> {
    var depth: f32 = start;
    for (var i: i32 = 0; i < 50; i++) {
        let dist = scene_sdf(eye + depth * dir, pixel_coord);
        if (dist.w < EPSILON) {
            return vec4<f32>(dist.xyz, depth);
        }
        depth += dist.w;
        if (depth >= MAX_DIST) {
            return vec4<f32>(0.1, 0.2, 0.3, MAX_DIST);
        }
    }
    return vec4<f32>(0.1, 0.2, 0.3, MAX_DIST);
}

fn approximate_normal(p: vec3<f32>, c: vec2<f32>) -> vec3<f32> {
    return normalize(vec3<f32>(
        scene_sdf(vec3<f32>(p.x + EPSILON, p.y, p.z), c).w - scene_sdf(vec3<f32>(p.x - EPSILON, p.y, p.z), c).w,
        scene_sdf(vec3<f32>(p.x, p.y + EPSILON, p.z), c).w - scene_sdf(vec3<f32>(p.x, p.y - EPSILON, p.z), c).w,
        scene_sdf(vec3<f32>(p.x, p.y, p.z + EPSILON), c).w - scene_sdf(vec3<f32>(p.x, p.y, p.z - EPSILON), c).w,
    ));
}

//#endregion

//#region lighting
fn phong_contrib(
    k_d: vec3<f32>, // Diffuse Color
    k_s: vec3<f32>, // Specular Color
    alpha: f32, // Shininess Coefficient
    p: vec3<f32>, // Position of point being lit
    eye: vec3<f32>, // Position of the camera
    light_pos: vec3<f32>, // Position of the light
    intensity: vec3<f32>, // color / intensity of the light
    pixel_coord: vec2<f32>
    ) -> vec3<f32> {
    let N = approximate_normal(p, pixel_coord);
    let L = normalize(light_pos - p);
    let V = normalize(eye - p);
    let R = normalize(reflect(-L, N));

    let dotLN = dot(L, N);
    let dotRV = dot(R, V);

    if (dotLN < 0.0) {
        // Surface is not visible to camera
        return vec3<f32>(0.0, 0.0, 0.0);
    }

    if (dotRV < 0.0) {
        // Light reflection pointed away from camera. Apply only diffuse color
        return intensity * (k_d * dotLN);
    }

    return intensity * (k_d * dotLN + k_s * pow(dotRV, alpha));
}

fn phong_illumination(
    k_a: vec3<f32>, // Ambient Color
    k_d: vec3<f32>, // Diffuse Color
    k_s: vec3<f32>, // Specular Color
    alpha: f32, // Shininess Coefficient
    p: vec3<f32>, // Position of point being lit
    eye: vec3<f32>, // Position of the camera
    pixel_coord: vec2<f32>, // pixel coordinate
) -> vec3<f32> {
    var color: vec3<f32> = ambient_light * k_a;

    let l1_pos = vec3<f32>(20.0, 20.0, 15.0);
    let l1_intensity = vec3<f32>(0.6, 0.6, 0.6);

    color += phong_contrib(k_d, k_s, alpha, p, eye, l1_pos, l1_intensity, pixel_coord);

//    let l2_pos = vec3<f32>(2.0 * sin(0.37 * config.time), 2.0 * cos(0.37 * config.time), 2.0);
//    let l2_intensity = vec3<f32>(0.4, 0.4, 0.4);
//
//    color += phong_contrib(k_d, k_s, alpha, p, eye, l2_pos, l2_intensity);

    return color;
}
//#endregion

//#region entrypoints

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(model.position, 1.0);
    out.position = model.position;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let MIN_DIST = 0.0;
    let MAX_DIST = 100.0;

    let screen_size = vec2<f32>(f32(config.width), f32(config.height));
//    let pixel_coord = map_screen_space(screen_size, in.position.xy);

    let pixel_coord = vec2<f32>(in.clip_position.x, in.clip_position.y);
    if ((i32(pixel_coord.x) == i32(screen_size.x / 2.0) || i32(pixel_coord.y) == i32(screen_size.y / 2.0)) && (abs(pixel_coord.x - screen_size.x / 2.0) < 10.0 && abs(pixel_coord.y - screen_size.y / 2.0) < 10.0)) {
        return vec4<f32>(0.5, 0.5, 0.5, 1.0);
    }

    let eye = camera.view_pos.xyz;
    var dir: vec3<f32> = ray_direction(45.0, screen_size, pixel_coord);
    dir = dir * mat3x3<f32>(camera.view_angle[0].xyz, camera.view_angle[1].xyz, camera.view_angle[2].xyz); // get 3x3 submatrix

    let dist = shortest_distance_to_surface(eye, dir, 0.0, 100.0, pixel_coord);
    if (dist.w >= MAX_DIST - EPSILON) {

        // didn't hit anything
        return vec4<f32>(dist.xyz, 1.0);
    }
    let k_a = vec3<f32>(0.5, 0.5, 0.5) * dist.xyz;
    let k_d = dist.xyz;
    let k_s = vec3<f32>(1.0, 1.0, 1.0);
    let shininess = 1000.0;

    let color = phong_illumination(k_a, k_d, k_s, shininess, eye + dist.w * dir, eye, pixel_coord);
    return rgb_to_srgb(vec4<f32>(color, 1.0));
    // return rgb_to_srgb(vec4<f32>(shapes[0].bounding_box.xy, 0.0, 1.0));
}

//#endregion
