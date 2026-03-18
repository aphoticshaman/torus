use glam::Vec3;

/// Signed distance to a torus centered at `center` with symmetry axis along Z.
/// Hole runs along the Z axis. Ring lies in the XY plane.
pub fn torus(p: Vec3, center: Vec3, major_r: f32, minor_r: f32) -> f32 {
    let q = p - center;
    let d_ring = (q.x * q.x + q.y * q.y).sqrt() - major_r;
    (d_ring * d_ring + q.z * q.z).sqrt() - minor_r
}

/// Signed distance to a plane defined by normal·p + d = 0.
/// Negative below the plane.
pub fn plane(p: Vec3, normal: Vec3, d: f32) -> f32 {
    p.dot(normal) + d
}

/// Union of two SDFs (min).
pub fn union(a: f32, b: f32) -> f32 {
    a.min(b)
}

/// The MVP scene: torus only (floor removed for topology experiment).
pub fn scene(p: Vec3) -> (f32, u8) {
    let d_torus = torus(p, Vec3::new(5.0, 5.0, 5.0), 2.0, 0.5);
    (d_torus, 1)
}

/// SDF gradient (surface normal) via central differences.
pub fn gradient(p: Vec3, eps: f32) -> Vec3 {
    let dx = scene(p + Vec3::X * eps).0 - scene(p - Vec3::X * eps).0;
    let dy = scene(p + Vec3::Y * eps).0 - scene(p - Vec3::Y * eps).0;
    let dz = scene(p + Vec3::Z * eps).0 - scene(p - Vec3::Z * eps).0;
    Vec3::new(dx, dy, dz).normalize_or_zero()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn torus_center_is_inside() {
        // Point on the ring (in XY plane at distance R from center) should be inside
        let p = Vec3::new(5.0 + 2.0, 5.0, 5.0); // on the ring
        assert!(torus(p, Vec3::new(5.0, 5.0, 5.0), 2.0, 0.5) < 0.0);
    }

    #[test]
    fn torus_far_point_is_outside() {
        let p = Vec3::new(0.0, 0.0, 0.0);
        assert!(torus(p, Vec3::new(5.0, 5.0, 5.0), 2.0, 0.5) > 0.0);
    }

    #[test]
    fn torus_hole_center_is_outside() {
        // The center of the torus (inside the hole) should be outside
        let p = Vec3::new(5.0, 5.0, 5.0);
        // Distance from ring = R = 2.0, distance = 2.0 - 0.5 = 1.5
        assert!(torus(p, Vec3::new(5.0, 5.0, 5.0), 2.0, 0.5) > 0.0);
    }

    #[test]
    fn floor_above_is_positive() {
        assert!(plane(Vec3::new(0.0, 0.0, 1.0), Vec3::Z, 0.0) > 0.0);
    }

    #[test]
    fn floor_below_is_negative() {
        assert!(plane(Vec3::new(0.0, 0.0, -1.0), Vec3::Z, 0.0) < 0.0);
    }

    #[test]
    fn scene_classifies_1000_points() {
        let mut inside_torus = 0;
        let mut inside_floor = 0;
        for i in 0..10 {
            for j in 0..10 {
                for k in 0..10 {
                    let p = Vec3::new(i as f32, j as f32, k as f32);
                    let (d, mat) = scene(p);
                    if d < 0.0 {
                        match mat {
                            1 => inside_torus += 1,
                            2 => inside_floor += 1,
                            _ => {}
                        }
                    }
                }
            }
        }
        assert!(inside_torus > 0, "some points should be inside torus");
    }
}
