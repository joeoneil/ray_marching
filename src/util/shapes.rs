use bytemuck::Contiguous;
use cgmath::{Quaternion, Rotation3, Vector2, Vector3};
use std::any::Any;
use wgpu::Device;

use super::super::ShaderParams;

pub enum Flag {
    Enabled,
}

pub struct Flags {
    enabled: bool,
}

impl Flags {
    fn all() -> Self {
        Flags { enabled: true }
    }

    fn none() -> Self {
        Flags { enabled: false }
    }

    fn enabled() -> Self {
        Flags { enabled: true }
    }

    fn as_u32(&self) -> u32 {
        let mut flags = 0;
        flags |= (self.enabled as u32) << 0;
        flags
    }

    fn get_flag(&self, flag: Flag) -> bool {
        match flag {
            Flag::Enabled => self.enabled,
        }
    }

    fn set_flag(&mut self, flag: Flag, value: bool) {
        match flag {
            Flag::Enabled => self.enabled = value,
        }
    }

    fn set(&mut self, other: Flags) {
        self.enabled = other.enabled;
    }
}

//#region Shape
pub trait Shape {
    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn shape_data(
        &self,
        inv_c_matrix: cgmath::Matrix4<f32>,
        proj_matrix: cgmath::Matrix4<f32>,
        screen_size: (usize, usize),
    ) -> ShapeData;

    fn translate(&mut self, translation: Vector3<f32>);

    fn set_pos(&mut self, pos: Vector3<f32>);

    fn rotate(&mut self, rotation: Quaternion<f32>);

    fn set_rotation(&mut self, rotation: Quaternion<f32>);

    fn get_index(&self) -> u32;

    fn get_world_bounding_box(&self) -> (Vector3<f32>, Vector3<f32>);

    fn get_screen_bounding_box(
        &self,
        inv_c_matrix: cgmath::Matrix4<f32>,
        proj_matrix: cgmath::Matrix4<f32>,
        screen_size: (usize, usize),
    ) -> [f32; 4] {
        let (c1, c2) = self.get_world_bounding_box();
        // Copilot please
        // get the 8 corners of the bounding box
        let corners = [
            Vector3::new(c1.x, c1.y, c1.z),
            Vector3::new(c1.x, c1.y, c2.z),
            Vector3::new(c1.x, c2.y, c1.z),
            Vector3::new(c1.x, c2.y, c2.z),
            Vector3::new(c2.x, c1.y, c1.z),
            Vector3::new(c2.x, c1.y, c2.z),
            Vector3::new(c2.x, c2.y, c1.z),
            Vector3::new(c2.x, c2.y, c2.z),
        ];
        // corners are in world space, transform to camera space using the inverse camera matrix
        // then transform to screen space using the projection matrix
        // then transform to normalized device coordinates using the screen size
        let corners = corners
            .iter()
            .map(|c| {
                let c = inv_c_matrix * c.extend(1.0); // world to camera
                let c = proj_matrix * c; // camera to screen
                let c = c.truncate() / c.w; // screen to NDC
                let c = Vector2::new((c.x + 1.0) / 2.0, (c.y + 1.0) / 2.0); // NDC to screen
                let c = Vector2::new(c.x * screen_size.0 as f32, c.y * screen_size.1 as f32); // screen to pixels
                c
            })
            .collect::<Vec<_>>();
        // get the min and max of the corners
        let min = corners
            .iter()
            .fold(Vector2::new(f32::MAX, f32::MAX), |acc, c| {
                Vector2::new(acc.x.min(c.x), acc.y.min(c.y))
            });
        let max = corners
            .iter()
            .fold(Vector2::new(f32::MIN, f32::MIN), |acc, c| {
                Vector2::new(acc.x.max(c.x), acc.y.max(c.y))
            });
        [min.x, min.y, max.x, max.y]
    }

    fn get_flags(&self) -> &Flags;

    fn get_flags_mut(&mut self) -> &mut Flags;

