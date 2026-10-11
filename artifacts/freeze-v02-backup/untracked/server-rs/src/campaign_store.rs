//! Three-world save boundary.
//!
//! This store deliberately owns only data/campaign-v2.json. The historical
//! data/save.json file is outside this boundary and is never read, copied, or
//! overwritten by campaign code.

use crate::campaign::{self, CampaignState};
use serde_json::{json, Map, Value};
use std::{
    collections::BTreeSet,
    fs::{self, File, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

const AUTO_SAVE_INTERVAL: Duration = Duration::from_secs(2);

pub struct CampaignStore {
    pub state: CampaignState,
    path: PathBuf,
    load_warning: Option<String>,
    last_error: Option<String>,
    last_save: Instant,
}

impl CampaignStore {
    pub fn load(path: PathBuf) -> Self {
        let (state, load_warning) = match fs::read(&path) {
            Ok(bytes) => match decode_campaign_state(&bytes) {
                Ok(state) => (state, None),
                Err(error) => (
                    CampaignState::default(),
                    Some(format!(
                        "存档无效（{error}），原文件已保留。请先备份，或在标题页显式选择新建旅程。"
                    )),
                ),
            },
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                (CampaignState::default(), None)
            }
            Err(error) => (
                CampaignState::default(),
                Some(format!(
                    "无法访问存档（{}）：{error}。原文件未覆盖；请先检查权限，或显式选择新建旅程。",
                    path.display()
                )),
            ),
        };
        Self {
            state,
            path,
            load_warning,
            last_error: None,
            last_save: Instant::now(),
        }
    }

    pub fn view(&self) -> Value {
        let mut result = campaign::view(&self.state);
        self.add_save_diagnostics(&mut result);
        result
    }

    fn add_save_diagnostics(&self, result: &mut Value) {
        result["save_warning"] = json!(self.current_warning());
        result["has_save"] = json!(self.path.is_file());
        result["pending_save"] = json!(self.pending_path().exists());
        result["save_path"] = json!(self.path.to_string_lossy());
        result["pending_path"] = json!(self.pending_path().to_string_lossy());
        result["backup_path"] = json!(self.backup_path().to_string_lossy());
        result["read_only_recovery_required"] = json!(self.load_warning.is_some());
    }

    fn current_warning(&self) -> Option<String> {
        self.load_warning
            .as_ref()
            .cloned()
            .or_else(|| self.last_error.as_ref().cloned())
    }

    fn fail(&mut self, message: String) -> Result<(), String> {
        self.last_error = Some(message.clone());
        Err(message)
    }

    fn pending_path(&self) -> PathBuf {
        self.path.with_extension("json.pending")
    }

    fn backup_path(&self) -> PathBuf {
        self.path.with_extension("json.bak")
    }

    pub fn pending_path_for_diagnostics(&self) -> PathBuf {
        self.pending_path()
    }

    pub fn backup_path_for_recovery(&self) -> PathBuf {
        self.backup_path()
    }

    fn save(&mut self) -> Result<(), String> {
        if self.load_warning.is_some() {
            return self.fail(
                "原存档读取异常，自动写入已暂停；原文件未覆盖。请显式选择 new（会先创建时间戳备份）或手动恢复。"
                    .into(),
            );
        }

        let parent = match self.path.parent() {
            Some(parent) => parent,
            None => return self.fail("存档路径无效：缺少父目录。".into()),
        };
        if let Err(error) = fs::create_dir_all(parent) {
            return self.fail(format!("无法创建存档目录 {}：{error}", parent.display()));
        }

        let pending = self.pending_path();
        if pending.exists() {
            return self.fail(format!(
                "原子保存被暂停：待处理文件 {} 已存在，未覆盖原存档。请先检查该 pending 文件并按恢复说明处理。",
                pending.display()
            ));
        }

        let bytes = match serde_json::to_vec_pretty(&self.state) {
            Ok(bytes) => bytes,
            Err(error) => return self.fail(format!("存档序列化失败：{error}")),
        };
        if let Err(error) = write_new_file(&pending, &bytes) {
            return self.fail(format!(
                "原子保存写入 pending 失败：{error}；原文件未覆盖，pending 文件保留为诊断线索：{}",
                pending.display()
            ));
        }

        let staged_backup = self.backup_path().with_extension("json.bak.pending");
        if self.path.exists() {
            if !self.path.is_file() {
                return self.fail(format!(
                    "原子保存目标不是普通文件：{}；原文件未覆盖，pending 文件保留：{}",
                    self.path.display(),
                    pending.display()
                ));
            }
            if staged_backup.exists() {
                return self.fail(format!(
                    "原子保存被暂停：备份 pending 文件 {} 已存在；原文件未覆盖，主 pending 文件保留：{}",
                    staged_backup.display(),
                    pending.display()
                ));
            }
            if let Err(error) = copy_to_new_file(&self.path, &staged_backup) {
                return self.fail(format!(
                    "旧存档备份失败：{error}；原文件未覆盖，主 pending 文件保留：{}",
                    pending.display()
                ));
            }
            if let Err(error) = fs::rename(&staged_backup, self.backup_path()) {
                return self.fail(format!(
                    "旧存档备份原子替换失败：{error}；原文件未覆盖，主 pending 文件保留：{}，备份 pending 保留：{}",
                    pending.display(),
                    staged_backup.display()
                ));
            }
        }

        if let Err(error) = fs::rename(&pending, &self.path) {
            return self.fail(format!(
                "原子保存提交失败：{error}；原文件未覆盖，pending 文件保留：{}",
                pending.display()
            ));
        }

        self.last_save = Instant::now();
        self.last_error = None;
        Ok(())
    }

    fn archive_before_new(&self) -> Result<Option<PathBuf>, String> {
        if !self.path.is_file() {
            return Ok(None);
        }

        let parent = self
            .path
            .parent()
            .ok_or_else(|| "新建旅程失败：存档路径缺少父目录。".to_string())?;
        fs::create_dir_all(parent)
            .map_err(|error| format!("新建旅程前无法创建备份目录 {}：{error}", parent.display()))?;

        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        let archive = next_archive_path(parent, stamp)?;
        copy_to_new_file(&self.path, &archive).map_err(|error| {
            format!(
                "旧旅程时间戳备份失败：{error}；新建已取消，原文件仍保留：{}",
                self.path.display()
            )
        })?;
        Ok(Some(archive))
    }

    pub fn action(&mut self, action: &str, args: &Value) -> Value {
        if action == "new" {
            match self.archive_before_new() {
                Ok(_) => {
                    // A deliberate new journey is the only operation allowed
                    // to clear a read warning, and only after the raw old file
                    // has been archived successfully.
                    self.load_warning = None;
                    self.last_error = None;
                }
                Err(error) => {
                    self.last_error = Some(error.clone());
                    let mut result = self.view();
                    result["last_event"] = json!(error);
                    return result;
                }
            }
        }

        let mut result = campaign::act(&mut self.state, action, args);
        if self.should_save_now(action) {
            if let Err(error) = self.save() {
                result["save_warning"] = json!(error);
            }
        }
        self.add_save_diagnostics(&mut result);
        result
    }

    fn should_save_now(&self, action: &str) -> bool {
        if is_high_frequency_action(action) {
            // Establish a first save promptly, then avoid writing and copying
            // the backup on every movement/combat frame.
            !self.path.is_file() || self.last_save.elapsed() >= AUTO_SAVE_INTERVAL
        } else {
            // Checkpoint actions include enter/interact/choice/extract/retreat,
            // upgrade/equip/continue/new/restart. Unknown actions are also
            // saved immediately for safety.
            true
        }
    }

    pub fn flush(&mut self) -> Result<(), String> {
        self.save()
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

fn is_high_frequency_action(action: &str) -> bool {
    matches!(
        action,
        "tick"
            | "move"
            | "move_intent"
            | "attack"
            | "skill"
            | "cast"
            | "cast_skill"
            | "use_skill"
            | "dash"
            | "dodge"
            | "roll"
            | "sprint"
    ) || action.starts_with("move_")
        || action.starts_with("attack_")
        || action.starts_with("skill_")
        || action.starts_with("dash_")
}

fn write_new_file(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(())
}

fn copy_to_new_file(source: &Path, destination: &Path) -> io::Result<()> {
    let mut source_file = File::open(source)?;
    let mut destination_file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(destination)?;
    io::copy(&mut source_file, &mut destination_file)?;
    destination_file.sync_all()?;
    Ok(())
}

fn next_archive_path(parent: &Path, stamp: u128) -> Result<PathBuf, String> {
    for suffix in 0..10_000_u32 {
        let name = if suffix == 0 {
            format!("campaign-v2-before-new-{stamp}.json")
        } else {
            format!("campaign-v2-before-new-{stamp}-{suffix}.json")
        };
        let candidate = parent.join(name);
        if !candidate.exists() {
            return Ok(candidate);
        }
    }
    Err(format!(
        "新建旅程失败：无法为时间戳备份找到未占用文件名（目录 {}）",
        parent.display()
    ))
}

const REQUIRED_STATE_FIELDS: &[&str] = &[
    "version",
    "mode",
    "title",
    "subtitle",
    "player",
    "worlds",
    "upgrades",
    "skills",
    "objectives",
    "dialog",
    "result",
    "last_event",
    "effect",
    "map",
    "decisions",
    "current_world_id",
    "flags",
    "spoils_claimed",
    "run_reward_allowed",
    "run_scrap",
    "enemies_defeated",
    "run_ticks",
    "run_damage_taken",
    "optional_choice",
    "shield",
    "attack_cooldown",
    "steam_phase",
    "spawn_protected",
];

const VALID_WORLD_IDS: [&str; 3] = ["grey_hive", "mist_harbor", "clockworks"];

/// 更名前使用的存档格式标识。既有的 data/campaign-v2.json 里仍然是这个名字，
/// 校验时必须继续接受，否则老存档会被判为无效。
const LEGACY_TITLE: &str = "余界：归航协议";

fn decode_campaign_state(bytes: &[u8]) -> Result<CampaignState, String> {
    let raw: Value =
        serde_json::from_slice(bytes).map_err(|error| format!("JSON 解析失败：{error}"))?;
    validate_raw_state(&raw)?;
    let mut state: CampaignState =
        serde_json::from_value(raw).map_err(|error| format!("schema 反序列化失败：{error}"))?;
    // 旧存档里的 title 是更名前的格式标识。读入后统一规范化为当前标题，
    // 老存档一旦被写回就自动迁移到新标识，不需要额外的迁移步骤。
    state.title = campaign::TITLE.into();
    Ok(state)
}

fn validate_raw_state(raw: &Value) -> Result<(), String> {
    let root = as_object(raw, "state")?;
    for key in REQUIRED_STATE_FIELDS {
        require_field(root, key, "state")?;
    }

    if require_i64(root, "version", "state")? != 2 {
        return Err("version 必须为 2，拒绝旧版或缺失版本存档".into());
    }
    // title 既是游戏标题，也是存档格式标识，作用等同于 save key：它已经被写进所有
    // 既有的 data/campaign-v2.json。这里对新旧两种标识都放行，老存档因此仍能读入；
    // 写入端始终使用 campaign::TITLE，新存档写的是新名。
    let title = require_string(root, "title", "state")?;
    if title != campaign::TITLE && title != LEGACY_TITLE {
        return Err("title 与当前存档格式标识不匹配".into());
    }
    require_non_empty_string(root, "subtitle", "state")?;
    require_non_empty_string(root, "last_event", "state")?;

    let mode = require_string(root, "mode", "state")?;
    if !matches!(mode, "hub" | "world" | "dialog" | "dead" | "result") {
        return Err(format!("state.mode 无效：{mode}"));
    }
    validate_player(require_field(root, "player", "state")?)?;
    validate_worlds(require_field(root, "worlds", "state")?)?;
    validate_upgrades(require_field(root, "upgrades", "state")?)?;
    validate_skills(require_field(root, "skills", "state")?)?;
    validate_objectives(require_field(root, "objectives", "state")?)?;
    validate_decisions(require_field(root, "decisions", "state")?)?;
    validate_string_list(require_field(root, "flags", "state")?, "state.flags")?;
    validate_spoils(require_field(root, "spoils_claimed", "state")?)?;
    validate_non_negative_counter(root, "run_scrap")?;
    validate_non_negative_counter(root, "enemies_defeated")?;
    validate_non_negative_counter(root, "run_ticks")?;
    validate_non_negative_counter(root, "run_damage_taken")?;
    validate_non_negative_counter(root, "shield")?;
    validate_non_negative_counter(root, "attack_cooldown")?;
    validate_non_negative_counter(root, "steam_phase")?;
    require_bool(root, "run_reward_allowed", "state")?;
    require_bool(root, "spawn_protected", "state")?;
    validate_nullable_string(root, "optional_choice", "state")?;

    let current_world = validate_current_world(root)?;
    validate_mode_relationships(
        mode,
        current_world,
        require_field(root, "map", "state")?,
        require_field(root, "dialog", "state")?,
        require_field(root, "result", "state")?,
    )?;
    validate_effect(require_field(root, "effect", "state")?)?;
    Ok(())
}

fn as_object<'a>(value: &'a Value, context: &str) -> Result<&'a Map<String, Value>, String> {
    value
        .as_object()
        .ok_or_else(|| format!("{context} 必须是 JSON object"))
}

fn require_field<'a>(
    object: &'a Map<String, Value>,
    key: &str,
    context: &str,
) -> Result<&'a Value, String> {
    object
        .get(key)
        .ok_or_else(|| format!("{context} 缺少关键字段 {key}"))
}

