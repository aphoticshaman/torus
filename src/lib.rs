pub mod api;
pub mod universe;
pub mod observer;

use glam::Vec3;
use api::{Universe, RaySample, HomologyGroups};
use universe::{photon, grid, topology};

/// System 1 implementation. Owns the geometry and light field.
/// Implements the Universe trait — the ONLY interface System 2 may use.
pub struct System1 {
    pub grid: grid::VoxelGrid,
    pub photon_map: photon::PhotonMap,
}

impl System1 {
    pub fn build(resolution: usize, n_photons: usize) -> Self {
        let extent = 10.0;

        eprintln!("[System1] Voxelizing scene at {}³...", resolution);
        let mut grid = grid::VoxelGrid::new(resolution, extent);
        grid.voxelize(universe::sdf::scene);
        let torus_voxels = grid.count_occupied(1);
        let floor_voxels = grid.count_occupied(2);
        eprintln!("[System1] Torus voxels: {}, Floor voxels: {}", torus_voxels, floor_voxels);

        eprintln!("[System1] Building photon map ({} photons)...", n_photons);
        let light_pos = Vec3::new(5.0, 5.0, 8.0);
        let light_power = Vec3::splat(200.0);
        let photon_map = photon::PhotonMap::build(light_pos, light_power, n_photons, extent, 3);
        eprintln!("[System1] Photon map: {} stored photons", photon_map.photon_count());

        Self { grid, photon_map }
    }
}

impl Universe for System1 {
    fn query_ray(&self, origin: Vec3, direction: Vec3) -> RaySample {
        match photon::trace_ray(origin, direction, self.grid.extent, 0.002) {
            Some((hit_pos, normal, mat)) => {
                let radiance = self.photon_map.gather(hit_pos, normal, 0.3);
                RaySample {
                    distance: (hit_pos - origin).length(),
                    normal,
                    radiance,
                    material: mat,
                    hit: true,
                }
            }
            None => RaySample::miss(),
        }
    }

    fn query_topology(&self, min: Vec3, max: Vec3) -> HomologyGroups {
        // Collect occupied torus voxels within the bounding box
        let occupied: Vec<(usize, usize, usize)> = self.grid.occupied_indices(Some(1))
            .into_iter()
            .filter(|&(x, y, z)| {
                let c = self.grid.cell_center(x, y, z);
                c.x >= min.x && c.x <= max.x
                    && c.y >= min.y && c.y <= max.y
                    && c.z >= min.z && c.z <= max.z
            })
            .collect();

        if occupied.is_empty() {
            return HomologyGroups {
                beta: vec![0, 0, 0],
                d_squared_zero: true,
            };
        }

        let (b0, b1, b2) = topology::betti_numbers(&occupied);
        HomologyGroups {
            beta: vec![b0, b1, b2],
            d_squared_zero: true, // verified inside betti_numbers
        }
    }
}
