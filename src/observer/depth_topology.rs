/// Sublevel-set persistent homology on a 2D depth map.
/// Detects H₁ features (holes) visible as depth discontinuities.
///
/// A hole in the depth map appears as a region of FAR pixels (high depth)
/// surrounded by NEAR pixels (low depth). In sublevel-set filtration of
/// the NEGATED depth map, holes appear as 1-cycles that persist.

use std::collections::HashMap;

/// Persistence pair: (birth, death) in filtration value.
#[derive(Debug, Clone, Copy)]
pub struct PersistencePair {
    pub birth: f32,
    pub death: f32,
    pub dimension: usize,
}

impl PersistencePair {
    pub fn persistence(&self) -> f32 {
        (self.death - self.birth).abs()
    }
}

/// Compute 0-dimensional persistent homology of a scalar field on a grid.
/// Uses union-find on sublevel sets.
/// Returns persistence pairs for connected components (β₀ features).
pub fn persistence_h0(values: &[f32], width: usize, height: usize) -> Vec<PersistencePair> {
    let n = width * height;
    assert_eq!(values.len(), n);

    // Sort pixels by value (ascending = sublevel set filtration)
    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by(|&a, &b| values[a].partial_cmp(&values[b]).unwrap());

    let mut parent = vec![usize::MAX; n]; // MAX = not yet born
    let mut rank_uf = vec![0u32; n];
    let mut birth = vec![0.0f32; n];
    let mut pairs = Vec::new();

    for &idx in &order {
        let val = values[idx];
        if !val.is_finite() {
            continue;
        }

        // Birth: this pixel enters the sublevel set
        parent[idx] = idx;
        birth[idx] = val;

        // Check 4-connected neighbors
        let x = idx % width;
        let y = idx / width;
        let neighbors = [
            if x > 0 { Some(idx - 1) } else { None },
            if x + 1 < width { Some(idx + 1) } else { None },
            if y > 0 { Some(idx - width) } else { None },
            if y + 1 < height { Some(idx + width) } else { None },
        ];

        for nb in neighbors.into_iter().flatten() {
            if parent[nb] == usize::MAX {
                continue; // neighbor not yet in sublevel set
            }

            let root_a = find(idx, &mut parent);
            let root_b = find(nb, &mut parent);

            if root_a != root_b {
                // Merge: younger component (higher birth) dies
                let (older, younger) = if birth[root_a] <= birth[root_b] {
                    (root_a, root_b)
                } else {
                    (root_b, root_a)
                };

                pairs.push(PersistencePair {
                    birth: birth[younger],
                    death: val,
                    dimension: 0,
                });

                // Union by rank
                if rank_uf[older] < rank_uf[younger] {
                    parent[older] = younger;
                    birth[younger] = birth[older]; // keep older birth
                } else {
                    parent[younger] = older;
                    if rank_uf[older] == rank_uf[younger] {
                        rank_uf[older] += 1;
                    }
                }
            }
        }
    }

    pairs
}

/// Estimate β₁ from a depth map using the Euler characteristic relation.
/// For a simplicial complex on a grid: χ = V - E + F = β₀ - β₁ + β₂
/// For a planar complex: β₂ = 0, so β₁ = β₀ - χ
///
/// We threshold the depth map at various levels and count connected
/// components of the complement (holes visible as depth discontinuities).
pub fn estimate_h1_from_depth(
    depth: &[f32],
    width: usize,
    height: usize,
    n_thresholds: usize,
) -> Vec<DepthTopologyResult> {
    let finite_depths: Vec<f32> = depth.iter().copied().filter(|d| d.is_finite()).collect();
    if finite_depths.is_empty() {
        return vec![];
    }

    let min_d = finite_depths.iter().copied().fold(f32::MAX, f32::min);
    let max_d = finite_depths.iter().copied().fold(f32::MIN, f32::max);

    let mut results = Vec::new();

    for i in 0..n_thresholds {
        let t = min_d + (max_d - min_d) * (i as f32 + 0.5) / n_thresholds as f32;

        // Binary image: 1 = "object" (depth < threshold), 0 = "hole" (depth >= threshold)
        let binary: Vec<bool> = depth.iter().map(|&d| d.is_finite() && d < t).collect();

        // Count connected components of the OBJECT (β₀ of foreground)
        let fg_components = count_components(&binary, width, height);

        // Count INTERIOR holes: background components NOT connected to the image border.
        // This is the correct topological measurement: a hole is a background region
        // completely surrounded by foreground.
        let bg: Vec<bool> = binary.iter().map(|&b| !b).collect();
        let interior_holes = count_interior_components(&bg, width, height);

        // β₁ of the foreground = number of interior holes
        let b1_estimate = interior_holes;

        let bg_components = count_components(&bg, width, height);
        results.push(DepthTopologyResult {
            threshold: t,
            fg_components,
            bg_components,
            b1_estimate,
        });
    }

    results
}

#[derive(Debug)]
pub struct DepthTopologyResult {
    pub threshold: f32,
    pub fg_components: usize,
    pub bg_components: usize,
    pub b1_estimate: usize,
}

