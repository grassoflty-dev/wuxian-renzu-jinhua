//! TP-03C1 isolated checkpoint-resolver contract probe.
//!
//! This crate is deliberately standalone.  It contains a typed resolver
//! boundary and a parameterized candidate-only nav projection used to prove
//! error and policy semantics.  It is not a production resolver and never
//! reads or writes a player save.

use std::cell::Cell;
use std::collections::BTreeMap;

pub const WORLD_ID: &str = "grey_hive";
pub const POST_BIO_ID: &str = "gh_cp_post_bio";
pub const POST_BIO_ROOM: &str = "GH-B05";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CheckpointSpec {
    pub id: &'static str,
    pub stage: &'static str,
    pub room: &'static str,
    pub requires_lockdown: bool,
}

const CHECKPOINTS: [CheckpointSpec; 6] = [
    CheckpointSpec { id: "gh_cp_airlock", stage: "initial", room: "GH-A01", requires_lockdown: false },
    CheckpointSpec { id: "gh_cp_power", stage: "post_power", room: "GH-A03", requires_lockdown: false },
    CheckpointSpec { id: "gh_cp_central", stage: "gate_a", room: "GH-B01", requires_lockdown: false },
    CheckpointSpec { id: POST_BIO_ID, stage: "post_bio_before_gate_b", room: POST_BIO_ROOM, requires_lockdown: true },
    CheckpointSpec { id: "gh_cp_pre_sentinel", stage: "pre_sentinel", room: "GH-C02", requires_lockdown: true },
    CheckpointSpec { id: "gh_cp_exit", stage: "result_exit", room: "GH-C04", requires_lockdown: true },
];

pub fn canonical_checkpoints() -> &'static [CheckpointSpec; 6] {
    &CHECKPOINTS
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FootTransform {
    pub x_m: f64,
    pub y_m: f64,
    pub z_m: f64,
    pub yaw_rad: f64,
}

impl FootTransform {
    pub const fn new(x_m: f64, y_m: f64, z_m: f64, yaw_rad: f64) -> Self {
        Self { x_m, y_m, z_m, yaw_rad }
    }

