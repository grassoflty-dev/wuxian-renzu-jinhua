//! Pure v2 -> Save V3 semantic migration.
//!
//! This module intentionally has no filesystem, runtime, renderer, physics, or
//! navigation dependencies. Persistence owns bytes and hashes; this mapper only
//! receives an optional source hash from that future layer.

use crate::world_v3::{CheckpointRef, SaveV3, Transform};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;

pub const V2_VERSION: u64 = 2;
pub const V3_VERSION: u32 = 3;
pub const GREY_HIVE_WORLD: &str = "grey_hive";
pub const OPTIONAL_CHOICE_UNRESOLVED: &str = "unresolved";

const CHECKPOINTS: [&str; 6] = [
    "gh_cp_airlock",
    "gh_cp_power",
    "gh_cp_central",
    "gh_cp_post_bio",
    "gh_cp_pre_sentinel",
    "gh_cp_exit",
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SaveV3MigrationError {
    JsonInvalid,
    V2MissingField(&'static str),
    V2Version,
    V2World,
    V2MainlineContradiction(&'static str),
    V2Choice,
    V3Checkpoint,
    V3Save,
}

impl SaveV3MigrationError {
    pub fn code(&self) -> String {
        match self {
            Self::JsonInvalid => "E_JSON_INVALID".into(),
            Self::V2MissingField(field) => format!("E_V2_MISSING_FIELD:{field}"),
            Self::V2Version => "E_V2_VERSION".into(),
            Self::V2World => "E_V2_WORLD".into(),
            Self::V2MainlineContradiction(field) => {
                format!("E_V2_MAINLINE_CONTRADICTION:{field}")
            }
            Self::V2Choice => "E_V2_CHOICE".into(),
            Self::V3Checkpoint => "E_V3_CHECKPOINT".into(),
            Self::V3Save => "E_V3_SAVE".into(),
        }
    }
}

impl fmt::Display for SaveV3MigrationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.code())
    }
}

