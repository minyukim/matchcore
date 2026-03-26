use std::{
    env, fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use rustc_hash::FxHashMap;
use serde::Deserialize;
use walkdir::WalkDir;

fn main() -> Result<()> {
    let mut args = env::args().skip(1);
    let Some(cmd) = args.next() else {
        print_help();
        return Ok(());
    };

    match cmd.as_str() {
        "gen-bench-docs" => generate_bench_docs(),
        other => bail!("unknown xtask command: {other}"),
    }
}

fn print_help() {
    eprintln!("xtask commands:");
    eprintln!("  cargo run -p xtask -- gen-bench-docs");
}

fn generate_bench_docs() -> Result<()> {
    let repo_root = find_repo_root()?;
    let criterion_root = repo_root.join("target/criterion");
    let output_path = repo_root.join("docs/benchmarks.md");

    if !criterion_root.exists() {
        bail!(
            "criterion output not found at {}. Run `cargo bench --bench benches` first.",
            criterion_root.display()
        );
    }

    let results = load_benchmark_results(&criterion_root)?;
    if results.is_empty() {
        bail!(
            "no benchmark results found under {}. Run `cargo bench --bench benches` first.",
            criterion_root.display()
        );
    }

    let markdown = render_markdown(&results);
    fs::create_dir_all(
        output_path
            .parent()
            .context("failed to determine docs output directory")?,
    )?;
    fs::write(&output_path, markdown)
        .with_context(|| format!("failed to write {}", output_path.display()))?;

    println!("generated {}", output_path.display());
    Ok(())
}

fn find_repo_root() -> Result<PathBuf> {
    let mut dir = env::current_dir().context("failed to get current directory")?;

    loop {
        let cargo_toml = dir.join("Cargo.toml");
        let src_dir = dir.join("src");
        let xtask_dir = dir.join("xtask");

        if cargo_toml.exists() && src_dir.exists() && xtask_dir.exists() {
            return Ok(dir);
        }

        if !dir.pop() {
            bail!("could not find repository root");
        }
    }
}

#[derive(Debug, Deserialize)]
struct Estimates {
    median: Statistic,
}

#[derive(Debug, Deserialize)]
struct Statistic {
    point_estimate: f64,
}

#[derive(Debug, Clone)]
struct BenchResult {
    id: String,
    ns: f64,
}

fn load_benchmark_results(criterion_root: &Path) -> Result<Vec<BenchResult>> {
    let mut results = Vec::new();

    for entry in WalkDir::new(criterion_root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_name() == "estimates.json")
    {
        let path = entry.path();

        // Expect: target/criterion/<bench-id>/new/estimates.json
        let Some(parent) = path.parent() else {
            continue;
        };
        if parent.file_name().and_then(|s| s.to_str()) != Some("new") {
            continue;
        }

        let Some(bench_dir) = parent.parent() else {
            continue;
        };

        let rel = bench_dir
            .strip_prefix(criterion_root)
            .with_context(|| format!("failed to strip criterion root from {}", path.display()))?;

        let id = rel
            .components()
            .map(|c| c.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .join("/");

        let raw = fs::read_to_string(path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let estimates: Estimates = serde_json::from_str(&raw)
            .with_context(|| format!("failed to parse {}", path.display()))?;

        results.push(BenchResult {
            id,
            ns: estimates.median.point_estimate,
        });
    }

    Ok(results)
}

fn render_markdown(results: &[BenchResult]) -> String {
    let map: FxHashMap<&str, f64> = results.iter().map(|r| (r.id.as_str(), r.ns)).collect();

    let mut out = String::new();

    out.push_str("## Submit\n\n");

    out.push_str("### Single-order submit\n\n");
    out.push_str("| Benchmark | Time (median) |\n");
    out.push_str("| --- | ---: |\n");
    push_row(
        &mut out,
        "Single standard order into a fresh book",
        map.get("submit/single_standard_order_fresh_book"),
    );
    push_row(
        &mut out,
        "Single iceberg order into a fresh book",
        map.get("submit/single_iceberg_order_fresh_book"),
    );
    push_row(
        &mut out,
        "Single post-only order into a fresh book",
        map.get("submit/single_post_only_order_fresh_book"),
    );
    push_row(
        &mut out,
        "Single good-till-date order into a fresh book",
        map.get("submit/single_good_till_date_order_fresh_book"),
    );
    push_row(
        &mut out,
        "Single pegged order into a fresh book",
        map.get("submit/single_pegged_order_fresh_book"),
    );
    out.push('\n');

    out.push_str("### 10k orders submit\n\n");
    out.push_str("| Benchmark | Time (median) |\n");
    out.push_str("| --- | ---: |\n");
    push_row(
        &mut out,
        "10k standard orders into a fresh book",
        map.get("submit/10k_standard_orders_fresh_book"),
    );
    push_row(
        &mut out,
        "10k iceberg orders into a fresh book",
        map.get("submit/10k_iceberg_orders_fresh_book"),
    );
    push_row(
        &mut out,
        "10k post-only orders into a fresh book",
        map.get("submit/10k_post_only_orders_fresh_book"),
    );
    push_row(
        &mut out,
        "10k good-till-date orders into a fresh book",
        map.get("submit/10k_good_till_date_orders_fresh_book"),
    );
    push_row(
        &mut out,
        "10k pegged orders into a fresh book",
        map.get("submit/10k_pegged_orders_fresh_book"),
    );
    out.push('\n');

    out.push_str("## Amend\n\n");

    out.push_str("### Single-order amend\n\n");
    out.push_str("| Benchmark | Time (median) |\n");
    out.push_str("| --- | ---: |\n");
    push_row(
        &mut out,
        "Single order in single-level book quantity decrease",
        map.get("amend/single_order_in_single_level_book_quantity_decrease"),
    );
    push_row(
        &mut out,
        "Single order in multi-level book quantity decrease",
        map.get("amend/single_order_in_multi_level_book_quantity_decrease"),
    );
    push_row(
        &mut out,
        "Single order in single-level book quantity increase",
        map.get("amend/single_order_in_single_level_book_quantity_increase"),
    );
    push_row(
        &mut out,
        "Single order in multi-level book quantity increase",
        map.get("amend/single_order_in_multi_level_book_quantity_increase"),
    );
    push_row(
        &mut out,
        "Single order in single-level book price update",
        map.get("amend/single_order_in_single_level_book_price_update"),
    );
    push_row(
        &mut out,
        "Single order in multi-level book price update",
        map.get("amend/single_order_in_multi_level_book_price_update"),
    );
    out.push('\n');

    out.push_str("### 10k orders amend\n\n");
    out.push_str("| Benchmark | Time (median) |\n");
    out.push_str("| --- | ---: |\n");
    push_row(
        &mut out,
        "10k orders in single-level book quantity decrease",
        map.get("amend/10k_orders_in_single_level_book_quantity_decrease"),
    );
    push_row(
        &mut out,
        "10k orders in multi-level book quantity decrease",
        map.get("amend/10k_orders_in_multi_level_book_quantity_decrease"),
    );
    push_row(
        &mut out,
        "10k orders in single-level book quantity increase",
        map.get("amend/10k_orders_in_single_level_book_quantity_increase"),
    );
    push_row(
        &mut out,
        "10k orders in multi-level book quantity increase",
        map.get("amend/10k_orders_in_multi_level_book_quantity_increase"),
    );
    push_row(
        &mut out,
        "10k orders in single-level book price update",
        map.get("amend/10k_orders_in_single_level_book_price_update"),
    );
    push_row(
        &mut out,
        "10k orders in multi-level book price update",
        map.get("amend/10k_orders_in_multi_level_book_price_update"),
    );
    out.push('\n');

    out.push_str("## Cancel\n\n");
    out.push_str("| Benchmark | Time (median) |\n");
    out.push_str("| --- | ---: |\n");
    push_row(
        &mut out,
        "Single order in single-level book cancel",
        map.get("cancel/single_order_in_single_level_book_cancel"),
    );
    push_row(
        &mut out,
        "Single order in multi-level book cancel",
        map.get("cancel/single_order_in_multi_level_book_cancel"),
    );
    push_row(
        &mut out,
        "10k orders in single-level book cancel",
        map.get("cancel/10k_orders_in_single_level_book_cancel"),
    );
    push_row(
        &mut out,
        "10k orders in multi-level book cancel",
        map.get("cancel/10k_orders_in_multi_level_book_cancel"),
    );
    out.push('\n');

    out.push_str("## Matching\n\n");

    out.push_str("### Single-level standard book\n\n");
    push_match_table(
        &mut out,
        &map,
        "matching/single_level_standard_book_match_volume",
    );

    out.push_str("### Multi-level standard book\n\n");
    push_match_table(
        &mut out,
        &map,
        "matching/multi_level_standard_book_match_volume",
    );

    out.push_str("### Single-level iceberg book\n\n");
    push_match_table(
        &mut out,
        &map,
        "matching/single_level_iceberg_book_match_volume",
    );

    out.push_str("### Multi-level iceberg book\n\n");
    push_match_table(
        &mut out,
        &map,
        "matching/multi_level_iceberg_book_match_volume",
    );

    out.push_str("## Mixed workload\n\n");
    out.push_str("| Benchmark | Time (median) |\n");
    out.push_str("| --- | ---: |\n");
    push_row(
        &mut out,
        "Submit + amend + match + cancel",
        map.get("mixed/submit_amend_match_cancel"),
    );

    out
}

fn push_match_table(out: &mut String, map: &FxHashMap<&str, f64>, prefix: &str) {
    out.push_str("| Match volume | Time (median) |\n");
    out.push_str("| --- | ---: |\n");

    for volume in ["1", "10", "100", "1000", "10000"] {
        let key = format!("{prefix}_{volume}");
        let value = map.get(key.as_str());
        push_row(out, volume, value);
    }

    out.push('\n');
}

fn push_row(out: &mut String, label: &str, value_ns: Option<&f64>) {
    let value = match value_ns {
        Some(v) => format_bench_time(*v),
        None => "N/A".to_string(),
    };

    out.push_str("| ");
    out.push_str(label);
    out.push_str(" | ");
    out.push_str(&value);
    out.push_str(" |\n");
}

fn format_bench_time(ns: f64) -> String {
    if ns < 1_000.0 {
        format!("~{:.0} ns", ns.round())
    } else if ns < 1_000_000.0 {
        format!("~{:.2} µs", ns / 1_000.0)
    } else if ns < 1_000_000_000.0 {
        format!("~{:.2} ms", ns / 1_000_000.0)
    } else {
        format!("~{:.2} s", ns / 1_000_000_000.0)
    }
}
