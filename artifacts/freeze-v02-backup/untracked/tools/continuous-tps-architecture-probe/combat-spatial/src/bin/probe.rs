use std::fs;

fn main() {
    let report = combat_spatial_tp12a1::run_all_scenarios();
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../results/combat-spatial-tp12a1.json");
    fs::write(
        &path,
        format!("{}\n", serde_json::to_string_pretty(&report).unwrap()),
    )
    .unwrap();
    println!("{}", serde_json::to_string_pretty(&report).unwrap());
    if !report.machine_pass {
        std::process::exit(1);
    }
}