fn require_string<'a>(
    object: &'a Map<String, Value>,
    key: &str,
    context: &str,
) -> Result<&'a str, String> {
    require_field(object, key, context)?
        .as_str()
        .ok_or_else(|| format!("{context}.{key} 必须是 string"))
}

fn require_non_empty_string<'a>(
    object: &'a Map<String, Value>,
    key: &str,
    context: &str,
) -> Result<&'a str, String> {
    let value = require_string(object, key, context)?;
    if value.trim().is_empty() {
        return Err(format!("{context}.{key} 不能为空"));
    }
    Ok(value)
}

fn require_i64(object: &Map<String, Value>, key: &str, context: &str) -> Result<i64, String> {
    require_field(object, key, context)?
        .as_i64()
        .ok_or_else(|| format!("{context}.{key} 必须是整数"))
}

fn require_bool(object: &Map<String, Value>, key: &str, context: &str) -> Result<bool, String> {
    require_field(object, key, context)?
        .as_bool()
        .ok_or_else(|| format!("{context}.{key} 必须是 boolean"))
}

fn require_array<'a>(
    object: &'a Map<String, Value>,
    key: &str,
    context: &str,
) -> Result<&'a Vec<Value>, String> {
    require_field(object, key, context)?
        .as_array()
        .ok_or_else(|| format!("{context}.{key} 必须是 array"))
}

