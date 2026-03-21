use serde::Deserialize;
use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
    process,
};

// 20% regressions allowed before CI script throws an error
const REGRESSION_THRESHOLD: f64 = 1.20;
const IMPROVEMENT_THRESHOLD: f64 = 0.90; // 10% faster threshold

// generated locally as a baseline benchmark
const LOCAL_BENCHMARKS_PATH: &str = "benches/baseline.json";

// generated in CI for automated comparisons
const CI_BENCHMARKS_PATH: &str = "benches/new.json";

#[derive(Debug, Deserialize)]
#[serde(tag = "reason")]
#[allow(clippy::large_enum_variant)]
pub enum BenchmarkReport {
    #[serde(rename = "benchmark-complete")]
    BenchmarkComplete {
        id: Option<String>,
        report_directory: String,
        iteration_count: Vec<u64>,
        measured_values: Vec<f64>,
        unit: String,
        throughput: Vec<f64>,

        typical: Option<Estimate>,
        mean: Option<Estimate>,
        median: Option<Estimate>,
        median_abs_dev: Option<Estimate>,
        slope: Option<Estimate>,

        change: Option<serde_json::Value>,
    },

    #[serde(rename = "group-complete")]
    GroupComplete {
        group_name: String,
        benchmarks: Vec<String>,
        report_directory: String,
    },
}

#[derive(Debug, Deserialize)]
pub struct Estimate {
    pub estimate: f64,
    pub lower_bound: f64,
    pub upper_bound: f64,
    pub unit: String,
}

fn load_benchmark_data(path: &str) -> HashMap<String, f64> {
    let file = File::open(path).unwrap_or_else(|e| panic!("failed to open {}: {}", path, e));
    let reader = BufReader::new(file);

    let mut results = HashMap::new();

    for line in reader.lines() {
        let line = line.expect("failed to read line");
        if line.trim().is_empty() {
            continue;
        }

        let entry: BenchmarkReport = serde_json::from_str(&line).expect("invalid JSON line");

        if let BenchmarkReport::BenchmarkComplete {
            id: Some(id),
            typical: Some(typical),
            ..
        } = entry
        {
            results.insert(id, typical.estimate);
        }
    }

    results
}

fn main() {
    println!("Loading locally-generated benchmarks: {LOCAL_BENCHMARKS_PATH}");
    println!("Loading current benchmarks from CI: {CI_BENCHMARKS_PATH}");

    let baseline = load_benchmark_data(LOCAL_BENCHMARKS_PATH);
    let current = load_benchmark_data(CI_BENCHMARKS_PATH);

    let mut failed = false;

    for (name, base_time) in &baseline {
        match current.get(name) {
            Some(new_time) => {
                let ratio = new_time / base_time;
                println!("{name}: {ratio:.2}x");

                // Skip extremely fast benchmarks (noise)
                if *base_time < 1e-6 {
                    println!("{name}: skipped (too fast)");
                    continue;
                }
                // Regression
                if ratio > REGRESSION_THRESHOLD {
                    println!("❌ {name}: {:.2}x slower", ratio);
                    failed = true;
                }
                // Improvement
                else if ratio < IMPROVEMENT_THRESHOLD {
                    println!("🚀 {name}: {:.2}x faster", ratio);
                }
                // Stable
                else {
                    println!("≈ {name}: {:.2}x (no significant change)", ratio);
                }
            }
            None => {
                eprintln!("⚠️ Missing benchmark in current run: {name}");
            }
        }
    }

    if failed {
        eprintln!("🚨 Performance regression detected");
        process::exit(1);
    } else {
        println!("✅ No significant regressions");
    }
}
