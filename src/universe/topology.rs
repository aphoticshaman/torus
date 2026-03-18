use std::collections::HashMap;

/// Cell types in a cubical complex, keyed by (position, direction).
/// Over Z₂ (mod 2) — sufficient for torus (no torsion).
///
/// 0-cell: vertex at (x,y,z)
/// 1-cell: edge at (x,y,z) in direction d ∈ {0=X, 1=Y, 2=Z}
///   connects vertex(x,y,z) to vertex(x+δ_d)
/// 2-cell: face at (x,y,z) in plane (d1,d2) with d1<d2
///   {XY=01, XZ=02, YZ=12}
/// 3-cell: cube at (x,y,z)

type V3 = (usize, usize, usize);

/// Compute Betti numbers β₀, β₁, β₂ of a cubical complex defined by
/// a set of occupied voxel coordinates. Uses column reduction over Z₂.
pub fn betti_numbers(occupied: &[V3]) -> (usize, usize, usize) {
    // Enumerate ALL cells of the cubical complex
    let mut verts: HashMap<V3, usize> = HashMap::new();
    let mut edges: HashMap<(V3, u8), usize> = HashMap::new(); // (pos, dir)
    let mut faces: HashMap<(V3, u8, u8), usize> = HashMap::new(); // (pos, d1, d2)
    let mut cubes: Vec<V3> = Vec::new();

    for &(x, y, z) in occupied {
        cubes.push((x, y, z));

        // 8 vertices
        for dz in 0..2usize {
            for dy in 0..2usize {
                for dx in 0..2usize {
                    let v = (x + dx, y + dy, z + dz);
                    let n = verts.len();
                    verts.entry(v).or_insert(n);
                }
            }
        }

        // 12 edges: 4 in each direction
        // X-edges at (x, y+dy, z+dz) for dy,dz in {0,1}
        for dz in 0..2usize {
            for dy in 0..2usize {
                let key = ((x, y + dy, z + dz), 0u8);
                let n = edges.len();
                edges.entry(key).or_insert(n);
            }
        }
        // Y-edges at (x+dx, y, z+dz)
        for dz in 0..2usize {
            for dx in 0..2usize {
                let key = ((x + dx, y, z + dz), 1u8);
                let n = edges.len();
                edges.entry(key).or_insert(n);
            }
        }
        // Z-edges at (x+dx, y+dy, z)
        for dy in 0..2usize {
            for dx in 0..2usize {
                let key = ((x + dx, y + dy, z), 2u8);
                let n = edges.len();
                edges.entry(key).or_insert(n);
            }
        }

        // 6 faces: 2 per plane orientation
        // XY-faces at (x, y, z+dz) for dz in {0,1}
        for dz in 0..2usize {
            let key = ((x, y, z + dz), 0, 1);
            let n = faces.len();
            faces.entry(key).or_insert(n);
        }
        // XZ-faces at (x, y+dy, z)
        for dy in 0..2usize {
            let key = ((x, y + dy, z), 0, 2);
            let n = faces.len();
            faces.entry(key).or_insert(n);
        }
        // YZ-faces at (x+dx, y, z)
        for dx in 0..2usize {
            let key = ((x + dx, y, z), 1, 2);
            let n = faces.len();
            faces.entry(key).or_insert(n);
        }
    }

    let n0 = verts.len();
    let n1 = edges.len();
    let n2 = faces.len();
    let n3 = cubes.len();
    eprintln!("Cubical complex: {} vertices, {} edges, {} faces, {} cubes", n0, n1, n2, n3);

    // ∂₁: edges → vertices (over Z₂)
    let mut d1_cols: Vec<Vec<usize>> = vec![Vec::new(); n1];
    for (&((x, y, z), dir), &ei) in &edges {
        let (dx, dy, dz) = match dir {
            0 => (1, 0, 0),
            1 => (0, 1, 0),
            2 => (0, 0, 1),
            _ => unreachable!(),
        };
        let v0 = verts[&(x, y, z)];
        let v1 = verts[&(x + dx, y + dy, z + dz)];
        let mut col = vec![v0, v1];
        col.sort();
        d1_cols[ei] = col;
    }

    // ∂₂: faces → edges (over Z₂)
    // ∂(xy-face at p) = x-edge(p) + x-edge(p+Y) + y-edge(p) + y-edge(p+X)
    // ∂(xz-face at p) = x-edge(p) + x-edge(p+Z) + z-edge(p) + z-edge(p+X)
    // ∂(yz-face at p) = y-edge(p) + y-edge(p+Z) + z-edge(p) + z-edge(p+Y)
    let mut d2_cols: Vec<Vec<usize>> = vec![Vec::new(); n2];
    for (&((x, y, z), d1, d2), &fi) in &faces {
        let boundary_edges: Vec<(V3, u8)> = match (d1, d2) {
            (0, 1) => vec![ // XY face
                ((x, y, z), 0),     // x-edge at p
                ((x, y + 1, z), 0), // x-edge at p+Y
                ((x, y, z), 1),     // y-edge at p
                ((x + 1, y, z), 1), // y-edge at p+X
            ],
            (0, 2) => vec![ // XZ face
                ((x, y, z), 0),     // x-edge at p
                ((x, y, z + 1), 0), // x-edge at p+Z
                ((x, y, z), 2),     // z-edge at p
                ((x + 1, y, z), 2), // z-edge at p+X
            ],
            (1, 2) => vec![ // YZ face
                ((x, y, z), 1),     // y-edge at p
                ((x, y, z + 1), 1), // y-edge at p+Z
                ((x, y, z), 2),     // z-edge at p
                ((x, y + 1, z), 2), // z-edge at p+Y
            ],
            _ => unreachable!(),
        };

        let mut col: Vec<usize> = boundary_edges
            .iter()
            .filter_map(|key| edges.get(key).copied())
            .collect();
        col.sort();
        d2_cols[fi] = col;
    }

    // ∂₃: cubes → faces (over Z₂)
    // ∂(cube at p) = xy-face(p) + xy-face(p+Z)
    //              + xz-face(p) + xz-face(p+Y)
    //              + yz-face(p) + yz-face(p+X)
    let mut d3_cols: Vec<Vec<usize>> = vec![Vec::new(); n3];
    for (ci, &(x, y, z)) in cubes.iter().enumerate() {
        let boundary_faces: Vec<(V3, u8, u8)> = vec![
            ((x, y, z), 0, 1),     // xy-face at p
            ((x, y, z + 1), 0, 1), // xy-face at p+Z
            ((x, y, z), 0, 2),     // xz-face at p
            ((x, y + 1, z), 0, 2), // xz-face at p+Y
            ((x, y, z), 1, 2),     // yz-face at p
            ((x + 1, y, z), 1, 2), // yz-face at p+X
        ];

        let mut col: Vec<usize> = boundary_faces
            .iter()
            .filter_map(|key| faces.get(key).copied())
            .collect();
        col.sort();
        d3_cols[ci] = col;
    }

    // Verify d² = 0
    let d2_ok = verify_d_squared_zero(&d1_cols, &d2_cols);
    let d3_ok = verify_d_squared_zero(&d2_cols, &d3_cols);
    eprintln!("d² = 0 check (∂₁∘∂₂): {}", if d2_ok { "VERIFIED ✓" } else { "FAILED ✗" });
    eprintln!("d² = 0 check (∂₂∘∂₃): {}", if d3_ok { "VERIFIED ✓" } else { "FAILED ✗" });

    // Column reduction over Z₂ for ranks
    let rank1 = column_reduce_rank_z2(&d1_cols, n0);
    let rank2 = column_reduce_rank_z2(&d2_cols, n1);
    let rank3 = column_reduce_rank_z2(&d3_cols, n2);

    eprintln!("Ranks: rank(∂₁)={}, rank(∂₂)={}, rank(∂₃)={}", rank1, rank2, rank3);

    let b0 = n0 - rank1;
    let b1 = (n1 - rank1) - rank2;
    let b2 = (n2 - rank2) - rank3;

    eprintln!("Betti: β₀={}, β₁={}, β₂={}", b0, b1, b2);

    (b0, b1, b2)
}

