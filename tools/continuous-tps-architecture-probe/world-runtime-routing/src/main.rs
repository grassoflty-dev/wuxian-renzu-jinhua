use std::fs;
use std::path::PathBuf;

fn result_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../results/world-runtime-routing-tp14b1.json")
}

fn main() {
    let result = continuous_world_runtime_routing_tp14b1::run_probe();
    let text = serde_json::to_string_pretty(&result).expect("probe result serializes");
    fs::write(result_path(), format!("{text}\n")).expect("probe result writes");
    println!(
        "TP-14B1 {} scenarios={} result={}",
        result["status"],
        result["scenarios"].as_array().map_or(0, Vec::len),
        result_path().display()
    );
    if result["pass"] != true {
        std::process::exit(1);
    }
}
