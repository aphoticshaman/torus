use glam::Vec3;

/// Dense 3D grid storing SDF values and material IDs.
/// Volume: [0, extent)³. Resolution: N³ cells.
pub struct VoxelGrid {
    pub n: usize,
    pub extent: f32,
    pub sdf: Vec<f32>,
    pub material: Vec<u8>,
}

impl VoxelGrid {
    pub fn new(n: usize, extent: f32) -> Self {
        let total = n * n * n;
        Self {
            n,
            extent,
            sdf: vec![f32::MAX; total],
            material: vec![0; total],
        }
    }

    /// Voxelize a scene defined by an SDF function.
    pub fn voxelize<F>(&mut self, sdf_fn: F)
    where
        F: Fn(Vec3) -> (f32, u8),
    {
        let cell_size = self.extent / self.n as f32;
        for iz in 0..self.n {
            for iy in 0..self.n {
                for ix in 0..self.n {
                    let p = Vec3::new(
                        (ix as f32 + 0.5) * cell_size,
                        (iy as f32 + 0.5) * cell_size,
                        (iz as f32 + 0.5) * cell_size,
                    );
                    let (d, mat) = sdf_fn(p);
                    let idx = self.index(ix, iy, iz);
                    self.sdf[idx] = d;
                    self.material[idx] = mat;
                }
            }
        }
    }

    #[inline]
    pub fn index(&self, x: usize, y: usize, z: usize) -> usize {
        z * self.n * self.n + y * self.n + x
    }

    #[inline]
    pub fn is_occupied(&self, x: usize, y: usize, z: usize) -> bool {
        self.sdf[self.index(x, y, z)] < 0.0
    }

    /// World position of voxel center.
    #[inline]
    pub fn cell_center(&self, x: usize, y: usize, z: usize) -> Vec3 {
        let s = self.extent / self.n as f32;
        Vec3::new((x as f32 + 0.5) * s, (y as f32 + 0.5) * s, (z as f32 + 0.5) * s)
    }

    /// World-space grid index from position (clamped).
    pub fn pos_to_idx(&self, p: Vec3) -> (usize, usize, usize) {
        let s = self.n as f32 / self.extent;
        let ix = ((p.x * s) as usize).min(self.n - 1);
        let iy = ((p.y * s) as usize).min(self.n - 1);
        let iz = ((p.z * s) as usize).min(self.n - 1);
        (ix, iy, iz)
    }

    /// SDF value at world position (nearest-voxel lookup).
    pub fn sdf_at(&self, p: Vec3) -> f32 {
        let (ix, iy, iz) = self.pos_to_idx(p);
        self.sdf[self.index(ix, iy, iz)]
    }

    /// Count occupied voxels with given material.
    pub fn count_occupied(&self, mat: u8) -> usize {
        self.sdf
            .iter()
            .zip(self.material.iter())
            .filter(|(&d, &m)| d < 0.0 && m == mat)
            .count()
    }

    /// Collect indices of occupied voxels (for topology computation).
    pub fn occupied_indices(&self, mat_filter: Option<u8>) -> Vec<(usize, usize, usize)> {
        let mut out = Vec::new();
        for iz in 0..self.n {
            for iy in 0..self.n {
                for ix in 0..self.n {
                    let idx = self.index(ix, iy, iz);
                    if self.sdf[idx] < 0.0 {
                        if let Some(mat) = mat_filter {
                            if self.material[idx] != mat {
                                continue;
                            }
                        }
                        out.push((ix, iy, iz));
                    }
                }
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::universe::sdf;

    #[test]
    fn voxelize_torus_has_occupied_cells() {
        let mut grid = VoxelGrid::new(64, 10.0);
        grid.voxelize(sdf::scene);
        let count = grid.count_occupied(1);
        assert!(count > 100, "torus should occupy many voxels, got {}", count);
        eprintln!("Torus voxels at 64³: {}", count);
    }

    #[test]
    fn voxel_center_roundtrips() {
        let grid = VoxelGrid::new(256, 10.0);
        let c = grid.cell_center(128, 128, 128);
        let (ix, iy, iz) = grid.pos_to_idx(c);
        assert_eq!((ix, iy, iz), (128, 128, 128));
    }
}
