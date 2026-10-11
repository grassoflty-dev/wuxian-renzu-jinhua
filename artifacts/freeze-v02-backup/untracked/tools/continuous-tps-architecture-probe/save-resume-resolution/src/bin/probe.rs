use std::fs;
use std::path::Path;

fn main() {
    let report = save_resume_resolution_tp03d1::run_all_scenarios();
    let output =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../results/save-resume-resolution-tp03d1.json");
    fs::write(
        &output,
        format!("{}\n", serde_json::to_string_pretty(&report).unwrap()),
    )
    .unwrap();
    println!("{}", serde_json::to_string_pretty(&report).unwrap());
    if !report.machine_pass {
        std::process::exit(1);
    }
}
