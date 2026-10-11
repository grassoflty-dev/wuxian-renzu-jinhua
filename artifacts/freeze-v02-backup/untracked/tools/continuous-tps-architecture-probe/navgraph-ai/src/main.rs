use continuous_navgraph_ai_tp11a1::run_probe;
use std::{fs, path::PathBuf};

fn main() {
    let report = run_probe();
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("..\\results\\navgraph-ai-tp11a1.json");
    let text = serde_json::to_string_pretty(&report).expect("serialize probe result");
    fs::write(&path, format!("{text}\n")).expect("write probe result");
    println!(
        "TP-11A1 {}: nodes={} edges={} scenarios={} result={}",
        report["status"],
        report["graph"]["expandedNodeCount"],
        report["graph"]["expandedEdgeCount"],
        report["scenarios"].as_array().map_or(0, Vec::len),
        path.display()
    );
    if report["pass"] != true {
        std::process::exit(1);
    }
}
