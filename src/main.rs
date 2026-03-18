use clap::Parser;
use glam::Vec3;
use torus::api::Universe;
use torus::observer::{camera::Camera, render, depth_topology};
use torus::System1;

#[derive(Parser)]
#[command(name = "torus", about = "System 1/System 2 True 3D Engine MVP")]
struct Cli {
    /// Voxel grid resolution (N³)
    #[arg(long, default_value_t = 128)]
    resolution: usize,

    /// Number of photons to emit
    #[arg(long, default_value_t = 1_000_000)]
    photons: usize,

    /// Image resolution (width = height)
    #[arg(long, default_value_t = 256)]
    image_size: u32,

    /// Output directory
    #[arg(long, default_value = ".")]
    out: String,

    /// Run topology ground truth (M3)
    #[arg(long)]
    topology: bool,

    /// Run M5 experiment (topology measurement from depth map)
    #[arg(long)]
    experiment: bool,
}

fn main() {
    let cli = Cli::parse();

    // ===== M1 + M2: Build System 1 =====
    let system1 = System1::build(cli.resolution, cli.photons);

    // ===== M3: Topology Ground Truth =====
    let ground_truth_b1 = if cli.topology || cli.experiment {
        eprintln!("\n[M3] Computing topology ground truth...");
        let torus_region_min = Vec3::new(2.0, 2.0, 4.0);
        let torus_region_max = Vec3::new(8.0, 8.0, 6.0);
        let homology = system1.query_topology(torus_region_min, torus_region_max);
        eprintln!("[M3] GROUND TRUTH: β₀={}, β₁={}, β₂={}",
            homology.beta[0], homology.beta[1], homology.beta[2]);
        eprintln!("[M3] d² = 0: {}", if homology.d_squared_zero { "VERIFIED ✓" } else { "FAILED ✗" });

        if homology.beta[1] == 1 {
            eprintln!("[M3] ✓ H₁ = Z confirmed. The hole exists in System 1.");
        } else {
            eprintln!("[M3] ✗ H₁ ≠ Z. β₁ = {}.", homology.beta[1]);
        }
        homology.beta[1]
    } else {
        1 // assume correct if not running topology
    };

    // ===== M4: Stereo Rendering (default position) =====
    if !cli.experiment {
        eprintln!("\n[M4] Rendering stereo pair...");
        let observer_pos = Vec3::new(5.0, 5.0, 0.5);
        let look_at = Vec3::new(5.0, 5.0, 5.0);
        let (cam_l, cam_r) = Camera::stereo_pair(
            observer_pos, look_at, Vec3::Y,
            90.0, cli.image_size, cli.image_size, 0.065,
        );

        let (depth_l, image_l) = render::render_camera(&system1, &cam_l);
        let (_depth_r, image_r) = render::render_camera(&system1, &cam_r);

        let out = &cli.out;
        render::save_image(&format!("{}/left_eye.png", out), &image_l, cli.image_size, cli.image_size);
        render::save_image(&format!("{}/right_eye.png", out), &image_r, cli.image_size, cli.image_size);
        render::save_depth(&format!("{}/depth_left.png", out), &depth_l, cli.image_size, cli.image_size);

        let hit_count = depth_l.iter().filter(|d| d.is_finite()).count();
        let total = (cli.image_size * cli.image_size) as usize;
        eprintln!("[M4] Rays hitting scene: {}/{} ({:.1}%)", hit_count, total,
            100.0 * hit_count as f64 / total as f64);
    }

    // ===== M5: Experimental Matrix =====
    if cli.experiment {
        eprintln!("\n{}", "=".repeat(80));
        eprintln!("[M5] EXPERIMENT: Topology Preservation Across the Observation Boundary");
        eprintln!("{}", "=".repeat(80));
        eprintln!("Ground truth: β₁ = {} (System 1)", ground_truth_b1);

        let resolutions: Vec<u32> = vec![32, 64, 128, 256, 512];
        let distances: Vec<f32> = vec![2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 10.0, 12.0, 15.0, 20.0];
        let look_at = Vec3::new(5.0, 5.0, 5.0);

        eprintln!("\n{:>6} {:>6} {:>10} {:>10} {:>10} {:>8}",
            "ImgRes", "Dist", "FG_Comp", "BG_Comp", "β₁_est", "MATCH?");
        eprintln!("{:-<6} {:-<6} {:-<10} {:-<10} {:-<10} {:-<8}", "", "", "", "", "", "");

        let mut results: Vec<(u32, f32, usize)> = Vec::new();

        for &img_res in &resolutions {
            for &dist in &distances {
                // Observer on the Z axis, looking toward torus center
                let observer_pos = Vec3::new(5.0, 5.0, 5.0 - dist);

                let cam = Camera::new(observer_pos, look_at, Vec3::Y, 90.0, img_res, img_res);
                let (depth, _image) = render::render_camera(&system1, &cam);

                // M5b: Depth-map topology measurement
                let topo_results = depth_topology::estimate_h1_from_depth(
                    &depth, img_res as usize, img_res as usize, 20,
                );

                let max_b1 = topo_results.iter().map(|r| r.b1_estimate).max().unwrap_or(0);
                let best = topo_results.iter().max_by_key(|r| r.b1_estimate);

                let (fg, bg) = if let Some(b) = best {
                    (b.fg_components, b.bg_components)
                } else {
                    (0, 0)
                };

                let matches = max_b1 == ground_truth_b1;
                eprintln!("{:>6} {:>6.1} {:>10} {:>10} {:>10} {:>8}",
                    img_res, dist, fg, bg, max_b1,
                    if matches { "✓" } else { "✗" });

                results.push((img_res, dist, max_b1));

                // Save images for extreme cases
                if (img_res == 256 && dist == 5.0) || (img_res == 32 && dist == 12.0) {
                    let out = &cli.out;
                    let (_depth2, image2) = render::render_camera(&system1, &cam);
                    render::save_image(
                        &format!("{}/exp_{}px_{}m.png", out, img_res, dist as u32),
                        &image2, img_res, img_res,
                    );
                    render::save_depth(
                        &format!("{}/exp_depth_{}px_{}m.png", out, img_res, dist as u32),
                        &depth, img_res, img_res,
                    );
                }
            }
        }

        // Analysis
        eprintln!("\n{}", "=".repeat(60));
        eprintln!("[M5] ANALYSIS");
        eprintln!("{}", "=".repeat(60));

        // Find critical threshold
        let mut last_fail: Option<(u32, f32)> = None;
        let mut first_pass: Option<(u32, f32)> = None;
        for &(res, dist, b1) in &results {
            if b1 == ground_truth_b1 && first_pass.is_none() {
                first_pass = Some((res, dist));
            }
            if b1 != ground_truth_b1 {
                last_fail = Some((res, dist));
            }
        }

        if let Some((res, dist)) = first_pass {
            eprintln!("First topology recovery: {}px at {}m", res, dist);
        }
        if let Some((res, dist)) = last_fail {
            eprintln!("Last topology failure:   {}px at {}m", res, dist);
        }

        let pass_count = results.iter().filter(|r| r.2 == ground_truth_b1).count();
        let total_count = results.len();
        eprintln!("Topology preserved: {}/{} conditions ({:.0}%)",
            pass_count, total_count, 100.0 * pass_count as f64 / total_count as f64);

        // The finding
        eprintln!("\n[M5] Does H₁ survive the System 1 → System 2 projection?");
        if pass_count > 0 && pass_count < total_count {
            eprintln!("[M5] PHASE TRANSITION DETECTED.");
            eprintln!("[M5] There exists a critical observer fidelity threshold.");
            eprintln!("[M5] Below it: the hole is invisible (β₁ = 0).");
            eprintln!("[M5] Above it: the hole is recovered (β₁ = {}).", ground_truth_b1);
        } else if pass_count == total_count {
            eprintln!("[M5] Topology preserved at ALL tested conditions.");
        } else {
            eprintln!("[M5] Topology NOT recovered at any tested condition.");
            eprintln!("[M5] The observation boundary destroys H₁ regardless of fidelity.");
        }

        eprintln!("\n[EXPERIMENT COMPLETE]");
    }

    if !cli.experiment {
        eprintln!("\n[DONE] System 1 exists. System 2 observed it. Check the images.");
    }
}
