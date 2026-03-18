use glam::Vec3;
use rand::Rng;

use super::sdf;

/// A photon stored in the photon map.
#[derive(Clone, Copy)]
pub struct Photon {
    pub position: Vec3,
    pub direction: Vec3, // incoming direction
    pub power: Vec3,     // RGB power
}

/// Hash-grid accelerated photon map for fast nearest-neighbor gathering.
pub struct PhotonMap {
    photons: Vec<Photon>,
    grid: Vec<Vec<u32>>, // cell index → photon indices
    cell_size: f32,
    grid_n: usize,
    extent: f32,
}

impl PhotonMap {
    /// Emit photons from a point light and trace them through the scene.
    pub fn build(
        light_pos: Vec3,
        light_power: Vec3,
        n_photons: usize,
        extent: f32,
        max_bounces: usize,
    ) -> Self {
        let mut rng = rand::thread_rng();
        let photon_power = light_power / n_photons as f32;
        let mut photons = Vec::with_capacity(n_photons);

        for _ in 0..n_photons {
            // Uniform sphere sampling
            let dir = uniform_sphere(&mut rng);
            let mut ray_origin = light_pos;
            let mut ray_dir = dir;
            let mut power = photon_power;

            for _bounce in 0..max_bounces {
                if let Some((hit_pos, hit_normal, _mat)) =
                    trace_ray(ray_origin, ray_dir, extent, 0.001)
                {
                    photons.push(Photon {
                        position: hit_pos,
                        direction: ray_dir,
                        power,
                    });

                    // Diffuse bounce: cosine-weighted hemisphere
                    let new_dir = cosine_hemisphere(hit_normal, &mut rng);
                    ray_origin = hit_pos + hit_normal * 0.01;
                    ray_dir = new_dir;
                    power *= 0.7; // albedo
                } else {
                    break; // ray escaped the volume
                }
            }
        }

        let cell_size = 0.2; // 20cm cells
        let grid_n = (extent / cell_size).ceil() as usize;
        let total_cells = grid_n * grid_n * grid_n;
        let mut grid = vec![Vec::new(); total_cells];

        for (i, ph) in photons.iter().enumerate() {
            let ci = pos_to_cell(ph.position, cell_size, grid_n);
            if let Some(idx) = ci {
                grid[idx].push(i as u32);
            }
        }

        Self {
            photons,
            grid,
            cell_size,
            grid_n,
            extent,
        }
    }

    /// Gather radiance at a surface point using k-nearest photons in radius.
    pub fn gather(&self, pos: Vec3, normal: Vec3, radius: f32) -> Vec3 {
        let r2 = radius * radius;
        let mut power_sum = Vec3::ZERO;
        let mut count = 0u32;

        let cells = self.neighbor_cells(pos, radius);
        for cell_idx in cells {
            for &pi in &self.grid[cell_idx] {
                let ph = &self.photons[pi as usize];
                let d2 = (ph.position - pos).length_squared();
                if d2 < r2 {
                    // Only count photons on the same side of the surface
                    let cos_theta = (-ph.direction).dot(normal).max(0.0);
                    if cos_theta > 0.0 {
                        power_sum += ph.power * cos_theta;
                        count += 1;
                    }
                }
            }
        }

        if count > 0 {
            // Radiance estimate: total power / (pi * r²)
            power_sum / (std::f32::consts::PI * r2)
        } else {
            Vec3::ZERO
        }
    }

    fn neighbor_cells(&self, pos: Vec3, radius: f32) -> Vec<usize> {
        let n = self.grid_n;
        let s = self.cell_size;
        let lo = ((pos - Vec3::splat(radius)) / s).floor();
        let hi = ((pos + Vec3::splat(radius)) / s).ceil();
        let mut cells = Vec::new();

        let x0 = (lo.x as isize).max(0) as usize;
        let x1 = (hi.x as usize).min(n - 1);
        let y0 = (lo.y as isize).max(0) as usize;
        let y1 = (hi.y as usize).min(n - 1);
        let z0 = (lo.z as isize).max(0) as usize;
        let z1 = (hi.z as usize).min(n - 1);

        for iz in z0..=z1 {
            for iy in y0..=y1 {
                for ix in x0..=x1 {
                    cells.push(iz * n * n + iy * n + ix);
                }
            }
        }
        cells
    }