/// Column reduction over Z₂. Returns the rank of the matrix.
fn column_reduce_rank_z2(cols: &[Vec<usize>], _n_rows: usize) -> usize {
    let n_cols = cols.len();
    let mut work: Vec<Vec<usize>> = cols.to_vec();
    let mut pivot_col: HashMap<usize, usize> = HashMap::new();
    let mut rank = 0;

    for j in 0..n_cols {
        loop {
            if work[j].is_empty() {
                break;
            }
            let pivot = *work[j].last().unwrap();
            if let Some(&k) = pivot_col.get(&pivot) {
                let col_k = work[k].clone();
                xor_sorted(&mut work[j], &col_k);
            } else {
                pivot_col.insert(pivot, j);
                rank += 1;
                break;
            }
        }
    }
    rank
}

/// XOR two sorted vectors (symmetric difference) in place.
fn xor_sorted(a: &mut Vec<usize>, b: &[usize]) {
    let mut result = Vec::with_capacity(a.len() + b.len());
    let (mut i, mut j) = (0, 0);
    while i < a.len() && j < b.len() {
        if a[i] < b[j] {
            result.push(a[i]);
            i += 1;
        } else if a[i] > b[j] {
            result.push(b[j]);
            j += 1;
        } else {
            i += 1;
            j += 1;
        }
    }
    result.extend_from_slice(&a[i..]);
    result.extend_from_slice(&b[j..]);
    *a = result;
}

