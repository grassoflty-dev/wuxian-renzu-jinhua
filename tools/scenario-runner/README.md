# Pivot Scenario Runner

This isolated Rust CLI replays the Grey Hive Power → Gate A route through the feature-gated, trusted-Rust `FormalRuntime::new_input_replay_with_save_dir` library API. Each accepted input advances the unchanged authoritative fixed-step body exactly once. This is deterministic input-scheduled acceptance, not native Tauri or real-time wall-clock acceptance. It never uses the desktop app, the default user save location, or a production save directory.

## Run the acceptance replay

From the repository root:

```powershell
cargo run --locked --manifest-path tools/scenario-runner/Cargo.toml -- verify --fixture tools/scenario-runner/fixtures/grey-hive-power-gate-a-v1.json
```

The command allocates a new, unique directory under `artifacts/acceptance/pivot-scenario-runner-01/runs/`. It does not delete any output. Two independent replays are performed in distinct save roots; each phase (seed, Continue A, Continue B, corrupt-slot rejection) is a real child process. The canonical `scenario.jsonl` contains the complete authoritative WorldView, WorldSnapshot and CommandReceipt, including tick, revision, simulation time, positions and route events, with no field normalization. Run-specific elapsed time, process IDs and temporary paths are kept outside this canonical record. Both replay JSONL files and every child’s raw stdout, stderr and exit code are retained before equality/assertion checks. `verification.json` records those run-specific details, assertion names, fixture/code/output SHA-256 values, and unsupported mechanics.

Use `--output-parent PATH` to choose another dedicated output parent. The runner creates a unique new run directory beneath it and refuses to reuse an existing run directory.

## Coverage boundaries

The fixture checks closed-gate collision, the authoritative out-of-range and near-range power interactions, idempotent retry, open-gate passage, distinct Save A/B checkpoints, Return-to-Hub save behavior, real process exit/restart and independent Continue of A and B, resumed input acceptance, and fail-closed handling of a malformed slot. Dash/QER/Energy formal gameplay is not implemented in this fixture and is explicitly unsupported/pending.


## Clock boundary

The server package defaults to no replay feature. Its ordinary constructors always spawn the existing wall-clock owner, even when a tool enables `deterministic-replay`. The extra constructor is available only to trusted Rust callers compiled with that feature; no browser command, save field, environment variable or production route selects it. It does not grant inventory, capabilities or progression. Input validation, collision, discrete effects, saving and Continue all use the same runtime implementation. It does not accept `ActionCommandRequest`; the input-only fixture has no action scheduler.

A caller's `clientTimeMs` still cannot choose a timestep: the fixed engine step is used. The accepted sample is fresh at its scheduled step, so a slow host or disk write cannot insert extra held-input ticks. The live owner's 250 ms stale-input behavior remains unchanged. The ordinary live-owner test runs with the feature disabled and enabled; replay tests compare the exact owner-step state and verify that host delays do not change tick count or expire scheduled inputs.
