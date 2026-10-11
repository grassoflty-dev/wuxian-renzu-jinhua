// Existing in-memory campaign tests; no real saves or media access.
include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../../server-rs/tests/campaign_flow.rs"
));
