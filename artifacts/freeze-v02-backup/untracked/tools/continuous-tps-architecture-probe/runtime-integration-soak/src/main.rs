use continuous_runtime_integration_soak_tp17a1::run_probe;
use std::{fs, path::PathBuf};

fn main() {
    let report = run_probe();
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("..\\results\\runtime-integration-soak-tp17a1.json");
    let text = serde_json::to_string_pretty(&report).expect("serialize probe result");
    fs::write(&path, format!("{text}\n")).expect("write probe result");
    println!(
        "TP-17A1 {}: scenarios={} result={}",
        report["status"],
        report["scenarios"].as_array().map_or(0, Vec::len),
        path.display()
    );
    // A stable blocked status is an intentional result for an unavailable
    // public integration boundary, not a successful formal-runtime claim.
    if report["status"] == "failed_probe" {
        std::process::exit(1);
    }
}
