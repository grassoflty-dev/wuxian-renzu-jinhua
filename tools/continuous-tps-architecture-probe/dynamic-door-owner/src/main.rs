fn main() {
    let result = dynamic_door_owner_tp04c1::run_probe();
    if std::env::args().any(|arg| arg == "--write-result") {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../results/dynamic-door-owner-tp04c1.json");
        std::fs::write(&path, serde_json::to_vec_pretty(&result).unwrap()).unwrap();
        println!("wrote {}", path.display());
    } else {
        println!("{}", serde_json::to_string_pretty(&result).unwrap());
    }
}
