// Compile and run the existing test source unchanged; do not copy it.
include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../../server-rs/tests/world_v3_contract_test.rs"
));