fn validate_player(value: &Value) -> Result<(), String> {
    let player = as_object(value, "state.player")?;
    require_non_empty_string(player, "name", "state.player")?;
    let hp = require_i64(player, "hp", "state.player")?;
    let max_hp = require_i64(player, "max_hp", "state.player")?;
    let energy = require_i64(player, "energy", "state.player")?;
    let max_energy = require_i64(player, "max_energy", "state.player")?;
    if max_hp <= 0 || hp < 0 || hp > max_hp {
        return Err("state.player 的 hp/max_hp 范围无效".into());
    }
    if max_energy <= 0 || energy < 0 || energy > max_energy {
        return Err("state.player 的 energy/max_energy 范围无效".into());
    }
    for key in ["medkits", "scrap", "level"] {
        if require_i64(player, key, "state.player")? < 0 {
            return Err(format!("state.player.{key} 不能为负数"));
        }
    }
    if require_i64(player, "level", "state.player")? == 0 {
        return Err("state.player.level 必须至少为 1".into());
    }
    let build = require_string(player, "build", "state.player")?;
    if !matches!(build, "vanguard" | "echo") {
        return Err(format!("state.player.build 无效：{build}"));
    }
    Ok(())
}

fn known_world_name(id: &str) -> Option<&'static str> {
    match id {
        "grey_hive" => Some("灰巢设施"),
        "mist_harbor" => Some("雾港余烬"),
        "clockworks" => Some("钟骨工厂"),
        _ => None,
    }
}

