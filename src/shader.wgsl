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

struct Flags {
    shape_type: u32,
    lhs: u32, // Unused for non-composite types
    rhs: u32, // unused for non-composite types
}

struct Shape {
    color: vec3<f32>,
    index: u32,
    shape_type: u32,
};

// type 0
struct Sphere {
    position: vec3<f32>,
    radius: f32,
}
//#endregion

//#region bindings

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(1) @binding(1)
var<storage> shapes: array<Shape>;

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

fn map_screen_space(size: vec2<f32>, input: vec2<f32>) -> vec2<f32> {
    let xs = size.x / 2.0;
    let ys = size.y / 2.0;
    return vec2<f32>((input.x + 1.0) * xs, (input.y + 1.0) * ys);
}
//#endregion

//#region SDF

fn sphere_sdf(sample_point: vec3<f32>, center: vec3<f32>, radius: f32) -> f32 {
    return length(sample_point - center) - radius;
}

fn scene_sdf(sample_point: vec3<f32>) -> f32 {
//    let sphere_center = camera_translate(vec3<f32>(0.0, 0.0, -3.0));
    let sphere_center = vec3<f32>(0.0, 0.0, -3.0);
    return sphere_sdf(sample_point, sphere_center, 1.0);
}

fn shortest_distance_to_surface(eye: vec3<f32>, dir: vec3<f32>, start: f32, MAX_DIST: f32) -> f32 {
    let EPSILON = 0.0001;

    var depth: f32 = start;
    for (var i: i32 = 0; i < 255; i++) {
        let dist = scene_sdf(eye + depth * dir);
        if (dist < EPSILON) {
            return depth;
        }
        depth += dist;
        if (depth >= MAX_DIST) {
            return MAX_DIST;
        }
    }
    return MAX_DIST;
}

fn approximate_normal(p: vec3<f32>) -> vec3<f32> {
    let EPSILON = 0.0001; // defined in THREE places now. When will it end?
    return normalize(vec3<f32>(
        scene_sdf(vec3<f32>(p.x + EPSILON, p.y, p.z)) - scene_sdf(vec3<f32>(p.x - EPSILON, p.y, p.z)),
        scene_sdf(vec3<f32>(p.x, p.y + EPSILON, p.z)) - scene_sdf(vec3<f32>(p.x, p.y - EPSILON, p.z)),
        scene_sdf(vec3<f32>(p.x, p.y, p.z + EPSILON)) - scene_sdf(vec3<f32>(p.x, p.y, p.z - EPSILON)),
    ));
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
    let EPSILON = 0.0001; // Defined in two places. Fix?

    let screen_size = vec2<f32>(800.0, 600.0); // Pass in though buffer? somewhere?

    let eye = camera.view_pos.xyz;
    var dir: vec3<f32> = ray_direction(45.0, screen_size, map_screen_space(screen_size, in.position.xy));
    dir = dir * mat3x3<f32>(camera.view_angle[0].xyz, camera.view_angle[1].xyz, camera.view_angle[2].xyz); // get 3x3 submatrix

    let dist = shortest_distance_to_surface(eye, dir, 0.0, 100.0);
    if (dist >= MAX_DIST - EPSILON) {
        // didn't hit anything
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }
    let color = approximate_normal(eye + dir * dist);
    return vec4<f32>(color, 1.0);
}

//#endregion