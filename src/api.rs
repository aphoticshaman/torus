use glam::Vec3;

/// Radiance sample returned by a ray query.
#[derive(Clone, Copy, Debug)]
pub struct RaySample {
    pub distance: f32,
    pub normal: Vec3,
    pub radiance: Vec3, // RGB
    pub material: u8,
    pub hit: bool,
}

impl RaySample {
    pub fn miss() -> Self {
        Self {
            distance: f32::MAX,
            normal: Vec3::ZERO,
            radiance: Vec3::ZERO,
            material: 0,
            hit: false,
        }
    }
}

/// Homology groups (Betti numbers).
#[derive(Clone, Debug)]
pub struct HomologyGroups {
    pub beta: Vec<usize>, // β₀, β₁, β₂, ...
    pub d_squared_zero: bool,
}

/// THE BOUNDARY. System 2 communicates with System 1 ONLY through this trait.
/// System 1 does not know System 2 exists.
pub trait Universe {
    /// Cast a ray, return radiance at first intersection.
    fn query_ray(&self, origin: Vec3, direction: Vec3) -> RaySample;

    /// Compute homology of geometry within a bounding box.
    fn query_topology(&self, min: Vec3, max: Vec3) -> HomologyGroups;
}