fn validate_worlds(value: &Value) -> Result<(), String> {
    let worlds = value
        .as_array()
        .ok_or_else(|| "state.worlds 必须是 array".to_string())?;
    if worlds.len() != VALID_WORLD_IDS.len() {
        return Err("state.worlds 必须完整包含三个当前世界".into());
    }
    let mut ids = BTreeSet::new();
    for (index, value) in worlds.iter().enumerate() {
        let context = format!("state.worlds[{index}]");
        let world = as_object(value, &context)?;
        let id = require_non_empty_string(world, "id", &context)?;
        let expected_name =
            known_world_name(id).ok_or_else(|| format!("{context}.id 不是当前世界"))?;
        if !ids.insert(id) {
            return Err(format!("{context}.id 重复：{id}"));
        }
        if require_string(world, "name", &context)? != expected_name {
            return Err(format!("{context}.name 与 id 不匹配"));
        }
        for key in ["kicker", "description", "mechanic", "accent"] {
            require_non_empty_string(world, key, &context)?;
        }
        require_bool(world, "cleared", &context)?;
        if !require_bool(world, "available", &context)? {
            return Err(format!("{context}.available 不能关闭当前世界"));
        }
    }
    let expected = VALID_WORLD_IDS.into_iter().collect::<BTreeSet<_>>();
    if ids != expected {
        return Err("state.worlds 缺少或混入了当前三世界之外的 id".into());
    }
    Ok(())
}

fn validate_upgrades(value: &Value) -> Result<(), String> {
    let upgrades = value
        .as_array()
        .ok_or_else(|| "state.upgrades 必须是 array".to_string())?;
    if upgrades.is_empty() {
        return Err("state.upgrades 不能为空".into());
    }
    let mut ids = BTreeSet::new();
    for (index, value) in upgrades.iter().enumerate() {
        let context = format!("state.upgrades[{index}]");
        let upgrade = as_object(value, &context)?;
        let id = require_non_empty_string(upgrade, "id", &context)?;
        if !ids.insert(id) {
            return Err(format!("{context}.id 重复：{id}"));
        }
        require_non_empty_string(upgrade, "name", &context)?;
        require_non_empty_string(upgrade, "description", &context)?;
        if require_i64(upgrade, "cost", &context)? <= 0 {
            return Err(format!("{context}.cost 必须为正数"));
        }
        let level = require_i64(upgrade, "level", &context)?;
        let max_level = require_i64(upgrade, "max_level", &context)?;
        if max_level < 0 || level < 0 || level > max_level || max_level > 100 {
            return Err(format!("{context} 的 level/max_level 范围无效"));
        }
    }
    Ok(())
}

