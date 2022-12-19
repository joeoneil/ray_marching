use bytemuck::Contiguous;
use cgmath::{Quaternion, Vector3};
use std::any::Any;
use wgpu::Device;

use super::super::ShaderParams;

pub trait Shape {
    fn as_any(&self) -> &dyn Any;

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

static mut SPHERE_INDEX: u32 = 0;

pub struct Sphere {
    pos: Vector3<f32>,
    radius: f32,
    color: Vector3<f32>,
    index: u32,
}

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

impl Shape for Sphere {
    fn as_any(&self) -> &dyn Any {
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

pub struct ShapeManager {
    // Shæps
    shapes: Vec<Box<dyn Shape>>,
    sphere_index: u32,

    // Buffers
    pub(crate) shape_buffer: Option<wgpu::Buffer>,
    pub(crate) shape_bind_group: Option<wgpu::BindGroup>,
    pub(crate) sphere_buffer: Option<wgpu::Buffer>,
    pub(crate) sphere_bind_group: Option<wgpu::BindGroup>,
}

impl ShapeManager {
    pub fn new() -> Self {
        Self {
            shapes: vec![],
            sphere_index: 0,
            shape_buffer: None,
            shape_bind_group: None,
            sphere_buffer: None,
            sphere_bind_group: None,
        }
    }

    pub fn serialize_shapes(&self) -> Vec<u8> {
        self.shapes
            .iter()
            .flat_map(|a| -> Vec<u8> { bytemuck::cast_slice(&[a.shape_data()]).to_vec() })
            .collect()
    }

    pub fn serialize_spheres(&self) -> Vec<u8> {
        self.shapes
            .iter()
            .filter_map(|a| -> Option<&Sphere> { a.as_any().downcast_ref::<Sphere>() })
            .flat_map(|a| -> Vec<u8> { bytemuck::cast_slice(&[a.sphere_data()]).to_vec() })
            .collect()
    }

    pub fn new_sphere(&mut self, pos: Vector3<f32>, radius: f32, color: Vector3<f32>) -> &Sphere {
        self.shapes.push(Box::new(Sphere {
            pos,
            radius,
            color,
            index: self.sphere_index,
        }));
        self.sphere_index += 1;
        self.shapes
            .last()
            .unwrap()
            .as_any()
            .downcast_ref::<Sphere>()
            .unwrap()
    }

    pub fn shape_buffer_size(&self, device: &Device) -> u32 {
        let raw_size = std::mem::size_of::<ShapeData>() * self.shapes.len();
        let chunk_size = device.limits().min_storage_buffer_offset_alignment;
        let chunks = (raw_size as f32 / chunk_size as f32).ceil() as u32;
        chunks * chunk_size
    }

    pub fn sphere_buffer_size(&self, device: &Device) -> u32 {
        let raw_size = std::mem::size_of::<SphereData>() * self.sphere_index as usize; // sphere index is sphere count
        let chunk_size = device.limits().min_storage_buffer_offset_alignment;
        let chunks = (raw_size as f32 / chunk_size as f32).ceil() as u32;
        chunks * chunk_size
    }

    pub fn update_shader_config(&self, config: &mut ShaderParams) {
        config.shape_count = self.shapes.len() as u32;
        config.sphere_count = self.sphere_index;
    }

    pub fn shape_count(&self) -> u32 {
        self.shapes.len() as u32
    }

    pub fn sphere_count(&self) -> u32 {
        self.sphere_index
    }
}
