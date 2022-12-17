use crate::geom::matrix::matrix3x3::Matrix3x3;
use crate::geom::points::vec2::Vec2;
use crate::geom::points::vec3::Vec3;

pub struct Camera {
    pub position: Vec3,
    camera_matrix: Matrix3x3,
    camera_matrix_inverse: Matrix3x3,
}

impl Camera {
    pub fn new(
        position: Vec3,
        image_width: u32,
        image_height: u32,
        h_fov: f32,
        v_fov: f32,
    ) -> Camera {
        let c_x = (image_width as f32) / 2.0;
        let c_y = (image_height as f32) / 2.0;
        let f_x = c_x / (h_fov / 2.0).tan();
        let f_y = c_y / (v_fov / 2.0).tan();
        let camera_matrix = Matrix3x3::new(
            Vec3::new(f_x, 0.0, c_x),
            Vec3::new(0.0, f_y, c_y),
            Vec3::new(0.0, 0.0, 1.0),
        );
        let camera_matrix_inverse = camera_matrix.inverse();
        Camera {
            position,
            camera_matrix,
            camera_matrix_inverse,
        }
    }

    pub fn get_screen_coords_from_world_pos(&self, point: Vec3) -> Vec2 {
        (self.camera_matrix * point).into()
    }

    pub fn get_world_pos_from_screen_coords(&self, point: Vec2) -> Vec3 {
        let point = Vec3::new(point.x, point.y, 1.0);
        self.camera_matrix_inverse * point
    }
}
