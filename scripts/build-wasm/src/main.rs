use anyhow::{Result, anyhow};
use clap::{Parser, ValueEnum};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::Instant,
};

/// Build profile
#[derive(Debug, Clone, ValueEnum)]
enum Profile {
    Dev,
    Release,
}

/// Output targets for wasm-bindgen
#[derive(Debug, Clone, ValueEnum)]
enum Target {
    Bundler,
    Nodejs,
    Web,
}

impl Target {
    fn as_str(&self) -> &'static str {
        match self {
            Target::Bundler => "bundler",
            Target::Nodejs => "nodejs",
            Target::Web => "web",
        }
    }

    fn all() -> Vec<Self> {
        vec![Self::Bundler, Self::Nodejs, Self::Web]
    }
}

/// CLI arguments
#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "WASM build tool for building & optimizing Rust/Wasm",
    long_about = "Builds a Rust WASM target, runs wasm-bindgen for selected outputs, \
                  and performs optional post-processing like sourcemaps and patching."
)]
struct Args {
    /// Build profile
    #[arg(long, value_enum, default_value_t = Profile::Dev)]
    profile: Profile,

    /// Optional target (build all if omitted)
    #[arg(long, value_enum, default_value = None)]
    target: Option<Target>,

    /// Path to workspace root
    #[arg(long, default_value = ".")]
    workspace: PathBuf,

    /// Path to output directory, relative to the workspace root
    #[arg(long, default_value = "./crates/resonix/dist")]
    out_dir: PathBuf,

    /// Path to wasm binary (relative to workspace)
    #[arg(long, default_value = "target/wasm32-unknown-unknown")]
    wasm_dir: PathBuf,

    /// Name of wasm file (without extension)
    #[arg(long, default_value = "resonix")]
    wasm_name: String,
}

// adapted from Loro's script: https://github.com/loro-dev/loro/blob/main/crates/loro-wasm/scripts/build.ts
fn main() -> Result<()> {
    let args = Args::parse();
    let start = Instant::now();

    run(&args)?;

    println!(
        "\n✅ Build complete in {:.2}s",
        start.elapsed().as_secs_f64()
    );

    Ok(())
}

fn run(args: &Args) -> Result<()> {
    cargo_build(args)?;

    let targets = match &args.target {
        Some(t) => vec![t.clone()],
        None => Target::all(),
    };

    for target in targets {
        build_target(args, &target)?;
    }

    Ok(())
}

fn cargo_build(args: &Args) -> Result<()> {
    let profile = match args.profile {
        Profile::Dev => "dev",
        Profile::Release => "release",
    };

    let mut cmd = Command::new("cargo");
    cmd.args([
        "build",
        "--target",
        "wasm32-unknown-unknown",
        "--features",
        "js",
        "--profile",
        profile,
    ])
    .current_dir(&args.workspace);

    if matches!(args.profile, Profile::Release) {
        cmd.env("RUSTFLAGS", "-C debuginfo=2")
            .env("CARGO_PROFILE_RELEASE_DEBUG", "true")
            .env("CARGO_PROFILE_RELEASE_STRIP", "none");
    }

    println!("> {:?}", cmd);

    let status = cmd.status()?;
    if !status.success() {
        return Err(anyhow!("cargo build failed"));
    }

    Ok(())
}

fn build_target(args: &Args, target: &Target) -> Result<()> {
    let target_name = target.as_str();
    println!("🏗️ Building target [{}]", target_name);

    let wasm_out_dir = args.workspace.join(&args.out_dir).join(target_name);
    let _ = fs::remove_dir_all(&wasm_out_dir);

    let out_dir_str = wasm_out_dir.to_str().unwrap();
    println!("Output directory for build [{}]", out_dir_str);

    let profile_dir = match args.profile {
        Profile::Dev => "debug",
        Profile::Release => "release",
    };

    let wasm_path = args
        .workspace
        .join(&args.wasm_dir)
        .join(profile_dir)
        .join(format!("{}.wasm", args.wasm_name));

    let mut cmd = Command::new("wasm-bindgen");
    cmd.args([
        "--keep-debug",
        "--weak-refs",
        "--target",
        target_name,
        "--out-dir",
        out_dir_str,
        wasm_path.to_str().unwrap(),
    ])
    .current_dir(&args.workspace);

    println!("> {:?}", cmd);

    let status = cmd.status()?;
    if !status.success() {
        return Err(anyhow!("wasm-bindgen failed"));
    }

    post_process(&wasm_out_dir, &args.wasm_name)?;

    Ok(())
}

fn post_process(wasm_out_dir: &Path, wasm_name: &str) -> Result<()> {
    let wasm_file_path = wasm_out_dir.join(format!("{}_bg.wasm", wasm_name));

    if !wasm_file_path.exists() {
        println!("⚠️ Skipping post-processing, missing {:?}", wasm_file_path);
        return Ok(());
    } else {
        println!("Running post-processing for {:?}", wasm_file_path);
    }

    // Placeholder for tools like:
    // - wasm-opt
    // - wasm-snip
    // - custom sourcemap tools

    Ok(())
}
