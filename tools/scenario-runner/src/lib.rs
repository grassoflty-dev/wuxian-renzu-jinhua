use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    process::{Command, Output},
    time::{Instant, SystemTime, UNIX_EPOCH},
};
use wuxian_horror_ch1::{
    continuous_input::InputSample,
    formal_runtime::{FormalInteractionResponse, FormalRuntime},
    world_v3::{CommandReceipt, WorldSnapshot, WorldView},
};

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Fixture {
    pub fixture_version: String,
    pub scenario_id: String,
    pub world_id: String,
    pub console_actor_id: String,
    pub gate_door_id: String,
    pub steps: Steps,
    pub expected: Expected,
    pub unsupported: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Steps {
    pub closed_gate_approach: u64,
    pub retreat_to_console: u64,
    pub approach_console_x: u64,
    pub return_to_passage_x: u64,
    pub open_gate_passage: u64,
    pub resume_probe: u64,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Expected {
    pub closed_gate_max_z: f32,
    pub passed_gate_min_z: f32,
    pub console_near_range_m: f32,
    pub save_slot_a: String,
    pub save_slot_b: String,
    pub corrupt_slot: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceRecord {
    pub phase: String,
    pub event: String,
    pub world_view: WorldView,
    pub world_snapshot: WorldSnapshot,
    pub command_receipt: Option<CommandReceipt>,
}

impl EvidenceRecord {
    fn from_view(
        phase: &str,
        event: &str,
        view: WorldView,
        receipt: Option<CommandReceipt>,
    ) -> Self {
        Self {
            phase: phase.into(),
            event: event.into(),
            world_snapshot: WorldSnapshot::from_view(view.clone()),
            world_view: view,
            command_receipt: receipt,
        }
    }

    fn from_interaction(phase: &str, event: &str, response: FormalInteractionResponse) -> Self {
        Self::from_view(phase, event, response.view, Some(response.receipt))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhaseOutput {
    pub phase: String,
    pub records: Vec<EvidenceRecord>,
    pub checks: Vec<String>,
    pub errors: Vec<String>,
}

impl PhaseOutput {
    fn new(phase: &str) -> Self {
        Self {
            phase: phase.into(),
            records: Vec::new(),
            checks: Vec::new(),
            errors: Vec::new(),
        }
    }

    fn record(&mut self, event: &str, view: WorldView, receipt: Option<CommandReceipt>) {
        self.records
            .push(EvidenceRecord::from_view(&self.phase, event, view, receipt));
    }

    fn ensure(&mut self, name: &str, ok: bool, detail: impl Into<String>) {
        let detail = detail.into();
        if ok {
            self.checks.push(name.into());
        } else {
            self.errors.push(format!("{name}: {detail}"));
        }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VerificationReport {
    pub suite: String,
    pub passed: bool,
    pub runtime_driver: String,
    pub fixture_version: String,
    pub fixture_sha256: String,
    pub code_sha: String,
    pub command: String,
    pub output_sha256: String,
    pub output_path: String,
    pub run_directory: String,
    pub duration_ms: u128,
    pub process_exit_codes: Vec<u32>,
    pub process_ids: Vec<u32>,
    pub evidence_records: usize,
    pub deterministic_replays_equal: bool,
    pub assertions: Vec<String>,
    pub unsupported: Vec<String>,
}

pub fn run_seed(fixture: &Fixture, save_root: &Path) -> Result<PhaseOutput, String> {
    let runtime = FormalRuntime::new_input_replay_with_save_dir(save_root.to_path_buf())?;
    let mut output = PhaseOutput::new("seed");
    let initial = runtime.snapshot()?;
    output.ensure(
        "world-is-grey-hive",
        initial.world_id == fixture.world_id,
        format!("worldId={}", initial.world_id),
    );
    output.record(
        "initial",
        initial.clone(),
        Some(CommandReceipt::from_view("scenario-start", initial.clone())),
    );

    let rejected = runtime.interact(&fixture.console_actor_id, "power-too-far")?;
    output.ensure(
        "power-interaction-rejected-out-of-range",
        rejected.error_code.as_deref() == Some("E_INTERACTION_OUT_OF_RANGE"),
        format!("errorCode={:?}", rejected.error_code),
    );
    output.records.push(EvidenceRecord::from_interaction(
        "seed",
        "power-too-far",
        rejected,
    ));

    let closed = move_ticks(
        &runtime,
        initial,
        0.0,
        1.0,
        fixture.steps.closed_gate_approach,
    )?;
    let closed_gate = door_open(&closed, &fixture.gate_door_id);
    let closed_z = closed.player.transform.position_m.z_m;
    output.ensure(
        "closed-gate-collision-blocks-passage",
        !closed_gate && closed_z <= fixture.expected.closed_gate_max_z && closed_z > 6.0,
        format!("gateOpen={closed_gate}, playerZ={closed_z}"),
    );
    output.record("closed-gate-blocked", closed.clone(), None);

    let backed = move_ticks(
        &runtime,
        closed,
        0.0,
        -1.0,
        fixture.steps.retreat_to_console,
    )?;
    let near = move_ticks(&runtime, backed, 1.0, 0.0, fixture.steps.approach_console_x)?;
    let power = runtime.interact(&fixture.console_actor_id, "power-event-first")?;
    let near_distance = distance_to_actor(&near, &fixture.console_actor_id);
    output.ensure(
        "console-approach-within-formal-range",
        near_distance <= fixture.expected.console_near_range_m,
        format!("distanceM={near_distance}"),
    );
    output.ensure(
        "power-interaction-applied",
        power.applied && !power.already_applied && power.error_code.is_none(),
        format!(
            "applied={}, alreadyApplied={}, errorCode={:?}",
            power.applied, power.already_applied, power.error_code
        ),
    );
    output.ensure(
        "gate-opens-after-power",
        door_open(&power.view, &fixture.gate_door_id),
        "Gate A did not open after the authoritative Power interaction",
    );
    output.records.push(EvidenceRecord::from_interaction(
        "seed",
        "power-applied",
        power.clone(),
    ));

    let retry = runtime.interact(&fixture.console_actor_id, "power-event-retry")?;
    output.ensure(
        "repeated-power-request-is-idempotent",
        !retry.applied
            && retry.already_applied
            && retry.view.authority_revision == power.view.authority_revision,
        format!(
            "applied={}, alreadyApplied={}, revision={}",
            retry.applied, retry.already_applied, retry.view.authority_revision
        ),
    );
    output.records.push(EvidenceRecord::from_interaction(
        "seed",
        "power-idempotent-retry",
        retry,
    ));

    let saved_a = runtime.save_slot(&fixture.expected.save_slot_a, "Save A", true)?;
    output.record(
        "save-a-created",
        saved_a.clone(),
        Some(CommandReceipt::from_view("save-slot-a", saved_a.clone())),
    );

    let align = move_ticks(
        &runtime,
        saved_a,
        -1.0,
        0.0,
        fixture.steps.return_to_passage_x,
    )?;
    let passed = move_ticks(&runtime, align, 0.0, 1.0, fixture.steps.open_gate_passage)?;
    let passed_z = passed.player.transform.position_m.z_m;
    output.ensure(
        "open-gate-collision-allows-passage",
        door_open(&passed, &fixture.gate_door_id) && passed_z > fixture.expected.passed_gate_min_z,
        format!(
            "gateOpen={}, playerZ={passed_z}",
            door_open(&passed, &fixture.gate_door_id)
        ),
    );
    output.record("open-gate-passed", passed.clone(), None);

    let saved_b = runtime.save_slot(&fixture.expected.save_slot_b, "Save B", true)?;
    output.record(
        "save-b-created",
        saved_b.clone(),
        Some(CommandReceipt::from_view("save-slot-b", saved_b.clone())),
    );
    let returned = runtime.return_to_hub_slot(&fixture.expected.save_slot_b, "Save B", false)?;
    output.record(
        "return-to-hub-saved-b",
        returned.clone(),
        Some(CommandReceipt::from_view("return-to-hub", returned.clone())),
    );
    let slots = runtime.list_save_slots();
    let a = slots
        .iter()
        .find(|slot| slot.slot_id == fixture.expected.save_slot_a);
    let b = slots
        .iter()
        .find(|slot| slot.slot_id == fixture.expected.save_slot_b);
    output.ensure(
        "save-a-b-are-distinct-and-valid",
        a.is_some_and(|slot| slot.valid && slot.gate_open == Some(true))
            && b.is_some_and(|slot| slot.valid && slot.gate_open == Some(true))
            && a.zip(b)
                .is_some_and(|(a, b)| a.player_position_m != b.player_position_m),
        format!(
            "slots={:?}",
            slots
                .iter()
                .map(|s| (&s.slot_id, s.valid, s.gate_open, s.player_position_m))
                .collect::<Vec<_>>()
        ),
    );
    Ok(output)
}

pub fn run_continue(
    fixture: &Fixture,
    save_root: &Path,
    slot_id: &str,
) -> Result<PhaseOutput, String> {
    let runtime = FormalRuntime::new_input_replay_with_save_dir(save_root.to_path_buf())?;
    let mut output = PhaseOutput::new(&format!("continue-{slot_id}"));
    let restored = runtime.continue_slot(slot_id)?;
    output.ensure(
        "continue-restores-powered-world",
        restored.world_id == fixture.world_id && door_open(&restored, &fixture.gate_door_id),
        format!(
            "worldId={}, gateOpen={}",
            restored.world_id,
            door_open(&restored, &fixture.gate_door_id)
        ),
    );
    output.ensure(
        "continue-starts-new-world-epoch",
        restored.world_epoch > 1,
        format!("worldEpoch={}", restored.world_epoch),
    );
    if slot_id == fixture.expected.save_slot_a.as_str() {
        let distance = distance_to_actor(&restored, &fixture.console_actor_id);
        output.ensure(
            "continue-a-restores-near-console-checkpoint",
            distance <= fixture.expected.console_near_range_m,
            format!("distanceM={distance}"),
        );
    } else if slot_id == fixture.expected.save_slot_b.as_str() {
        let z = restored.player.transform.position_m.z_m;
        output.ensure(
            "continue-b-restores-beyond-gate-checkpoint",
            z > fixture.expected.passed_gate_min_z,
            format!("playerZ={z}"),
        );
    }
    output.record(
        "continued-from-slot",
        restored.clone(),
        Some(CommandReceipt::from_view("continue-slot", restored.clone())),
    );
    let moved = move_ticks(&runtime, restored, 1.0, 0.0, fixture.steps.resume_probe)?;
    output.ensure(
        "continued-runtime-accepts-new-input",
        moved.ack_seq > 0,
        format!("ackSeq={}", moved.ack_seq),
    );
    output.record("post-continue-input", moved, None);
    Ok(output)
}

pub fn run_corrupt(save_root: &Path, slot_id: &str) -> Result<PhaseOutput, String> {
    let runtime = FormalRuntime::new_input_replay_with_save_dir(save_root.to_path_buf())?;
    let mut output = PhaseOutput::new("corrupt-slot");
    let result = runtime.continue_slot(slot_id);
    output.ensure(
        "corrupt-slot-fails-closed",
        result.is_err(),
        format!("unexpectedly restored corrupt slot: {}", result.is_ok()),
    );
    let message = result.err().unwrap_or_else(|| "unexpected-success".into());
    let view = runtime.snapshot()?;
    output.record(
        "corrupt-slot-rejected-with-owner-state-unchanged",
        view.clone(),
        Some(CommandReceipt::outcome(
            "continue-corrupt-slot",
            false,
            false,
            Some(message.clone()),
            view,
        )),
    );
    let owner_state_unchanged = output.records.last().is_some_and(|record| {
        record.world_view.world_id == "grey_hive"
            && record.world_view.player.transform.position_m.x_m == 0.0
            && record.world_view.player.transform.position_m.z_m == 0.0
    });
    output.ensure(
        "corrupt-slot-does-not-mutate-owner-state",
        owner_state_unchanged,
        "owner state changed after corrupt-slot rejection",
    );
    output.checks.push(format!("corrupt-slot-error={message}"));
    Ok(output)
}

fn move_ticks(
    runtime: &FormalRuntime,
    mut view: WorldView,
    move_x: f32,
    move_z: f32,
    ticks: u64,
) -> Result<WorldView, String> {
    for _ in 0..ticks {
        let prior = view.clone();
        view = runtime.submit_input(
            InputSample::new(
                prior.world_epoch,
                prior.ack_seq + 1,
                prior.server_time_ms + 50,
                move_x,
                move_z,
            )
            .map_err(|error| format!("input construction failed: {error:?}"))?,
            vec![],
        )?;
        if view.world_epoch != prior.world_epoch
            || view.authority_revision != prior.authority_revision + 1
            || view.ack_seq != prior.ack_seq + 1
        {
            return Err(format!(
                "input revision invariant failed: prior=({}, {}, {}), next=({}, {}, {})",
                prior.world_epoch,
                prior.authority_revision,
                prior.ack_seq,
                view.world_epoch,
                view.authority_revision,
                view.ack_seq
            ));
        }
    }
    Ok(view)
}

fn door_open(view: &WorldView, door_id: &str) -> bool {
    view.doors
        .iter()
        .find(|door| door.door_id == door_id)
        .is_some_and(|door| door.open && !door.locked)
}

fn distance_to_actor(view: &WorldView, actor_id: &str) -> f32 {
    let Some(actor) = view.actors.iter().find(|actor| actor.entity_id == actor_id) else {
        return f32::INFINITY;
    };
    let player = view.player.transform.position_m;
    let target = actor.transform.position_m;
    let dx = player.x_m - target.x_m;
    let dy = player.y_m - target.y_m;
    let dz = player.z_m - target.z_m;
    (dx * dx + dy * dy + dz * dz).sqrt()
}

pub fn run_verification(
    fixture_path: &Path,
    output_parent: &Path,
) -> Result<VerificationReport, String> {
    let started = Instant::now();
    let fixture_bytes = fs::read(fixture_path).map_err(|error| format!("read fixture: {error}"))?;
    let fixture: Fixture = serde_json::from_slice(&fixture_bytes)
        .map_err(|error| format!("parse fixture: {error}"))?;
    fs::create_dir_all(output_parent).map_err(|error| format!("create output parent: {error}"))?;
    let run_dir = create_fresh_run_dir(output_parent)?;
    let exe = std::env::current_exe().map_err(|error| format!("current exe: {error}"))?;
    let mut process_ids = Vec::new();
    let mut process_exit_codes = Vec::new();
    let mut aggregate_checks = Vec::new();
    let mut canonical_first: Option<Vec<u8>> = None;
    let mut records_count = 0usize;

    for replay in 1..=2 {
        let replay_dir = run_dir.join(format!("replay-{replay}"));
        let save_root = replay_dir.join("isolated-save-root");
        fs::create_dir_all(&save_root)
            .map_err(|error| format!("create isolated save root: {error}"))?;
        let mut records = Vec::new();
        let mut checks = Vec::new();

        let seed = invoke_child(
            &exe,
            "seed",
            fixture_path,
            &save_root,
            None,
            &replay_dir,
            "seed",
        )?;
        record_process(&seed, &mut process_ids, &mut process_exit_codes)?;
        collect_phase(seed, &mut records, &mut checks)?;

        for (label, slot_id) in [
            ("continue-a", &fixture.expected.save_slot_a),
            ("continue-b", &fixture.expected.save_slot_b),
        ] {
            let resumed = invoke_child(
                &exe,
                "continue",
                fixture_path,
                &save_root,
                Some(slot_id),
                &replay_dir,
                label,
            )?;
            record_process(&resumed, &mut process_ids, &mut process_exit_codes)?;
            collect_phase(resumed, &mut records, &mut checks)?;
        }

        let corrupt_path = save_root
            .join("slots")
            .join(&fixture.expected.corrupt_slot)
            .join("slot-v4.json");
        fs::create_dir_all(corrupt_path.parent().expect("slot path has a parent"))
            .map_err(|error| format!("create corrupt fixture slot: {error}"))?;
        fs::write(&corrupt_path, b"{\"schemaVersion\":")
            .map_err(|error| format!("write corrupt slot: {error}"))?;
        let corrupt = invoke_child(
            &exe,
            "corrupt",
            fixture_path,
            &save_root,
            Some(&fixture.expected.corrupt_slot),
            &replay_dir,
            "corrupt",
        )?;
        record_process(&corrupt, &mut process_ids, &mut process_exit_codes)?;
        collect_phase(corrupt, &mut records, &mut checks)?;

        validate_monotonic(&records)?;
        records_count = records.len();
        let mut current = Vec::new();
        for record in &records {
            serde_json::to_writer(&mut current, record)
                .map_err(|error| format!("serialize evidence: {error}"))?;
            current.push(b'\n');
        }
        // Keep the complete evidence for both runs even when equality fails.
        fs::write(replay_dir.join("scenario.jsonl"), &current)
            .map_err(|error| format!("write replay evidence: {error}"))?;
        if replay == 1 {
            canonical_first = Some(current);
            aggregate_checks = checks;
        } else {
            require_identical_evidence(canonical_first.as_deref().unwrap_or_default(), &current)?;
        }
    }

    let canonical = canonical_first.ok_or("no deterministic evidence was produced")?;
    let output_file = run_dir.join("scenario.jsonl");
    fs::write(&output_file, &canonical).map_err(|error| format!("write evidence: {error}"))?;
    let output_sha = sha256_hex(&canonical);
    let fixture_sha = sha256_hex(&fixture_bytes);
    let code_sha = git_head_from_manifest();
    let command = format!(
        "cargo run --locked --manifest-path tools/scenario-runner/Cargo.toml -- verify --fixture {} --output-parent {}",
        fixture_path.display(), output_parent.display()
    );
    let report = VerificationReport {
        suite: fixture.scenario_id.clone(),
        passed: true,
        runtime_driver: "deterministic-input-fixed-step".into(),
        fixture_version: fixture.fixture_version.clone(),
        fixture_sha256: fixture_sha,
        code_sha,
        command,
        output_sha256: output_sha,
        output_path: display_path(&output_file),
        run_directory: display_path(&run_dir),
        duration_ms: started.elapsed().as_millis(),
        process_exit_codes,
        process_ids,
        evidence_records: records_count,
        deterministic_replays_equal: true,
        assertions: aggregate_checks,
        unsupported: fixture.unsupported,
    };
    let report_path = run_dir.join("verification.json");
    let report_bytes =
        serde_json::to_vec_pretty(&report).map_err(|error| format!("serialize report: {error}"))?;
    fs::write(&report_path, report_bytes).map_err(|error| format!("write report: {error}"))?;
    Ok(report)
}

fn invoke_child(
    exe: &Path,
    phase: &str,
    fixture: &Path,
    root: &Path,
    slot: Option<&str>,
    evidence_dir: &Path,
    evidence_label: &str,
) -> Result<ChildRun, String> {
    let mut command = Command::new(exe);
    command
        .arg("__child")
        .arg(phase)
        .arg("--fixture")
        .arg(fixture)
        .arg("--save-root")
        .arg(root);
    if let Some(slot) = slot {
        command.arg("--slot").arg(slot);
    }
    let output = command
        .output()
        .map_err(|error| format!("spawn child phase {phase}: {error}"))?;
    // Fixed caller-supplied labels never embed fixture slot paths. Preserve raw
    // output before parsing/assertion checks so a failed child is auditable.
    fs::write(
        evidence_dir.join(format!("{evidence_label}.stdout.json")),
        &output.stdout,
    )
    .map_err(|error| format!("write child stdout: {error}"))?;
    fs::write(
        evidence_dir.join(format!("{evidence_label}.stderr.txt")),
        &output.stderr,
    )
    .map_err(|error| format!("write child stderr: {error}"))?;
    fs::write(
        evidence_dir.join(format!("{evidence_label}.exit-code.txt")),
        output.status.code().unwrap_or(u32::MAX as i32).to_string(),
    )
    .map_err(|error| format!("write child status: {error}"))?;
    Ok(ChildRun {
        exit_code: output.status.code().unwrap_or(u32::MAX as i32) as u32,
        output,
        phase: phase.into(),
    })
}

struct ChildRun {
    exit_code: u32,
    output: Output,
    phase: String,
}

fn record_process(run: &ChildRun, pids: &mut Vec<u32>, codes: &mut Vec<u32>) -> Result<(), String> {
    // std::process::Command::output reaps the child but does not expose its PID;
    // child PID is emitted by the child protocol for acceptance metadata.
    let stdout: serde_json::Value =
        serde_json::from_slice(&run.output.stdout).map_err(|error| {
            format!(
                "phase {} emitted invalid JSON: {error}; stderr={}",
                run.phase,
                String::from_utf8_lossy(&run.output.stderr)
            )
        })?;
    let pid = stdout
        .get("processId")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0) as u32;
    pids.push(pid);
    codes.push(run.exit_code);
    if run.exit_code != 0 {
        return Err(format!(
            "child phase {} exited {}: {}",
            run.phase,
            run.exit_code,
            String::from_utf8_lossy(&run.output.stderr)
        ));
    }
    Ok(())
}

fn collect_phase(
    run: ChildRun,
    records: &mut Vec<EvidenceRecord>,
    checks: &mut Vec<String>,
) -> Result<(), String> {
    let envelope: ChildEnvelope = serde_json::from_slice(&run.output.stdout)
        .map_err(|error| format!("parse {} output: {error}", run.phase))?;
    if !envelope.errors.is_empty() {
        return Err(format!(
            "phase {} assertions failed: {}",
            envelope.phase,
            envelope.errors.join("; ")
        ));
    }
    records.extend(envelope.records);
    checks.extend(
        envelope
            .checks
            .into_iter()
            .map(|check| format!("{}:{check}", envelope.phase)),
    );
    Ok(())
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ChildEnvelope {
    phase: String,
    records: Vec<EvidenceRecord>,
    checks: Vec<String>,
    errors: Vec<String>,
}

pub fn child_json(output: PhaseOutput) -> Result<(), String> {
    let envelope = ChildEnvelopeOwned {
        phase: output.phase,
        process_id: std::process::id(),
        records: output.records,
        checks: output.checks,
        errors: output.errors,
    };
    serde_json::to_writer(io::stdout().lock(), &envelope)
        .map_err(|error| format!("serialize child output: {error}"))?;
    io::stdout()
        .lock()
        .write_all(b"\n")
        .map_err(|error| format!("write child output: {error}"))?;
    Ok(())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ChildEnvelopeOwned {
    phase: String,
    process_id: u32,
    records: Vec<EvidenceRecord>,
    checks: Vec<String>,
    errors: Vec<String>,
}

fn validate_monotonic(records: &[EvidenceRecord]) -> Result<(), String> {
    let mut last_by_phase = std::collections::BTreeMap::<&str, (u64, u64, u64)>::new();
    for record in records {
        let view = &record.world_view;
        let key = (view.world_epoch, view.authority_revision, view.ack_seq);
        if let Some(previous) = last_by_phase.get(record.phase.as_str()) {
            if key.0 < previous.0 || key.1 < previous.1 || key.2 < previous.2 {
                return Err(format!(
                    "non-monotonic revision in {} at {}: previous={previous:?}, next={key:?}",
                    record.phase, record.event
                ));
            }
        }
        last_by_phase.insert(record.phase.as_str(), key);
    }
    Ok(())
}

fn create_fresh_run_dir(parent: &Path) -> Result<PathBuf, String> {
    for attempt in 0..64u32 {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| format!("system clock: {error}"))?
            .as_nanos();
        let path = parent.join(format!("run-{}-{nanos}-{attempt}", std::process::id()));
        match fs::create_dir(&path) {
            Ok(()) => return Ok(path),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(format!("create fresh run directory: {error}")),
        }
    }
    Err("unable to allocate unique acceptance run directory".into())
}

fn sha256_hex(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn git_head_from_manifest() -> String {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo = manifest
        .parent()
        .and_then(Path::parent)
        .unwrap_or(&manifest);
    Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_owned())
        .unwrap_or_else(|| "unavailable".into())
}

fn display_path(path: &Path) -> String {
    path.canonicalize()
        .unwrap_or_else(|_| path.to_path_buf())
        .display()
        .to_string()
}

fn require_identical_evidence(first: &[u8], second: &[u8]) -> Result<(), String> {
    if first == second {
        Ok(())
    } else {
        let offset = first
            .iter()
            .zip(second)
            .position(|(a, b)| a != b)
            .unwrap_or(first.len().min(second.len()));
        Err(format!("determinism mismatch between replay-1 and replay-2 full authoritative evidence at byte {offset}; both replay directories retain raw outputs"))
    }
}

#[cfg(test)]
mod evidence_tests {
    use super::*;

    #[test]
    fn comparison_rejects_authoritative_tick_time_position_and_event_changes() {
        let runtime = FormalRuntime::new_input_replay_with_save_dir(PathBuf::from(
            "unused-evidence-save-root",
        ))
        .unwrap();
        let view = runtime.snapshot().unwrap();
        let receipt = CommandReceipt::from_view("evidence-test", view.clone());
        let record = EvidenceRecord::from_view("test", "start", view, Some(receipt));
        let value = serde_json::to_value(&record).unwrap();
        let first = serde_json::to_vec(&value).unwrap();
        require_identical_evidence(&first, &first).unwrap();
        for path in [
            "/worldView/serverTick",
            "/worldView/authorityRevision",
            "/worldView/serverTimeMs",
            "/worldView/player/transform/positionM/xM",
            "/worldView/progression/eventSeq",
            "/worldSnapshot/serverTick",
            "/commandReceipt/authorityRevision",
            "/commandReceipt/snapshot/serverTick",
        ] {
            let mut changed = value.clone();
            *changed.pointer_mut(path).unwrap() = serde_json::json!(999);
            let second = serde_json::to_vec(&changed).unwrap();
            assert!(
                require_identical_evidence(&first, &second).is_err(),
                "ignored {path}"
            );
            assert_ne!(sha256_hex(&first), sha256_hex(&second));
        }
    }
}
