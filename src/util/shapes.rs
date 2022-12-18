use bytemuck::Contiguous;
use cgmath::{Quaternion, Vector3};

pub trait Shape {
    fn shape_data(&self) -> ShapeData;

    fn translate(&mut self, translation: Vector3<f32>);

    fn set_pos(&mut self, pos: Vector3<f32>);

    fn rotate(&mut self, rotation: Quaternion<f32>);

    fn set_rotation(&mut self, rotation: Quaternion<f32>);

    fn get_index(&self) -> u16;
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ShapeData {
    // 20 bytes per shape
    color: [f32; 3],
    index: u32,
    shape_type: u32,
}

impl Default for ShapeData {
    fn default() -> Self {
        Self {
            color: [0.0, 0.0, 0.0],
            index: u32::MAX_VALUE,
            shape_type: u32::MAX_VALUE,
        }
    }
}

static mut SPHERE_INDEX: u32 = 0;

struct Sphere {
    pos: Vector3<f32>,
    radius: f32,
    color: Vector3<f32>,
    index: u32,
}

impl Sphere {
    const TYPE: u32 = 0;

    pub fn new(pos: Vector3<f32>, radius: f32, color: Vector3<f32>) -> Self {
        // This is guaranteed to be safe so long as Sphere is never constructed
        // using the struct definition, which is enforced by the fact it is not
        // public outside this module. This function should be wrapped by a
        // manager which ensures that the sphere's internal index is identical
        // to its position in the storage buffer for spheres, otherwise undefined
        // behavior will occur in the fragment shader, which cannot be checked by
        // the rust compiler.
        unsafe {
            SPHERE_INDEX += 1;
            Self {
                pos,
                radius,
                color,
                index: SPHERE_INDEX - 1,
            }
        }
    }
}

impl Shape for Sphere {
    fn shape_data(&self) -> ShapeData {
        ShapeData {
            color: self.color.into(),
            index: self.index,
            shape_type: Self::TYPE,
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

    fn get_index(&self) -> u16 {
        todo!()
    }
}
