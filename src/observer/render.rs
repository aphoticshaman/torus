use glam::Vec3;
use crate::api::Universe;
use super::camera::Camera;

/// Render a camera view by querying System 1 through the Universe trait.
/// Returns (depth_map, radiance_image).
pub fn render_camera<U: Universe>(universe: &U, camera: &Camera) -> (Vec<f32>, Vec<Vec3>) {
    let npix = (camera.width * camera.height) as usize;
    let mut depth = vec![f32::MAX; npix];
    let mut image = vec![Vec3::ZERO; npix];

    for py in 0..camera.height {
        for px in 0..camera.width {
            let dir = camera.ray_direction(px, py);
            let sample = universe.query_ray(camera.position, dir);
            let idx = (py * camera.width + px) as usize;

            depth[idx] = sample.distance;
            image[idx] = sample.radiance;
        }
    }

    (depth, image)
}

/// Save radiance image as PNG (raw HDR → clamped 8-bit).
pub fn save_image(path: &str, image: &[Vec3], width: u32, height: u32) {
    let mut buf = vec![0u8; (width * height * 3) as usize];
    for (i, &rad) in image.iter().enumerate() {
        // Simple tone mapping: clamp and gamma
        let r = (rad.x.sqrt().clamp(0.0, 1.0) * 255.0) as u8;
        let g = (rad.y.sqrt().clamp(0.0, 1.0) * 255.0) as u8;
        let b = (rad.z.sqrt().clamp(0.0, 1.0) * 255.0) as u8;
        buf[i * 3] = r;
        buf[i * 3 + 1] = g;
        buf[i * 3 + 2] = b;
    }

    image::save_buffer(path, &buf, width, height, image::ColorType::Rgb8)
        .expect("failed to save image");
}

/// Save depth map as grayscale PNG (normalized to [0,255]).
pub fn save_depth(path: &str, depth: &[f32], width: u32, height: u32) {
    let max_d = depth.iter().copied().filter(|d| d.is_finite()).fold(0.0f32, f32::max);
    let buf: Vec<u8> = depth
        .iter()
        .map(|&d| {
            if d.is_finite() && max_d > 0.0 {
                (255.0 * (1.0 - d / max_d)) as u8
            } else {
                0
            }
        })
        .collect();

    image::save_buffer(path, &buf, width, height, image::ColorType::L8)
        .expect("failed to save depth");
}