fn validate_skills(value: &Value) -> Result<(), String> {
    let skills = value
        .as_array()
        .ok_or_else(|| "state.skills 必须是 array".to_string())?;
    if skills.is_empty() {
        return Err("state.skills 不能为空".into());
    }
    let mut ids = BTreeSet::new();
    for (index, value) in skills.iter().enumerate() {
        let context = format!("state.skills[{index}]");
        let skill = as_object(value, &context)?;
        let id = require_non_empty_string(skill, "id", &context)?;
        if !ids.insert(id) {
            return Err(format!("{context}.id 重复：{id}"));
        }
        for key in ["name", "key", "description"] {
            require_non_empty_string(skill, key, &context)?;
        }
        if require_i64(skill, "cost", &context)? < 0
            || require_i64(skill, "cooldown", &context)? < 0
        {
            return Err(format!("{context} 的 cost/cooldown 不能为负数"));
        }
        let cooldown_left = require_i64(skill, "cooldown_left", &context)?;
        if cooldown_left < 0 {
            return Err(format!("{context}.cooldown_left 不能为负数"));
        }
        let ready = require_bool(skill, "ready", &context)?;
        if ready != (cooldown_left == 0) {
            return Err(format!("{context}.ready 与 cooldown_left 不一致"));
        }
    }
    Ok(())
}

fn validate_objectives(value: &Value) -> Result<(), String> {
    let objectives = value
        .as_array()
        .ok_or_else(|| "state.objectives 必须是 array".to_string())?;
    let mut ids = BTreeSet::new();
    for (index, value) in objectives.iter().enumerate() {
        let context = format!("state.objectives[{index}]");
        let objective = as_object(value, &context)?;
        let id = require_non_empty_string(objective, "id", &context)?;
        if !ids.insert(id) {
            return Err(format!("{context}.id 重复：{id}"));
        }
        require_non_empty_string(objective, "title", &context)?;
        require_non_empty_string(objective, "description", &context)?;
        require_bool(objective, "done", &context)?;
        require_bool(objective, "optional", &context)?;
    }
    Ok(())
}

fn validate_decisions(value: &Value) -> Result<(), String> {
    let decisions = value
        .as_object()
        .ok_or_else(|| "state.decisions 必须是 object".to_string())?;
    for (world_id, choice_id) in decisions {
        if known_world_name(world_id).is_none() {
            return Err(format!("state.decisions 含未知世界：{world_id}"));
        }
        let choice_id = choice_id
            .as_str()
            .filter(|choice| !choice.trim().is_empty())
            .ok_or_else(|| format!("state.decisions.{world_id} 必须是非空 string"))?;
        let valid = matches!(
            (world_id.as_str(), choice_id),
            ("grey_hive", "hive_help_baizhi" | "hive_leave_sample")
                | ("mist_harbor", "harbor_save_fuel" | "harbor_burn_fuel")
                | ("clockworks", "clock_free_mengque" | "clock_take_core")
        );
        if !valid {
            return Err(format!(
                "state.decisions.{world_id} 的 choice 无效：{choice_id}"
            ));
        }
    }
    Ok(())
}

fn validate_string_list(value: &Value, context: &str) -> Result<(), String> {
    let values = value
        .as_array()
        .ok_or_else(|| format!("{context} 必须是 array"))?;
    let mut unique = BTreeSet::new();
    for (index, value) in values.iter().enumerate() {
        let text = value
            .as_str()
            .ok_or_else(|| format!("{context}[{index}] 必须是 string"))?;
        if text.trim().is_empty() || !unique.insert(text) {
            return Err(format!("{context}[{index}] 为空或重复"));
        }
    }
    Ok(())
}

fn validate_spoils(value: &Value) -> Result<(), String> {
    validate_string_list(value, "state.spoils_claimed")?;
    let values = value.as_array().expect("validated array");
    for (index, value) in values.iter().enumerate() {
        let id = value.as_str().expect("validated string");
        if known_world_name(id).is_none() {
            return Err(format!("state.spoils_claimed[{index}] 不是当前世界"));
        }
    }
    Ok(())
}

fn validate_non_negative_counter(object: &Map<String, Value>, key: &str) -> Result<(), String> {
    if require_i64(object, key, "state")? < 0 {
        return Err(format!("state.{key} 不能为负数"));
    }
    Ok(())
}

fn validate_current_world<'a>(root: &'a Map<String, Value>) -> Result<Option<&'a str>, String> {
    let current_world = validate_nullable_string(root, "current_world_id", "state")?;
    if let Some(id) = current_world {
        if known_world_name(id).is_none() {
            return Err(format!("state.current_world_id 无效：{id}"));
        }
    }
    Ok(current_world)
}

