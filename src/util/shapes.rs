use bytemuck::Contiguous;
use cgmath::{Quaternion, Rotation3, Vector3};
use std::any::Any;
use wgpu::Device;

use super::super::ShaderParams;

//#region Shape
pub trait Shape {
    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn shape_data(&self) -> ShapeData;

    fn translate(&mut self, translation: Vector3<f32>);

    fn set_pos(&mut self, pos: Vector3<f32>);

    fn rotate(&mut self, rotation: Quaternion<f32>);

    fn set_rotation(&mut self, rotation: Quaternion<f32>);

    fn get_index(&self) -> u32;
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ShapeData {
    color: [f32; 4], // align of vec_3 is 16 bytes on GPU
    index: u32,
    shape_type: u32,
    _padding: [f32; 2], // aligns total size to align of largest element (vec3, 16 bytes)
}

impl Default for ShapeData {
    fn default() -> Self {
        Self {
            color: [0.0, 0.0, 0.0, 0.0],
            index: u32::MAX_VALUE,
            shape_type: u32::MAX_VALUE,
            _padding: [0.0, 0.0],
        }
    }
}
//#endregion

pub struct Sphere {
    pos: Vector3<f32>,
    radius: f32,
    color: Vector3<f32>,
    index: u32,
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

    fn shape_data(&self) -> ShapeData {
        ShapeData {
            color: [self.color.x, self.color.y, self.color.z, 0.0],
            index: self.index,
            shape_type: 0,
            _padding: [0.0, 0.0],
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
}
//#endregion

//#region Cube
pub struct Cube {
    pos: Vector3<f32>,
    _p1: f32, // padding (vec3 is 16 bytes on GPU)
    bounds: Vector3<f32>,
    _p2: f32, // padding (vec3 is 16 bytes on GPU)
    rot: Quaternion<f32>,
    color: Vector3<f32>,
    index: u32,
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
            _p1: 0.0,
            bounds: Vector3::new(1.0, 1.0, 1.0),
            _p2: 0.0,
            rot: Quaternion::new(1.0, 0.0, 0.0, 0.0),
            color: Vector3::new(0.0, 0.0, 0.0),
            index: u32::MAX_VALUE,
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

    fn shape_data(&self) -> ShapeData {
        ShapeData {
            color: [self.color.x, self.color.y, self.color.z, 0.0],
            index: self.index,
            shape_type: 1,
            _padding: [0.0, 0.0],
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

    pub fn serialize_shapes(&self) -> Vec<u8> {
        if self.shapes.len() == 0 {
            return bytemuck::cast_slice(&[ShapeData::default()]).to_vec();
        }
        self.shapes
            .iter()
            .flat_map(|a| -> Vec<u8> { bytemuck::cast_slice(&[a.shape_data()]).to_vec() })
            .collect()
    }

    pub fn serialize_spheres(&self) -> Vec<u8> {
        if self.map[0].is_empty() {
            return bytemuck::cast_slice(&[Sphere::default().sphere_data()])
                .to_vec();
        }
        self.shapes
            .iter()
            .filter_map(|a| -> Option<&Sphere> { a.as_any().downcast_ref::<Sphere>() })
            .flat_map(|a| -> Vec<u8> { bytemuck::cast_slice(&[a.sphere_data()]).to_vec() })
            .collect()
    }

    pub fn serialize_cubes(&self) -> Vec<u8> {
        if self.map[1].is_empty() {
            return bytemuck::cast_slice(&[Cube::default().cube_data()])
                .to_vec();
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

    pub fn new_sphere(&mut self, pos: Vector3<f32>, radius: f32, color: Vector3<f32>) -> &mut Sphere {
        self.map[0].push(self.shapes.len() as u32);
        self.shapes.push(Box::new(Sphere {
            pos,
            radius,
            color,
            index: self.indices[0],
        }));
        self.indices[0] += 1;
        self.shapes
            .last_mut()
            .unwrap()
            .as_any_mut()
            .downcast_mut::<Sphere>()
            .unwrap()
    }

    pub fn new_cube(&mut self, pos: Vector3<f32>, bounds: Vector3<f32>, color: Vector3<f32>) -> &mut Cube {
        self.map[1].push(self.shapes.len() as u32);
        self.shapes.push(Box::new(Cube {
            pos,
            _p1: 0.0,
            bounds,
            _p2: 0.0,
            rot: Quaternion::from_angle_z(cgmath::Rad(0.0)),
            color,
            index: self.indices[1],
        }));
        self.indices[1] += 1;
        self.shapes
            .last_mut()
            .unwrap()
            .as_any_mut()
            .downcast_mut::<Cube>()
            .unwrap()
    }

    pub fn shape_buffer_size(&self, device: &Device) -> u32 {
        let raw_size = std::mem::size_of::<ShapeData>() * self.shapes.len();
        let chunk_size = device.limits().min_storage_buffer_offset_alignment;
        let chunks = (raw_size as f32 / chunk_size as f32).ceil() as u32;
        chunks * chunk_size
    }

    pub fn sphere_buffer_size(&self, device: &Device) -> u32 {
        let raw_size = std::mem::size_of::<SphereData>() * self.map[0].len() as usize; // sphere index is sphere count
        let chunk_size = device.limits().min_storage_buffer_offset_alignment;
        let chunks = (raw_size as f32 / chunk_size as f32).ceil() as u32;
        chunks * chunk_size
    }

    pub fn cube_buffer_size(&self, device: &Device) -> u32 {
        let raw_size = std::mem::size_of::<CubeData>() * self.map[1].len() as usize; // cube index is cube count
        let chunk_size = device.limits().min_storage_buffer_offset_alignment;
        let chunks = (raw_size as f32 / chunk_size as f32).ceil() as u32;
        chunks * chunk_size
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

    pub fn get_sphere_mut(&mut self, index: u32) -> Option<&mut Sphere> {
        // A very elegant solution to my tangled mess of a data structure
        self.map[0]
            .get_mut(index as usize)
            .and_then(|a| self.shapes.get_mut(*a as usize))
            .map(|a| a.as_any_mut().downcast_mut::<Sphere>())
            .flatten()
    }

    pub fn get_cube_mut(&mut self, index: u32) -> Option<&mut Cube> {
        self.map[1]
            .get_mut(index as usize)
            .and_then(|a| self.shapes.get_mut(*a as usize))
            .map(|a| a.as_any_mut().downcast_mut::<Cube>())
            .flatten()
    }
}
//#endregion