    pub fn photon_count(&self) -> usize {
        self.photons.len()
    }

    pub fn photon_positions(&self) -> Vec<Vec3> {
        self.photons.iter().map(|p| p.position).collect()
    }
}

/// Sphere-trace a ray through the scene SDF.
pub fn trace_ray(origin: Vec3, dir: Vec3, extent: f32, epsilon: f32) -> Option<(Vec3, Vec3, u8)> {
    let mut t = 0.0f32;
    let max_t = extent * 2.0;
    let max_steps = 256;

    for _ in 0..max_steps {
        let p = origin + dir * t;
        // Check bounds: ray has left the simulation volume.
        // Use generous bounds to allow observers positioned outside the volume
        // to cast rays toward objects inside it.
        let margin = extent;
        if p.x < -margin || p.x > extent + margin
            || p.y < -margin || p.y > extent + margin
            || p.z < -margin || p.z > extent + margin
        {
            return None;
        }

        let (d, mat) = sdf::scene(p);
        if d < epsilon {
            let normal = sdf::gradient(p, 0.001);
            return Some((p, normal, mat));
        }
        if t > max_t {
            return None;
        }
        t += d.max(epsilon); // clamp step to avoid zero-step loops
    }
    None
}

fn uniform_sphere(rng: &mut impl Rng) -> Vec3 {
    let z: f32 = rng.gen_range(-1.0..1.0);
    let phi: f32 = rng.gen_range(0.0..std::f32::consts::TAU);
    let r = (1.0 - z * z).sqrt();
    Vec3::new(r * phi.cos(), r * phi.sin(), z)
}

fn cosine_hemisphere(normal: Vec3, rng: &mut impl Rng) -> Vec3 {
    let r1: f32 = rng.gen();
    let r2: f32 = rng.gen();
    let phi = std::f32::consts::TAU * r1;
    let cos_theta = r2.sqrt();
    let sin_theta = (1.0 - r2).sqrt();

    // Build local frame
    let w = normal;
    let u = if w.x.abs() > 0.9 {
        Vec3::Y.cross(w).normalize()
    } else {
        Vec3::X.cross(w).normalize()
    };
    let v = w.cross(u);

    (u * phi.cos() * sin_theta + v * phi.sin() * sin_theta + w * cos_theta).normalize()
}

fn pos_to_cell(p: Vec3, cell_size: f32, n: usize) -> Option<usize> {
    let ix = (p.x / cell_size) as usize;
    let iy = (p.y / cell_size) as usize;
    let iz = (p.z / cell_size) as usize;
    if ix < n && iy < n && iz < n {
        Some(iz * n * n + iy * n + ix)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn photon_map_has_photons_on_torus() {
        let map = PhotonMap::build(
            Vec3::new(5.0, 5.0, 8.0), // light above torus
            Vec3::splat(100.0),
            100_000, // 100K for fast test
            10.0,
            2,
        );
        assert!(
            map.photon_count() > 1000,
            "should have many stored photons, got {}",
            map.photon_count()
        );
        eprintln!("Photons stored: {}", map.photon_count());
    }

    #[test]
    fn trace_ray_through_hole_misses_torus() {
        // Ray from (5,5,0.5) along +Z goes straight through the torus hole.
        // It should NOT hit the torus (mat=1). It may hit the floor or escape.
        let result = trace_ray(Vec3::new(5.0, 5.0, 0.5), Vec3::Z, 10.0, 0.001);
        if let Some((_, _, mat)) = result {
            assert_ne!(mat, 1, "ray through hole should NOT hit torus");
        }
        // Either misses entirely or hits floor — both valid
    }

    #[test]
    fn trace_ray_hits_torus_from_side() {
        // Ray from side should hit torus surface
        let result = trace_ray(Vec3::new(0.0, 5.0, 5.0), Vec3::X, 10.0, 0.001);
        assert!(result.is_some(), "ray along X toward torus should hit");
        let (pos, _, mat) = result.unwrap();
        assert_eq!(mat, 1, "should hit torus material");
        eprintln!("Hit torus at {:?}", pos);
    }
}