fn validate_nullable_string<'a>(
    object: &'a Map<String, Value>,
    key: &str,
    context: &str,
) -> Result<Option<&'a str>, String> {
    let value = require_field(object, key, context)?;
    if value.is_null() {
        return Ok(None);
    }
    value
        .as_str()
        .map(Some)
        .ok_or_else(|| format!("{context}.{key} 必须是 string 或 null"))
}

fn validate_mode_relationships(
    mode: &str,
    current_world: Option<&str>,
    map_value: &Value,
    dialog_value: &Value,
    result_value: &Value,
) -> Result<(), String> {
    match mode {
        "hub" => {
            if current_world.is_some()
                || !map_value.is_null()
                || !dialog_value.is_null()
                || !result_value.is_null()
            {
                return Err("hub 存档必须没有 current_world_id/map/dialog/result".into());
            }
        }
        "world" | "dead" => {
            let world_id =
                current_world.ok_or_else(|| format!("{mode} 存档缺少 current_world_id"))?;
            if dialog_value.is_null() == false || !result_value.is_null() {
                return Err(format!("{mode} 存档的 dialog/result 关联无效"));
            }
            validate_map(map_value, world_id)?;
        }
        "dialog" => {
            let world_id =
                current_world.ok_or_else(|| "dialog 存档缺少 current_world_id".to_string())?;
            validate_map(map_value, world_id)?;
            validate_dialog(dialog_value)?;
            if !result_value.is_null() {
                return Err("dialog 存档不能同时有 result".into());
            }
        }
        "result" => {
            let world_id =
                current_world.ok_or_else(|| "result 存档缺少 current_world_id".to_string())?;
            if !map_value.is_null() || !dialog_value.is_null() {
                return Err("result 存档必须没有 map/dialog".into());
            }
            validate_result(result_value, world_id)?;
        }
        _ => unreachable!("mode was checked before relationship validation"),
    }
    Ok(())
}

fn validate_map(value: &Value, current_world: &str) -> Result<(), String> {
    let map = as_object(value, "state.map")?;
    for key in [
        "world_id",
        "world_name",
        "w",
        "h",
        "floor",
        "tiles",
        "px",
        "py",
        "revision",
        "explored",
        "enemies",
        "npcs",
        "points",
        "zones",
        "portals",
        "gates",
        "props",
        "nearby",
    ] {
        require_field(map, key, "state.map")?;
    }
    if require_string(map, "world_id", "state.map")? != current_world {
        return Err("state.map.world_id 与 current_world_id 不匹配".into());
    }
    let expected_name = known_world_name(current_world).expect("validated world id");
    if require_string(map, "world_name", "state.map")? != expected_name {
        return Err("state.map.world_name 与 world_id 不匹配".into());
    }
    let width = require_i64(map, "w", "state.map")?;
    let height = require_i64(map, "h", "state.map")?;
    if !(1..=256).contains(&width) || !(1..=256).contains(&height) {
        return Err("state.map.w/h 超出安全尺寸范围 1..=256".into());
    }
    let tiles = require_array(map, "tiles", "state.map")?;
    if tiles.len() != height as usize {
        return Err("state.map.tiles 行数与 h 不一致".into());
    }
    for (row_index, row) in tiles.iter().enumerate() {
        let row = row
            .as_str()
            .ok_or_else(|| format!("state.map.tiles[{row_index}] 必须是 string"))?;
        if row.as_bytes().len() != width as usize {
            return Err(format!(
                "state.map.tiles[{row_index}] 行长与 w 不一致（按字节计）"
            ));
        }
        if row
            .as_bytes()
            .iter()
            .any(|tile| !matches!(tile, b'.' | b'#' | b'~'))
        {
            return Err(format!("state.map.tiles[{row_index}] 含未知 tile"));
        }
    }
    let px = require_i64(map, "px", "state.map")?;
    let py = require_i64(map, "py", "state.map")?;
    if px < 0 || px >= width || py < 0 || py >= height {
        return Err("state.map.px/py 超出地图范围".into());
    }
    if tiles[py as usize]
        .as_str()
        .expect("validated tile row")
        .as_bytes()[px as usize]
        != b'.'
    {
        return Err("state.map.px/py 必须落在可行走地板上".into());
    }
    let floor = require_i64(map, "floor", "state.map")?;
    let revision = require_i64(map, "revision", "state.map")?;
    if !(0..=64).contains(&floor) || revision < 0 {
        return Err("state.map.floor/revision 范围无效".into());
    }
    validate_explored(require_field(map, "explored", "state.map")?, width, height)?;
    validate_enemy_entries(require_field(map, "enemies", "state.map")?, width, height)?;
    validate_basic_position_entries(
        require_field(map, "npcs", "state.map")?,
        "state.map.npcs",
        width,
        height,
        &["id", "name", "kind"],
    )?;
    validate_basic_position_entries(
        require_field(map, "points", "state.map")?,
        "state.map.points",
        width,
        height,
        &["id", "name", "kind", "done"],
    )?;
    validate_basic_position_entries(
        require_field(map, "portals", "state.map")?,
        "state.map.portals",
        width,
        height,
        &["id", "name", "kind", "active"],
    )?;
    validate_basic_position_entries(
        require_field(map, "gates", "state.map")?,
        "state.map.gates",
        width,
        height,
        &["id", "name", "locked"],
    )?;
    validate_zone_entries(require_field(map, "zones", "state.map")?, width, height)?;
    validate_prop_entries(require_field(map, "props", "state.map")?, width, height)?;
    require_array(map, "nearby", "state.map")?;
    Ok(())
}