    pub fn is_finite(self) -> bool {
        self.x_m.is_finite()
            && self.y_m.is_finite()
            && self.z_m.is_finite()
            && self.yaw_rad.is_finite()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChoiceState {
    Taken,
    Left,
    Unresolved,
}

impl ChoiceState {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Taken => "taken",
            Self::Left => "left",
            Self::Unresolved => "unresolved",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolveInput {
    pub checkpoint_id: String,
    pub world_id: String,
    pub layout_version: String,
    pub world_epoch: u64,
    pub lockdown_completed: bool,
    pub choice: ChoiceState,
    pub result_map_null: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolverContract {
    pub world_id: String,
    pub layout_version: String,
    /// False is intentional: this probe has no formal scene export.
    pub production_scene_bound: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProjectionFailure {
    MissingSpawn,
    Blocked,
    Unreachable,
    Failed,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProjectionSample {
    pub room: &'static str,
    pub transform: FootTransform,
    /// Candidate-only samples are evidence, never a production fallback.
    pub candidate_only: bool,
}

pub trait ReadOnlyNavProjection {
    fn project(&self, checkpoint: &CheckpointSpec, input: &ResolveInput)
        -> Result<ProjectionSample, ProjectionFailure>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResolveError {
    UnknownCheckpoint,
    WrongWorld,
    WrongLayout,
    MissingSpawn,
    NonFinite,
    Blocked,
    Unreachable,
    ProjectionFailed,
    TerminalNotApplicable,
}

impl ResolveError {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UnknownCheckpoint => "unknown_checkpoint",
            Self::WrongWorld => "wrong_world",
            Self::WrongLayout => "wrong_layout",
            Self::MissingSpawn => "missing_spawn",
            Self::NonFinite => "non_finite",
            Self::Blocked => "blocked",
            Self::Unreachable => "unreachable",
            Self::ProjectionFailed => "projection_failed",
            Self::TerminalNotApplicable => "terminal_not_applicable",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Resolution {
    pub transform: FootTransform,
    pub candidate_only: bool,
}

fn checkpoint(id: &str) -> Option<&'static CheckpointSpec> {
    CHECKPOINTS.iter().find(|spec| spec.id == id)
}

pub fn resolve_checkpoint(
    contract: &ResolverContract,
    input: &ResolveInput,
    nav: &dyn ReadOnlyNavProjection,
) -> Result<Resolution, ResolveError> {
    let target = checkpoint(&input.checkpoint_id).ok_or(ResolveError::UnknownCheckpoint)?;
    if input.world_id != contract.world_id || input.world_id != WORLD_ID {
        return Err(ResolveError::WrongWorld);
    }
    if input.layout_version != contract.layout_version {
        return Err(ResolveError::WrongLayout);
    }
    if input.result_map_null {
        return Err(ResolveError::TerminalNotApplicable);
    }
    if target.requires_lockdown && !input.lockdown_completed {
        return Err(ResolveError::Unreachable);
    }

    let sample = nav.project(target, input).map_err(|failure| match failure {
        ProjectionFailure::MissingSpawn => ResolveError::MissingSpawn,
        ProjectionFailure::Blocked => ResolveError::Blocked,
        ProjectionFailure::Unreachable => ResolveError::Unreachable,
        ProjectionFailure::Failed => ResolveError::ProjectionFailed,
    })?;
    if sample.room != target.room || !sample.transform.is_finite() {
        return Err(if sample.transform.is_finite() {
            ResolveError::ProjectionFailed
        } else {
            ResolveError::NonFinite
        });
    }
    Ok(Resolution { transform: sample.transform, candidate_only: sample.candidate_only })
}

#[derive(Debug)]
pub struct CandidateNav {
    samples: BTreeMap<&'static str, ProjectionSample>,
    failures: BTreeMap<&'static str, ProjectionFailure>,
    calls: Cell<usize>,
}

impl CandidateNav {
    pub fn grey_hive_v7_candidate() -> Self {
        let mut samples = BTreeMap::new();
        let entries = [
            ("gh_cp_airlock", "GH-A01", [10.0, 0.0, 8.0]),
            ("gh_cp_power", "GH-A03", [34.0, 0.0, 14.0]),
            ("gh_cp_central", "GH-B01", [52.0, 0.0, 38.0]),
            (POST_BIO_ID, POST_BIO_ROOM, [52.0, 0.0, 57.0]),
            ("gh_cp_pre_sentinel", "GH-C02", [52.0, -3.0, 77.0]),
            ("gh_cp_exit", "GH-C04", [70.0, -3.0, 93.0]),
        ];
        for (id, room, [x_m, y_m, z_m]) in entries {
            samples.insert(
                id,
                ProjectionSample {
                    room,
                    transform: FootTransform::new(x_m, y_m, z_m, 0.0),
                    candidate_only: true,
                },
            );
        }
        Self { samples, failures: BTreeMap::new(), calls: Cell::new(0) }
    }

    pub fn without_spawn(mut self, id: &'static str) -> Self {
        self.samples.remove(id);
        self
    }

    pub fn with_failure(mut self, id: &'static str, failure: ProjectionFailure) -> Self {
        self.failures.insert(id, failure);
        self
    }

    pub fn with_sample(mut self, id: &'static str, sample: ProjectionSample) -> Self {
        self.samples.insert(id, sample);
        self
    }

    pub fn calls(&self) -> usize {
        self.calls.get()
    }
}

impl ReadOnlyNavProjection for CandidateNav {
    fn project(
        &self,
        checkpoint: &CheckpointSpec,
        _input: &ResolveInput,
    ) -> Result<ProjectionSample, ProjectionFailure> {
        self.calls.set(self.calls.get() + 1);
        if let Some(failure) = self.failures.get(checkpoint.id) {
            return Err(*failure);
        }
        self.samples.get(checkpoint.id).copied().ok_or(ProjectionFailure::MissingSpawn)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn contract() -> ResolverContract {
        ResolverContract {
            world_id: WORLD_ID.into(),
            layout_version: "ag23-v7-candidate".into(),
            production_scene_bound: false,
        }
    }

    fn input(id: &str, lockdown: bool, choice: ChoiceState) -> ResolveInput {
        ResolveInput {
            checkpoint_id: id.into(),
            world_id: WORLD_ID.into(),
            layout_version: "ag23-v7-candidate".into(),
            world_epoch: 7,
            lockdown_completed: lockdown,
            choice,
            result_map_null: false,
        }
    }

    #[test]
    fn six_canonical_ids_and_post_bio_semantics_are_frozen() {
        assert_eq!(canonical_checkpoints().len(), 6);
        assert_eq!(canonical_checkpoints()[3].id, POST_BIO_ID);
        assert_eq!(canonical_checkpoints()[3].room, POST_BIO_ROOM);
        assert!(canonical_checkpoints().iter().all(|item| item.id.starts_with("gh_cp_")));
    }

    #[test]
    fn choice_states_do_not_gate_post_bio_after_lockdown() {
        for choice in [ChoiceState::Taken, ChoiceState::Left, ChoiceState::Unresolved] {
            let nav = CandidateNav::grey_hive_v7_candidate();
            let result = resolve_checkpoint(&contract(), &input(POST_BIO_ID, true, choice), &nav).unwrap();
            assert!(result.candidate_only);
            assert_eq!(result.transform.y_m, 0.0);
        }
    }

    #[test]
    fn lockdown_false_cannot_project_gate_b_downstream() {
        for id in [POST_BIO_ID, "gh_cp_pre_sentinel"] {
            let nav = CandidateNav::grey_hive_v7_candidate();
            assert_eq!(resolve_checkpoint(&contract(), &input(id, false, ChoiceState::Unresolved), &nav), Err(ResolveError::Unreachable));
            assert_eq!(nav.calls(), 0);
        }
        let nav = CandidateNav::grey_hive_v7_candidate();
        assert!(resolve_checkpoint(&contract(), &input("gh_cp_central", false, ChoiceState::Unresolved), &nav).is_ok());
    }

    #[test]
    fn result_map_null_is_terminal_and_does_not_call_nav() {
        let nav = CandidateNav::grey_hive_v7_candidate();
        let mut request = input("gh_cp_exit", true, ChoiceState::Unresolved);
        request.result_map_null = true;
        assert_eq!(resolve_checkpoint(&contract(), &request, &nav), Err(ResolveError::TerminalNotApplicable));
        assert_eq!(nav.calls(), 0);
    }

    #[test]
    fn contract_rejects_wrong_world_layout_and_unknown_id() {
        let nav = CandidateNav::grey_hive_v7_candidate();
        let mut wrong_world = input("gh_cp_airlock", false, ChoiceState::Unresolved);
        wrong_world.world_id = "mist_harbor".into();
        assert_eq!(resolve_checkpoint(&contract(), &wrong_world, &nav), Err(ResolveError::WrongWorld));
        let mut wrong_layout = input("gh_cp_airlock", false, ChoiceState::Unresolved);
        wrong_layout.layout_version = "ag22-v6-candidate".into();
        assert_eq!(resolve_checkpoint(&contract(), &wrong_layout, &nav), Err(ResolveError::WrongLayout));
        let unknown = input("gh_cp_not_real", false, ChoiceState::Unresolved);
        assert_eq!(resolve_checkpoint(&contract(), &unknown, &nav), Err(ResolveError::UnknownCheckpoint));
    }

    #[test]
    fn projection_errors_are_stable_and_never_fallback() {
        let missing = CandidateNav::grey_hive_v7_candidate().without_spawn("gh_cp_power");
        assert_eq!(resolve_checkpoint(&contract(), &input("gh_cp_power", false, ChoiceState::Unresolved), &missing), Err(ResolveError::MissingSpawn));
        let blocked = CandidateNav::grey_hive_v7_candidate().with_failure("gh_cp_power", ProjectionFailure::Blocked);
        assert_eq!(resolve_checkpoint(&contract(), &input("gh_cp_power", false, ChoiceState::Unresolved), &blocked), Err(ResolveError::Blocked));
        let unreachable = CandidateNav::grey_hive_v7_candidate().with_failure("gh_cp_power", ProjectionFailure::Unreachable);
        assert_eq!(resolve_checkpoint(&contract(), &input("gh_cp_power", false, ChoiceState::Unresolved), &unreachable), Err(ResolveError::Unreachable));
        let failed = CandidateNav::grey_hive_v7_candidate().with_failure("gh_cp_power", ProjectionFailure::Failed);
        assert_eq!(resolve_checkpoint(&contract(), &input("gh_cp_power", false, ChoiceState::Unresolved), &failed), Err(ResolveError::ProjectionFailed));
    }

    #[test]
    fn nonfinite_and_wrong_room_projection_are_rejected() {
        let nonfinite = CandidateNav::grey_hive_v7_candidate().with_sample(
            "gh_cp_power",
            ProjectionSample { room: "GH-A03", transform: FootTransform::new(f64::NAN, 0.0, 14.0, 0.0), candidate_only: true },
        );
        assert_eq!(resolve_checkpoint(&contract(), &input("gh_cp_power", false, ChoiceState::Unresolved), &nonfinite), Err(ResolveError::NonFinite));
        let wrong_room = CandidateNav::grey_hive_v7_candidate().with_sample(
            POST_BIO_ID,
            ProjectionSample { room: "GH-B04", transform: FootTransform::new(70.0, 0.0, 51.0, 0.0), candidate_only: true },
        );
        assert_eq!(resolve_checkpoint(&contract(), &input(POST_BIO_ID, true, ChoiceState::Unresolved), &wrong_room), Err(ResolveError::ProjectionFailed));
    }

    #[test]
    fn resolver_is_read_only_and_never_emits_gameplay_side_effects() {
        let nav = CandidateNav::grey_hive_v7_candidate();
        let request = input(POST_BIO_ID, true, ChoiceState::Unresolved);
        let before = request.clone();
        let _ = resolve_checkpoint(&contract(), &request, &nav);
        assert_eq!(request, before);
        assert!(!contract().production_scene_bound);
    }
}