/// Verify d² = 0 by checking ∂_lower ∘ ∂_upper = 0 (over Z₂).
fn verify_d_squared_zero(d_lower: &[Vec<usize>], d_upper: &[Vec<usize>]) -> bool {
    for col_upper in d_upper {
        let mut result: Vec<usize> = Vec::new();
        for &idx in col_upper {
            if idx < d_lower.len() {
                xor_sorted(&mut result, &d_lower[idx]);
            }
        }
        if !result.is_empty() {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_cube_betti() {
        let occ = vec![(0, 0, 0)];
        let (b0, b1, b2) = betti_numbers(&occ);
        assert_eq!(b0, 1, "single cube: β₀ should be 1");
        assert_eq!(b1, 0, "single cube: β₁ should be 0");
        assert_eq!(b2, 0, "single cube: β₂ should be 0");
    }

    #[test]
    fn two_disconnected_cubes_betti() {
        let occ = vec![(0, 0, 0), (5, 5, 5)];
        let (b0, b1, _) = betti_numbers(&occ);
        assert_eq!(b0, 2, "two disconnected: β₀ should be 2");
        assert_eq!(b1, 0, "two disconnected: β₁ should be 0");
    }

    #[test]
    fn line_of_cubes_betti() {
        let occ = vec![(0, 0, 0), (1, 0, 0), (2, 0, 0)];
        let (b0, b1, _) = betti_numbers(&occ);
        assert_eq!(b0, 1, "line: β₀ should be 1");
        assert_eq!(b1, 0, "line: β₁ should be 0");
    }

    #[test]
    fn ring_of_cubes_betti() {
        // Square ring in XY plane: should have β₁ = 1 (one hole)
        // The ring must be one voxel thick with a clear hole.
        //  XXX
        //  X X
        //  XXX
        let occ = vec![
            (0, 0, 0), (1, 0, 0), (2, 0, 0), // bottom row
            (0, 1, 0),             (2, 1, 0), // middle sides
            (0, 2, 0), (1, 2, 0), (2, 2, 0), // top row
        ];
        let (b0, b1, b2) = betti_numbers(&occ);
        eprintln!("Ring: β₀={}, β₁={}, β₂={}", b0, b1, b2);
        assert_eq!(b0, 1, "ring: β₀ should be 1");
        assert_eq!(b1, 1, "ring: β₁ should be 1 (one hole)");
        assert_eq!(b2, 0, "ring: β₂ should be 0");
    }
}