impl std::error::Error for SaveV3MigrationError {}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveV3Source {
    pub schema_version: u32,
    pub fixture_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_sha256: Option<String>,
    pub migration: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GateFacts {
    pub gate_a: String,
    pub gate_b: String,
    pub gate_b_requires: String,
    pub hive_choice_state: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveV3Campaign {
    pub world_id: String,
    pub mode: String,
    pub stage: String,
    pub source_map_was_null: bool,
    pub gate_facts: GateFacts,
    pub choice_policy: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveV3Progression {
    pub objectives: Value,
    pub flags: Value,
    pub decisions: Value,
    pub optional_choice: Option<String>,
    pub run_reward_allowed: Value,
    pub run_scrap: Value,
    pub enemies_defeated: Value,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveV3WorldResume {
    pub world_id: String,
    pub checkpoint_id: String,
    pub resume_policy: String,
    pub transform: Option<Transform>,
    pub transform_resolver: String,
}

/// Formal envelope around the TP-02 `SaveV3` primitive DTO.
///
/// The flattened primitive keeps the wire-level `schemaVersion`, `saveId`,
/// `checkpoint`, and `transform`; the semantic migration groups are explicit
/// and remain independent of persistence.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveV3Envelope {
    #[serde(flatten)]
    pub save: SaveV3,
    pub source: SaveV3Source,
    pub campaign: SaveV3Campaign,
    pub progression: SaveV3Progression,
    pub world_resume: SaveV3WorldResume,
}

pub fn validate_checkpoint_id(checkpoint_id: &str) -> Result<(), SaveV3MigrationError> {
    if CHECKPOINTS.contains(&checkpoint_id) {
        Ok(())
    } else {
        Err(SaveV3MigrationError::V3Checkpoint)
    }
}

pub fn validate_envelope(envelope: &SaveV3Envelope) -> Result<(), SaveV3MigrationError> {
    if envelope.save.schema_version() != V3_VERSION
        || envelope.source.schema_version != V2_VERSION as u32
        || envelope.world_resume.world_id != GREY_HIVE_WORLD
        || envelope.campaign.world_id != GREY_HIVE_WORLD
        || envelope.world_resume.transform.is_some()
    {
        return Err(SaveV3MigrationError::V3Save);
    }
    validate_checkpoint_id(&envelope.world_resume.checkpoint_id)
}

fn required<'a>(v2: &'a Value, field: &'static str) -> Result<&'a Value, SaveV3MigrationError> {
    v2.get(field)
        .ok_or(SaveV3MigrationError::V2MissingField(field))
}

fn flag_present(v2: &Value, wanted: &str) -> bool {
    v2["flags"]
        .as_array()
        .is_some_and(|flags| flags.iter().any(|flag| flag.as_str() == Some(wanted)))
}

fn objective_done(v2: &Value, wanted: &str) -> Option<bool> {
    v2["objectives"].as_array().and_then(|objectives| {
        objectives
            .iter()
            .find(|objective| objective["id"].as_str() == Some(wanted))
            .and_then(|objective| objective["done"].as_bool())
    })
}

fn completed(v2: &Value, wanted: &'static str) -> Result<bool, SaveV3MigrationError> {
    let flag = flag_present(v2, wanted);
    if let Some(done) = objective_done(v2, wanted) {
        if done != flag {
            return Err(SaveV3MigrationError::V2MainlineContradiction(wanted));
        }
    }
    Ok(flag)
}

fn optional_choice(v2: &Value) -> Result<Option<String>, SaveV3MigrationError> {
    match v2.get("optional_choice") {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(choice))
            if choice == "hive_help_baizhi" || choice == "hive_leave_sample" =>
        {
            Ok(Some(choice.clone()))
        }
        _ => Err(SaveV3MigrationError::V2Choice),
    }
}

fn stage_for(
    v2: &Value,
    lockdown_done: bool,
    power_done: bool,
) -> (&'static str, &'static str, &'static str) {
    if v2["mode"] == "result" || v2["map"].is_null() {
        return ("result", "gh_cp_exit", "result_then_hub");
    }
    let revision = v2["map"]["revision"].as_u64().unwrap_or_default();
    if lockdown_done && revision >= 150 {
        return ("pre_sentinel", "gh_cp_pre_sentinel", "world_resume");
    }
    if lockdown_done {
        return ("post_baizhi", "gh_cp_post_bio", "world_resume");
    }
    if power_done && revision >= 40 {
        return ("gate_a", "gh_cp_central", "world_resume");
    }
    if power_done {
        return ("post_power", "gh_cp_power", "world_resume");
    }
    ("initial", "gh_cp_airlock", "world_resume")
}

fn migrate_value(
    v2: &Value,
    save_id: &str,
    source_sha256: Option<&str>,
) -> Result<SaveV3Envelope, SaveV3MigrationError> {
    let version = required(v2, "version")?
        .as_u64()
        .ok_or(SaveV3MigrationError::V2Version)?;
    if version != V2_VERSION {
        return Err(SaveV3MigrationError::V2Version);
    }
    for field in [
        "mode",
        "current_world_id",
        "objectives",
        "flags",
        "decisions",
        "map",
    ] {
        required(v2, field)?;
    }
    if v2["current_world_id"].as_str() != Some(GREY_HIVE_WORLD) {
        return Err(SaveV3MigrationError::V2World);
    }
    let power_done = completed(v2, "hive_power")?;
    let lockdown_done = completed(v2, "hive_lockdown")?;
    let _ = completed(v2, "hive_rescue")?;
    let choice = optional_choice(v2)?;
    let (stage, checkpoint_id, resume_policy) = stage_for(v2, lockdown_done, power_done);
    validate_checkpoint_id(checkpoint_id)?;

    let checkpoint = CheckpointRef::new(GREY_HIVE_WORLD, checkpoint_id)
        .map_err(|_| SaveV3MigrationError::V3Checkpoint)?;
    let save = SaveV3::new(save_id, checkpoint, None).map_err(|_| SaveV3MigrationError::V3Save)?;
    let envelope = SaveV3Envelope {
        save,
        source: SaveV3Source {
            schema_version: V2_VERSION as u32,
            fixture_id: save_id.to_owned(),
            source_sha256: source_sha256.map(str::to_owned),
            migration: "v2_to_v3_optional_choice_tp03a".into(),
        },
        campaign: SaveV3Campaign {
            world_id: GREY_HIVE_WORLD.into(),
            mode: v2["mode"].as_str().unwrap_or_default().into(),
            stage: stage.into(),
            source_map_was_null: v2["map"].is_null(),
            gate_facts: GateFacts {
                gate_a: if power_done { "open" } else { "locked" }.into(),
                gate_b: if lockdown_done { "open" } else { "locked" }.into(),
                gate_b_requires: "hive_lockdown==completed".into(),
                hive_choice_state: choice
                    .as_deref()
                    .unwrap_or(OPTIONAL_CHOICE_UNRESOLVED)
                    .into(),
            },
            choice_policy: "optional".into(),
        },
        progression: SaveV3Progression {
            objectives: v2["objectives"].clone(),
            flags: v2["flags"].clone(),
            decisions: v2["decisions"].clone(),
            optional_choice: choice,
            run_reward_allowed: v2["run_reward_allowed"].clone(),
            run_scrap: v2["run_scrap"].clone(),
            enemies_defeated: v2["enemies_defeated"].clone(),
        },
        world_resume: SaveV3WorldResume {
            world_id: GREY_HIVE_WORLD.into(),
            checkpoint_id: checkpoint_id.into(),
            resume_policy: resume_policy.into(),
            transform: None,
            transform_resolver: "semantic_checkpoint_only_no_v2_px_py_scaling".into(),
        },
    };
    validate_envelope(&envelope)?;
    Ok(envelope)
}

pub fn migrate_v2_value(
    v2: &Value,
    save_id: &str,
    source_sha256: Option<&str>,
) -> Result<SaveV3Envelope, SaveV3MigrationError> {
    migrate_value(v2, save_id, source_sha256)
}

pub fn migrate_v2_json(
    bytes: &[u8],
    save_id: &str,
    source_sha256: Option<&str>,
) -> Result<SaveV3Envelope, SaveV3MigrationError> {
    let v2: Value = serde_json::from_slice(bytes).map_err(|_| SaveV3MigrationError::JsonInvalid)?;
    migrate_value(&v2, save_id, source_sha256)
}