/// Count INTERIOR connected components: components in binary image that
/// do NOT touch the image border. These are "holes" surrounded by the complement.
fn count_interior_components(binary: &[bool], width: usize, height: usize) -> usize {
    let n = width * height;
    let mut visited = vec![false; n];

    // First: flood-fill from all border pixels that are TRUE in binary.
    // These are "exterior" components.
    let mut border_queue: Vec<usize> = Vec::new();
    for x in 0..width {
        if binary[x] { border_queue.push(x); visited[x] = true; }
        let bot = (height - 1) * width + x;
        if binary[bot] { border_queue.push(bot); visited[bot] = true; }
    }
    for y in 1..height-1 {
        let left = y * width;
        let right = y * width + width - 1;
        if binary[left] { border_queue.push(left); visited[left] = true; }
        if binary[right] { border_queue.push(right); visited[right] = true; }
    }

    // BFS from border
    while let Some(idx) = border_queue.pop() {
        let x = idx % width;
        let y = idx / width;
        let neighbors = [
            if x > 0 { Some(idx - 1) } else { None },
            if x + 1 < width { Some(idx + 1) } else { None },
            if y > 0 { Some(idx - width) } else { None },
            if y + 1 < height { Some(idx + width) } else { None },
        ];
        for nb in neighbors.into_iter().flatten() {
            if binary[nb] && !visited[nb] {
                visited[nb] = true;
                border_queue.push(nb);
            }
        }
    }

    // Now count remaining unvisited TRUE components — these are interior
    let mut count = 0;
    for start in 0..n {
        if !binary[start] || visited[start] {
            continue;
        }
        count += 1;
        let mut queue = vec![start];
        visited[start] = true;
        while let Some(idx) = queue.pop() {
            let x = idx % width;
            let y = idx / width;
            let neighbors = [
                if x > 0 { Some(idx - 1) } else { None },
                if x + 1 < width { Some(idx + 1) } else { None },
                if y > 0 { Some(idx - width) } else { None },
                if y + 1 < height { Some(idx + width) } else { None },
            ];
            for nb in neighbors.into_iter().flatten() {
                if binary[nb] && !visited[nb] {
                    visited[nb] = true;
                    queue.push(nb);
                }
            }
        }
    }

    count
}

/// Count connected components in a binary image (4-connected).
fn count_components(binary: &[bool], width: usize, height: usize) -> usize {
    let n = width * height;
    let mut visited = vec![false; n];
    let mut count = 0;

    for start in 0..n {
        if !binary[start] || visited[start] {
            continue;
        }
        // BFS flood fill
        count += 1;
        let mut queue = vec![start];
        visited[start] = true;

        while let Some(idx) = queue.pop() {
            let x = idx % width;
            let y = idx / width;

            let neighbors = [
                if x > 0 { Some(idx - 1) } else { None },
                if x + 1 < width { Some(idx + 1) } else { None },
                if y > 0 { Some(idx - width) } else { None },
                if y + 1 < height { Some(idx + width) } else { None },
            ];

            for nb in neighbors.into_iter().flatten() {
                if binary[nb] && !visited[nb] {
                    visited[nb] = true;
                    queue.push(nb);
                }
            }
        }
    }

    count
}

fn find(x: usize, parent: &mut [usize]) -> usize {
    if parent[x] != x {
        parent[x] = find(parent[x], parent);
    }
    parent[x]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ring_in_depth_map() {
        // 10x10 depth map with a ring: near pixels surrounding far pixels
        let mut depth = vec![1.0f32; 100]; // background at distance 1

        // Object ring (near, distance 0.5)
        for x in 2..8 {
            depth[2 * 10 + x] = 0.5;  // top
            depth[7 * 10 + x] = 0.5;  // bottom
        }
        for y in 2..8 {
            depth[y * 10 + 2] = 0.5;  // left
            depth[y * 10 + 7] = 0.5;  // right
        }
        // Hole in the middle (far, distance 2.0 — seeing through to background)
        for y in 3..7 {
            for x in 3..7 {
                depth[y * 10 + x] = 2.0;
            }
        }

        let results = estimate_h1_from_depth(&depth, 10, 10, 5);
        let max_b1 = results.iter().map(|r| r.b1_estimate).max().unwrap_or(0);
        eprintln!("Ring depth map: max β₁ estimate = {}", max_b1);
        for r in &results {
            eprintln!("  t={:.2}: fg={}, bg={}, β₁≈{}", r.threshold, r.fg_components, r.bg_components, r.b1_estimate);
        }
        assert!(max_b1 >= 1, "should detect hole in depth map ring, got β₁={}", max_b1);
    }

    #[test]
    fn solid_disk_in_depth_map() {
        // Solid disk (no hole) — should have β₁ = 0
        let mut depth = vec![2.0f32; 100];
        for y in 2..8 {
            for x in 2..8 {
                depth[y * 10 + x] = 0.5;
            }
        }

        let results = estimate_h1_from_depth(&depth, 10, 10, 5);
        let max_b1 = results.iter().map(|r| r.b1_estimate).max().unwrap_or(0);
        assert_eq!(max_b1, 0, "solid disk should have β₁=0, got {}", max_b1);
    }
}
