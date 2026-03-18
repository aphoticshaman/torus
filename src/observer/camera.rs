use glam::Vec3;

/// Pinhole camera model.
pub struct Camera {
    pub position: Vec3,
    pub forward: Vec3,
    pub up: Vec3,
    pub right: Vec3,
    pub fov_rad: f32,
    pub width: u32,
    pub height: u32,
}

impl Camera {
    pub fn new(position: Vec3, look_at: Vec3, up_hint: Vec3, fov_deg: f32, width: u32, height: u32) -> Self {
        let forward = (look_at - position).normalize();
        let right = forward.cross(up_hint).normalize();
        let up = right.cross(forward).normalize();
        Self {
            position,
            forward,
            up,
            right,
            fov_rad: fov_deg.to_radians(),
            width,
            height,
        }
    }

    /// Generate ray direction for pixel (px, py).
    pub fn ray_direction(&self, px: u32, py: u32) -> Vec3 {
        let aspect = self.width as f32 / self.height as f32;
        let half_w = (self.fov_rad / 2.0).tan();
        let half_h = half_w / aspect;

        let u = (2.0 * (px as f32 + 0.5) / self.width as f32 - 1.0) * half_w;
        let v = (1.0 - 2.0 * (py as f32 + 0.5) / self.height as f32) * half_h;

        (self.forward + self.right * u + self.up * v).normalize()
    }

    /// Create stereo pair (left eye, right eye) with given interpupillary distance.
    pub fn stereo_pair(position: Vec3, look_at: Vec3, up_hint: Vec3, fov_deg: f32, width: u32, height: u32, ipd: f32) -> (Self, Self) {
        let forward = (look_at - position).normalize();
        let right = forward.cross(up_hint).normalize();

        let left_pos = position - right * (ipd / 2.0);
        let right_pos = position + right * (ipd / 2.0);

        (
            Camera::new(left_pos, look_at, up_hint, fov_deg, width, height),
            Camera::new(right_pos, look_at, up_hint, fov_deg, width, height),
        )
    }
}