fn validate_explored(value: &Value, width: i64, height: i64) -> Result<(), String> {
    let values = value
        .as_array()
        .ok_or_else(|| "state.map.explored 必须是 array".to_string())?;
    let mut unique = BTreeSet::new();
    for (index, value) in values.iter().enumerate() {
        let text = value
            .as_str()
            .ok_or_else(|| format!("state.map.explored[{index}] 必须是 string"))?;
        let mut parts = text.split(':');
        let x = parts
            .next()
            .and_then(|part| part.parse::<i64>().ok())
            .ok_or_else(|| format!("state.map.explored[{index}] 坐标无效"))?;
        let y = parts
            .next()
            .and_then(|part| part.parse::<i64>().ok())
            .ok_or_else(|| format!("state.map.explored[{index}] 坐标无效"))?;
        if parts.next().is_some() || x < 0 || x >= width || y < 0 || y >= height {
            return Err(format!("state.map.explored[{index}] 超出地图范围"));
        }
        if !unique.insert(text) {
            return Err(format!("state.map.explored[{index}] 重复"));
        }
    }
    Ok(())
}

fn validate_enemy_entries(value: &Value, width: i64, height: i64) -> Result<(), String> {
    let entries = value
        .as_array()
        .ok_or_else(|| "state.map.enemies 必须是 array".to_string())?;
    let mut ids = BTreeSet::new();
    for (index, value) in entries.iter().enumerate() {
        let context = format!("state.map.enemies[{index}]");
        let enemy = as_object(value, &context)?;
        validate_position_and_identity(enemy, &context, width, height, &["id", "name", "kind"])?;
        let id = require_string(enemy, "id", &context)?;
        if !ids.insert(id) {
            return Err(format!("{context}.id 重复"));
        }
        let hp = require_i64(enemy, "hp", &context)?;
        let max_hp = require_i64(enemy, "max_hp", &context)?;
        if max_hp <= 0 || hp < 0 || hp > max_hp {
            return Err(format!("{context} 的 hp/max_hp 范围无效"));
        }
        require_bool(enemy, "alive", &context)?;
        if require_i64(enemy, "attack_clock", &context)? < 0 {
            return Err(format!("{context}.attack_clock 不能为负数"));
        }
        validate_telegraph(
            require_field(enemy, "telegraph", &context)?,
            width,
            height,
            &context,
        )?;
    }
    Ok(())
}

fn validate_basic_position_entries(
    value: &Value,
    context: &str,
    width: i64,
    height: i64,
    extra_fields: &[&str],
) -> Result<(), String> {
    let entries = value
        .as_array()
        .ok_or_else(|| format!("{context} 必须是 array"))?;
    let mut ids = BTreeSet::new();
    for (index, value) in entries.iter().enumerate() {
        let entry_context = format!("{context}[{index}]");
        let entry = as_object(value, &entry_context)?;
        validate_position_and_identity(entry, &entry_context, width, height, extra_fields)?;
        let id = require_string(entry, "id", &entry_context)?;
        if !ids.insert(id) {
            return Err(format!("{entry_context}.id 重复"));
        }
    }
    Ok(())
}

fn validate_position_and_identity(
    object: &Map<String, Value>,
    context: &str,
    width: i64,
    height: i64,
    fields: &[&str],
) -> Result<(), String> {
    for key in fields {
        let value = require_field(object, key, context)?;
        match *key {
            "id" | "name" | "kind" => {
                if value
                    .as_str()
                    .map(str::trim)
                    .filter(|text| !text.is_empty())
                    .is_none()
                {
                    return Err(format!("{context}.{key} 必须是非空 string"));
                }
            }
            "done" | "active" | "locked" => {
                if !value.is_boolean() {
                    return Err(format!("{context}.{key} 必须是 boolean"));
                }
            }
            _ => {}
        }
    }
    require_non_empty_string(object, "id", context)?;
    require_non_empty_string(object, "name", context)?;
    let x = require_i64(object, "x", context)?;
    let y = require_i64(object, "y", context)?;
    if x < 0 || x >= width || y < 0 || y >= height {
        return Err(format!("{context}.x/y 超出地图范围"));
    }
    Ok(())
}

