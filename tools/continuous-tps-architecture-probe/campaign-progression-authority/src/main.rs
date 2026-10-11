use continuous_campaign_progression_authority_tp14a1::run_probe;
use std::{fs, path::PathBuf};

fn main() {
    let report = run_probe();
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("..\\results\\campaign-progression-authority-tp14a1.json");
    let text = serde_json::to_string_pretty(&report).expect("serialize probe result");
    fs::write(&path, format!("{text}\n")).expect("write probe result");
    println!(
        "TP-14A1 {}: scenarios={} result={}",
        report["status"],
        report["scenarios"].as_array().map_or(0, Vec::len),
        path.display()
    );
    if report["pass"] != true {
        std::process::exit(1);
    }
}
