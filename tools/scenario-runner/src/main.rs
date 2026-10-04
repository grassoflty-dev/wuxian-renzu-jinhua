use scenario_runner::{child_json, run_continue, run_corrupt, run_seed, run_verification, Fixture};
use std::{
    collections::BTreeMap,
    env,
    ffi::OsString,
    path::{Path, PathBuf},
    process,
};

fn main() {
    if let Err(error) = run() {
        eprintln!("scenario-runner: {error}");
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args_os().skip(1);
    let command = args
        .next()
        .and_then(|value| value.into_string().ok())
        .ok_or_else(usage)?;
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let default_fixture = manifest.join("fixtures/grey-hive-power-gate-a-v1.json");
    let repo = manifest
        .parent()
        .and_then(Path::parent)
        .ok_or("cannot resolve repository root from crate manifest")?
        .to_path_buf();

    if command == "__child" {
        let phase = args
            .next()
            .and_then(|value| value.into_string().ok())
            .ok_or_else(usage)?;
        let options = parse_options(args.collect(), &["--fixture", "--save-root", "--slot"])?;
        let fixture_path = option_path(&options, "--fixture").unwrap_or(default_fixture);
        let save_root =
            option_path(&options, "--save-root").ok_or("child phase requires --save-root")?;
        let slot = option_string(&options, "--slot");
        let fixture: Fixture = serde_json::from_slice(
            &std::fs::read(&fixture_path).map_err(|error| format!("read fixture: {error}"))?,
        )
        .map_err(|error| format!("parse fixture: {error}"))?;
        let output = match phase.as_str() {
            "seed" => run_seed(&fixture, &save_root)?,
            "continue" => run_continue(
                &fixture,
                &save_root,
                slot.as_deref().ok_or("continue phase requires --slot")?,
            )?,
            "corrupt" => run_corrupt(
                &save_root,
                slot.as_deref().ok_or("corrupt phase requires --slot")?,
            )?,
            _ => return Err(format!("unknown child phase: {phase}")),
        };
        return child_json(output);
    }

    match command.as_str() {
        "verify" => {
            let options = parse_options(args.collect(), &["--fixture", "--output-parent"])?;
            let fixture_path = option_path(&options, "--fixture").unwrap_or(default_fixture);
            let output_parent = option_path(&options, "--output-parent")
                .unwrap_or_else(|| repo.join("artifacts/acceptance/pivot-scenario-runner-01/runs"));
            let report = run_verification(&fixture_path, &output_parent)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
            );
            Ok(())
        }
        "help" | "--help" | "-h" => {
            println!("{}", usage());
            Ok(())
        }
        _ => Err(format!("unknown command: {command}\n{}", usage())),
    }
}

fn parse_options(
    args: Vec<OsString>,
    allowed: &[&str],
) -> Result<BTreeMap<String, OsString>, String> {
    let mut options = BTreeMap::new();
    let mut index = 0;
    while index < args.len() {
        let key = args[index].to_string_lossy().into_owned();
        if !allowed.contains(&key.as_str()) {
            return Err(format!("unexpected argument: {key}"));
        }
        let value = args
            .get(index + 1)
            .ok_or_else(|| format!("{key} requires a value"))?
            .clone();
        if options.insert(key.clone(), value).is_some() {
            return Err(format!("duplicate option {key}"));
        }
        index += 2;
    }
    Ok(options)
}

fn option_path(options: &BTreeMap<String, OsString>, name: &str) -> Option<PathBuf> {
    options.get(name).map(PathBuf::from)
}

fn option_string(options: &BTreeMap<String, OsString>, name: &str) -> Option<String> {
    options
        .get(name)
        .map(|value| value.to_string_lossy().into_owned())
}

fn usage() -> String {
    "Usage: scenario-runner verify [--fixture PATH] [--output-parent PATH]\n       scenario-runner help".into()
}