    fn set_flags(&mut self, flags: Flags) {
        self.get_flags_mut().set(flags);
    }

    fn get_flag(&self, flag: Flag) -> bool {
        self.get_flags().get_flag(flag)
    }

    fn set_flag(&mut self, flag: Flag, value: bool) {
        self.get_flags_mut().set_flag(flag, value);
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ShapeData {
    color: [f32; 4], // align of vec_3 is 16 bytes on GPU
    index: u32,
    shape_type: u32,
    flags: u32,
    _padding: [f32; 1], // aligns total size to align of largest element (vec3, 16 bytes)
    bounding_box: [f32; 4], // screen-space bounding box
}

impl Default for ShapeData {
    fn default() -> Self {
        Self {
            color: [0.0, 0.0, 0.0, 0.0],
            index: u32::MAX_VALUE,
            shape_type: u32::MAX_VALUE,
            flags: 0,
            _padding: [0.0],
            bounding_box: [f32::MIN, f32::MIN, f32::MAX, f32::MAX],
        }
    }
}
//#endregion

pub struct Sphere {
    pos: Vector3<f32>,
    radius: f32,
    color: Vector3<f32>,
    index: u32,
    flags: Flags,
}

//#region Sphere
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct SphereData {
    model: [f32; 4], // vec3 pos, f32 radius
}

impl Sphere {
    fn sphere_data(&self) -> SphereData {
        SphereData {
            model: [self.pos.x, self.pos.y, self.pos.z, self.radius],
        }
    }
}

impl Default for Sphere {
    fn default() -> Self {
        Self {
            pos: Vector3::new(0.0, 0.0, 0.0),
            radius: 1.0,
            color: Vector3::new(1.0, 1.0, 1.0),
            index: u32::MAX_VALUE,
            flags: Flags::enabled(),
        }
    }
}

impl Shape for Sphere {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn shape_data(
        &self,
        inv_c_matrix: cgmath::Matrix4<f32>,
        proj_matrix: cgmath::Matrix4<f32>,
        screen_size: (usize, usize),
    ) -> ShapeData {
        ShapeData {
            color: [self.color.x, self.color.y, self.color.z, 0.0],
            index: self.index,
            shape_type: 0,
            flags: self.flags.as_u32(),
            _padding: [0.0],
            bounding_box: self.get_screen_bounding_box(inv_c_matrix, proj_matrix, screen_size),
        }
    }

    fn translate(&mut self, translation: Vector3<f32>) {
        self.pos += translation;
    }

    fn set_pos(&mut self, pos: Vector3<f32>) {
        self.pos = pos;
    }

    fn rotate(&mut self, _: Quaternion<f32>) {
        // No-op
    }

    fn set_rotation(&mut self, _: Quaternion<f32>) {
        // No-op
    }

    fn get_index(&self) -> u32 {
        self.index
    }

    fn get_world_bounding_box(&self) -> (Vector3<f32>, Vector3<f32>) {
        (
            Vector3::new(
                self.pos.x - self.radius,
                self.pos.y - self.radius,
                self.pos.z - self.radius,
            ),
            Vector3::new(
                self.pos.x + self.radius,
                self.pos.y + self.radius,
                self.pos.z + self.radius,
            ),
        )
    }

    fn get_flags(&self) -> &Flags {
        &self.flags
    }

    fn get_flags_mut(&mut self) -> &mut Flags {
        &mut self.flags
    }
}
//#endregion

//#region Cube
pub struct Cube {
    pos: Vector3<f32>,
    bounds: Vector3<f32>,
    rot: Quaternion<f32>,
    color: Vector3<f32>,
    index: u32,
    flags: Flags,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CubeData {
    model: [f32; 3], // vec3 pos
    _p1: f32,        // padding (vec3 is 16 bytes on GPU)
    size: [f32; 3],  // vec3 bounds
    _p2: f32,        // padding (vec3 is 16 bytes on GPU)
    rot: [f32; 4],   // vec4 rot
}

impl Cube {
    fn cube_data(&self) -> CubeData {
        CubeData {
            model: [self.pos.x, self.pos.y, self.pos.z],
            _p1: 0.0,
            size: [self.bounds.x, self.bounds.y, self.bounds.z],
            _p2: 0.0,
            rot: [self.rot.v.x, self.rot.v.y, self.rot.v.z, self.rot.s],
        }
    }

    pub fn set_bounds(&mut self, bounds: Vector3<f32>) {
        self.bounds = bounds;
    }
}

impl Default for Cube {
    fn default() -> Self {
        Self {
            pos: Vector3::new(0.0, 0.0, 0.0),
            bounds: Vector3::new(1.0, 1.0, 1.0),
            rot: Quaternion::new(1.0, 0.0, 0.0, 0.0),
            color: Vector3::new(0.0, 0.0, 0.0),
            index: u32::MAX_VALUE,
            flags: Flags::enabled(),
        }
    }
}

impl Shape for Cube {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn shape_data(
        &self,
        inv_c_matrix: cgmath::Matrix4<f32>,
        proj_matrix: cgmath::Matrix4<f32>,
        screen_size: (usize, usize),
    ) -> ShapeData {
        ShapeData {
            color: [self.color.x, self.color.y, self.color.z, 0.0],
            index: self.index,
            shape_type: 1,
            flags: self.flags.as_u32(),
            _padding: [0.0],
            bounding_box: self.get_screen_bounding_box(inv_c_matrix, proj_matrix, screen_size),
        }
    }

    fn translate(&mut self, translation: Vector3<f32>) {
        self.pos += translation;
    }

    fn set_pos(&mut self, pos: Vector3<f32>) {
        self.pos = pos;
    }

    fn rotate(&mut self, rotation: Quaternion<f32>) {
        // self.rot = rotation * self.rot;
        self.rot = self.rot * rotation;
    }

    fn set_rotation(&mut self, rotation: Quaternion<f32>) {
        self.rot = rotation;
    }

    fn get_index(&self) -> u32 {
        self.index
    }

    fn get_world_bounding_box(&self) -> (Vector3<f32>, Vector3<f32>) {
        (
            Vector3::new(
                self.pos.x - self.bounds.x,
                self.pos.y - self.bounds.y,
                self.pos.z - self.bounds.z,
            ),
            Vector3::new(
                self.pos.x + self.bounds.x,
                self.pos.y + self.bounds.y,
                self.pos.z + self.bounds.z,
            ),
        )
    }

    fn get_flags(&self) -> &Flags {
        &self.flags
    }

    fn get_flags_mut(&mut self) -> &mut Flags {
        &mut self.flags
    }
}
//#endregion

//#region Union
pub struct Union {
    left: u32,
    right: u32,
    index: u32,
    flags: Flags,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct UnionData {
    left: u32,
    right: u32,
    index: u32,
}

impl Union {
    fn union_data(&self) -> UnionData {
        UnionData {
            left: self.left,
            right: self.right,
            index: self.index,
        }
    }
}

impl Shape for Union {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn shape_data(
        &self,
        inv_c_matrix: cgmath::Matrix4<f32>,
        proj_matrix: cgmath::Matrix4<f32>,
        screen_size: (usize, usize),
    ) -> ShapeData {
        ShapeData {
            color: [0.0, 0.0, 0.0, 0.0],
            index: self.index,
            shape_type: 2,
            flags: self.flags.as_u32(),
            _padding: [0.0],
            bounding_box: self.get_screen_bounding_box(inv_c_matrix, proj_matrix, screen_size),
        }
    }

    fn get_index(&self) -> u32 {
        self.index
    }

    fn set_rotation(&mut self, _: Quaternion<f32>) {
        // No-op
    }

    fn rotate(&mut self, _: Quaternion<f32>) {
        // No-op
    }

    fn set_pos(&mut self, _: Vector3<f32>) {
        // Unions cannot themselves be translated.
    }

    fn translate(&mut self, _: Vector3<f32>) {
        // Unions cannot themselves be translated.
    }

    fn get_world_bounding_box(&self) -> (Vector3<f32>, Vector3<f32>) {
        todo!()
    }

    fn get_flags(&self) -> &Flags {
        &self.flags
    }

    fn get_flags_mut(&mut self) -> &mut Flags {
        &mut self.flags
    }
}
//#endregion

//#region ShapeManager
pub struct ShapeManager {
    // Shæps
    shapes: Vec<Box<dyn Shape>>,
    indices: [u32; 1000],
    map: Vec<Vec<u32>>, // map of indices to shapes
}

impl ShapeManager {
    pub fn new() -> Self {
        Self {
            shapes: vec![],
            indices: [0; 1000],
            map: vec![vec![], vec![]],
        }
    }

    pub fn serialize_shapes(
        &self,
        inv_c_matrix: cgmath::Matrix4<f32>,
        proj_matrix: cgmath::Matrix4<f32>,
        screen_size: (usize, usize),
    ) -> Vec<u8> {
        if self.shapes.len() == 0 {
            return bytemuck::cast_slice(&[ShapeData::default()]).to_vec();
        }
        self.shapes
            .iter()
            .flat_map(|a| -> Vec<u8> {
                bytemuck::cast_slice(&[a.shape_data(inv_c_matrix, proj_matrix, screen_size)])
                    .to_vec()
            })
            .collect()
    }

    pub fn serialize_spheres(&self) -> Vec<u8> {
        if self.map[0].is_empty() {
            return bytemuck::cast_slice(&[Sphere::default().sphere_data()]).to_vec();
        }
        self.shapes
            .iter()
            .filter_map(|a| -> Option<&Sphere> { a.as_any().downcast_ref::<Sphere>() })
            .flat_map(|a| -> Vec<u8> { bytemuck::cast_slice(&[a.sphere_data()]).to_vec() })
            .collect()
    }

    pub fn serialize_cubes(&self) -> Vec<u8> {
        if self.map[1].is_empty() {
            return bytemuck::cast_slice(&[Cube::default().cube_data()]).to_vec();
        }
        self.shapes
            .iter()
            .filter_map(|a| -> Option<&Cube> { a.as_any().downcast_ref::<Cube>() })
            .flat_map(|a| -> Vec<u8> { bytemuck::cast_slice(&[a.cube_data()]).to_vec() })
            .collect()
    }

    pub fn iter_shapes(&self) -> impl Iterator<Item = &Box<dyn Shape>> {
        self.shapes.iter()
    }

    pub fn iter_shapes_mut(&mut self) -> impl Iterator<Item = &mut Box<dyn Shape>> {
        self.shapes.iter_mut()
    }

    pub fn new_sphere(
        &mut self,
        pos: Vector3<f32>,
        radius: f32,
        color: Vector3<f32>,
    ) -> &mut Sphere {
        self.map[0].push(self.shapes.len() as u32);
        self.shapes.push(Box::new(Sphere {
            pos,
            radius,
            color,
            index: self.indices[0],
            flags: Flags::enabled(),
        }));
        self.indices[0] += 1;
        self.shapes
            .last_mut()
            .unwrap()
            .as_any_mut()
            .downcast_mut::<Sphere>()
            .unwrap()
    }

    pub fn new_cube(
        &mut self,
        pos: Vector3<f32>,
        bounds: Vector3<f32>,
        color: Vector3<f32>,
    ) -> &mut Cube {
        self.map[1].push(self.shapes.len() as u32);
        self.shapes.push(Box::new(Cube {
            pos,
            bounds,
            rot: Quaternion::from_angle_z(cgmath::Rad(0.0)),
            color,
            index: self.indices[1],
            flags: Flags::enabled(),
        }));
        self.indices[1] += 1;
        self.shapes
            .last_mut()
            .unwrap()
            .as_any_mut()
            .downcast_mut::<Cube>()
            .unwrap()
    }

    pub fn new_union(&mut self, left: u32, right: u32) -> Option<&mut Union> {
        match (
            self.get_shape(left).is_some(),
            self.get_shape(right).is_some(),
        ) {
            (true, true) => {
                self.get_shape_mut(left)
                    .unwrap()
                    .set_flag(Flag::Enabled, false);
                self.get_shape_mut(right)
                    .unwrap()
                    .set_flag(Flag::Enabled, false);
            }
            _ => return None,
        }

        self.map[2].push(self.shapes.len() as u32);
        self.shapes.push(Box::new(Union {
            left,
            right,
            index: self.indices[2],
            flags: Flags::enabled(),
        }));
        self.indices[2] += 1;
        Some(
            self.shapes
                .last_mut()
                .unwrap()
                .as_any_mut()
                .downcast_mut::<Union>()
                .unwrap(),
        )
    }

    pub fn shape_buffer_size(&self, device: &Device) -> u32 {
        let raw_size = std::mem::size_of::<ShapeData>() * self.shapes.len();
        ShapeManager::buffer_size(raw_size, device)
    }

    pub fn sphere_buffer_size(&self, device: &Device) -> u32 {
        let raw_size = std::mem::size_of::<SphereData>() * self.map[0].len(); // sphere index is sphere count
        ShapeManager::buffer_size(raw_size, device)
    }

    pub fn cube_buffer_size(&self, device: &Device) -> u32 {
        let raw_size = std::mem::size_of::<CubeData>() * self.map[1].len(); // cube index is cube count
        ShapeManager::buffer_size(raw_size, device)
    }

    fn buffer_size(raw_size: usize, device: &Device) -> u32 {
        let chunk_size = device.limits().min_storage_buffer_offset_alignment;
        let chunks = (raw_size as f32 / chunk_size as f32).ceil() as u32;
        chunks * chunk_size as u32
    }

    pub fn update_shader_config(&self, config: &mut ShaderParams) {
        config.shape_count = self.shapes.len() as u32;
        config.sphere_count = self.map[0].len() as u32;
        config.cube_count = self.map[1].len() as u32;
    }

    pub fn shape_count(&self) -> u32 {
        self.shapes.len() as u32
    }

    pub fn sphere_count(&self) -> u32 {
        self.indices[0]
    }

    pub fn get_shape_mut(&mut self, index: u32) -> Option<&mut Box<dyn Shape>> {
        self.shapes.get_mut(index as usize)
    }

    pub fn get_shape(&self, index: u32) -> Option<&Box<dyn Shape>> {
        self.shapes.get(index as usize)
    }

    pub fn get_sphere_mut(&mut self, index: u32) -> Option<&mut Sphere> {
        // A very elegant solution to my tangled mess of a data structure
        self.map[0]
            .get_mut(index as usize)
            .and_then(|a| self.shapes.get_mut(*a as usize))
            .map(|a| a.as_any_mut().downcast_mut::<Sphere>())
            .flatten()
    }

    pub fn get_sphere(&self, index: u32) -> Option<&Sphere> {
        self.map[0]
            .get(index as usize)
            .and_then(|a| self.shapes.get(*a as usize))
            .map(|a| a.as_any().downcast_ref::<Sphere>())
            .flatten()
    }

    pub fn get_cube_mut(&mut self, index: u32) -> Option<&mut Cube> {
        self.map[1]
            .get_mut(index as usize)
            .and_then(|a| self.shapes.get_mut(*a as usize))
            .map(|a| a.as_any_mut().downcast_mut::<Cube>())
            .flatten()
    }

    pub fn get_cube(&self, index: u32) -> Option<&Cube> {
        self.map[1]
            .get(index as usize)
            .and_then(|a| self.shapes.get(*a as usize))
            .map(|a| a.as_any().downcast_ref::<Cube>())
            .flatten()
    }
}
//#endregion