fn validate_zone_entries(value: &Value, width: i64, height: i64) -> Result<(), String> {
    let entries = value
        .as_array()
        .ok_or_else(|| "state.map.zones 必须是 array".to_string())?;
    for (index, value) in entries.iter().enumerate() {
        let context = format!("state.map.zones[{index}]");
        let zone = as_object(value, &context)?;
        require_non_empty_string(zone, "kind", &context)?;
        let x = require_i64(zone, "x", &context)?;
        let y = require_i64(zone, "y", &context)?;
        let w = require_i64(zone, "w", &context)?;
        let h = require_i64(zone, "h", &context)?;
        if w <= 0 || h <= 0 || x < 0 || y < 0 || x + w > width || y + h > height {
            return Err(format!("{context} 的范围超出地图"));
        }
        require_bool(zone, "active", &context)?;
        validate_telegraph(
            require_field(zone, "telegraph", &context)?,
            width,
            height,
            &context,
        )?;
    }
    Ok(())
}

fn validate_prop_entries(value: &Value, width: i64, height: i64) -> Result<(), String> {
    let entries = value
        .as_array()
        .ok_or_else(|| "state.map.props 必须是 array".to_string())?;
    for (index, value) in entries.iter().enumerate() {
        let context = format!("state.map.props[{index}]");
        let prop = as_object(value, &context)?;
        require_non_empty_string(prop, "kind", &context)?;
        let x = require_i64(prop, "x", &context)?;
        let y = require_i64(prop, "y", &context)?;
        if x < 0 || x >= width || y < 0 || y >= height {
            return Err(format!("{context}.x/y 超出地图范围"));
        }
        for key in ["w", "h", "rotation"] {
            let value = require_field(prop, key, &context)?;
            if !value.is_null() {
                let number = value
                    .as_i64()
                    .ok_or_else(|| format!("{context}.{key} 必须是整数或 null"))?;
                if key != "rotation" && number <= 0 {
                    return Err(format!("{context}.{key} 必须为正数"));
                }
            }
        }
    }
    Ok(())
}

fn validate_telegraph(value: &Value, width: i64, height: i64, context: &str) -> Result<(), String> {
    if value.is_null() {
        return Ok(());
    }
    let telegraph = as_object(value, &format!("{context}.telegraph"))?;
    let x = require_i64(telegraph, "x", &format!("{context}.telegraph"))?;
    let y = require_i64(telegraph, "y", &format!("{context}.telegraph"))?;
    let radius = require_i64(telegraph, "radius", &format!("{context}.telegraph"))?;
    let remaining = require_i64(telegraph, "remaining", &format!("{context}.telegraph"))?;
    if x < 0 || x >= width || y < 0 || y >= height || radius < 0 || remaining < 0 {
        return Err(format!("{context}.telegraph 范围无效"));
    }
    Ok(())
}

fn validate_dialog(value: &Value) -> Result<(), String> {
    let dialog = as_object(value, "state.dialog")?;
    for key in ["title", "speaker", "body"] {
        require_non_empty_string(dialog, key, "state.dialog")?;
    }
    let choices = require_array(dialog, "choices", "state.dialog")?;
    if choices.is_empty() {
        return Err("state.dialog.choices 不能为空".into());
    }
    let mut ids = BTreeSet::new();
    for (index, value) in choices.iter().enumerate() {
        let context = format!("state.dialog.choices[{index}]");
        let choice = as_object(value, &context)?;
        let id = require_non_empty_string(choice, "id", &context)?;
        if !ids.insert(id) {
            return Err(format!("{context}.id 重复"));
        }
        require_non_empty_string(choice, "label", &context)?;
        require_non_empty_string(choice, "description", &context)?;
    }
    Ok(())
}

fn validate_result(value: &Value, current_world: &str) -> Result<(), String> {
    let result = as_object(value, "state.result")?;
    for key in ["title", "world_name", "summary", "rank"] {
        require_non_empty_string(result, key, "state.result")?;
    }
    if require_string(result, "world_name", "state.result")?
        != known_world_name(current_world).expect("validated world id")
    {
        return Err("state.result.world_name 与 current_world_id 不匹配".into());
    }
    if require_i64(result, "reward", "state.result")? < 0 {
        return Err("state.result.reward 不能为负数".into());
    }
    require_bool(result, "all_complete", "state.result")?;
    if !require_field(result, "stats", "state.result")?.is_object() {
        return Err("state.result.stats 必须是 object".into());
    }
    Ok(())
}

fn validate_effect(value: &Value) -> Result<(), String> {
    if value.is_null() {
        return Ok(());
    }
    let effect = as_object(value, "state.effect")?;
    require_non_empty_string(effect, "kind", "state.effect")?;
    for key in ["x", "y", "dx", "dy"] {
        let number = require_i64(effect, key, "state.effect")?;
        if !(-256..=256).contains(&number) {
            return Err(format!("state.effect.{key} 超出范围"));
        }
    }
    Ok(())
}
