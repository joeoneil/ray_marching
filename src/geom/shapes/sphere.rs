use crate::geom::points::vec3::Vec3;
use crate::geom::shapes::Obj;

pub struct Sphere {
    center: Vec3,
    radius: f32,
}

impl Sphere {
    pub fn new(center: Vec3, radius: f32) -> Sphere {
        Sphere { center, radius }
    }

    pub fn get_center(&self) -> Vec3 {
        self.center
    }

    pub fn get_radius(&self) -> f32 {
        self.radius
    }
}

impl Default for Sphere {
    fn default() -> Self {
        Sphere::new(Vec3::default(), 1.0)
    }
}

impl Obj for Sphere {
    fn sdf(&self, sample_point: Vec3) -> f32 {
        (sample_point - self.center).length() - self.radius
    }
}
