use crate::geom::points::vec3::Vec3;

pub mod sphere;

pub trait Obj {
    fn sdf(&self, sample_point: Vec3) -> f32;
}
