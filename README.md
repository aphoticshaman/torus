# torus

Observer-independent 3D simulation with verified topological preservation across projection boundaries.

This is the companion code for the paper *"Observer-Independent 3D Simulation and Topological Preservation Across Projection Boundaries"* (Cardwell 2026, Zenodo DOI: [TBD]).

## What This Does

A minimal Rust engine that enforces a hard separation between an observer-independent simulation (System 1) and an observation apparatus (System 2). System 1 builds a solid torus in a voxel grid, propagates light via forward photon mapping, and computes the cubical complex homology with explicit d² = 0 verification. System 2 queries System 1 through a compile-time-enforced API boundary and attempts to recover the torus's topology from depth maps.

## The Result

The solid torus has β₁ = 1 (one non-contractible loop — the hole). System 2 recovers this topological feature at close range and loses it at long range, with a sharp phase transition that is **distance-dependent, not resolution-dependent**:

| Resolution | 2m | 5m | 8m | 12m |
|:----------:|:--:|:--:|:--:|:---:|
| 32x32      | β₁=1 | β₁=1 | β₁=0 | β₁=0 |
| 64x64      | β₁=1 | β₁=1 | β₁=0 | β₁=0 |
| 128x128    | β₁=1 | β₁=1 | β₁=0 | β₁=0 |
| 256x256    | β₁=1 | β₁=1 | β₁=0 | β₁=0 |
| 512x512    | β₁=1 | β₁=1 | β₁=0 | β₁=0 |

More pixels do not help. A different position does.

## Build

```bash
cargo build --release
```

## Test

```bash
cargo test
```

17 tests pass, covering SDF correctness, voxelization, photon mapping, ray tracing, cubical complex homology (including d² = 0 verification), and depth-map topology detection.

## Run

Verify the hole exists (ground truth):

```bash
cargo run --release -- --resolution 128 --photons 1000000 --topology --out .
```

Run the full experiment (20-condition matrix):

```bash
cargo run --release -- --resolution 128 --photons 1000000 --experiment --out .
```

Total time: under 5 minutes on any modern CPU. No GPU required.

## Requirements

- Rust toolchain (stable, 1.70+): https://rustup.rs
- No other dependencies. No GPU. No external data.

## License

MIT

## Citation

Cardwell, R. J. (2026). Observer-Independent 3D Simulation and Topological Preservation Across Projection Boundaries. Zenodo. DOI: [TBD]

## Acknowledgments

Crystalline Labs LLC. AI computational collaborator: Claude (Anthropic).
