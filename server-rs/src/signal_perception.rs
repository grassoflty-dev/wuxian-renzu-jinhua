//! Read-only signal-position projection. This primitive is NOT enabled by native
//! spawning yet; the paired ranged-controller slice owns its opt-in and context.
//! Output points are visual alternatives for ONE actor, never damage targets.
use crate::world_v3::{ActorAiState, Vec3};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SignalProjectionMode { Disabled, Unmapped, AcousticMapping }

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SignalPerceptionConfig { pub echo_distance_m: f32 }
impl Default for SignalPerceptionConfig {
    fn default() -> Self { Self { echo_distance_m: 0.65 } } // Development tuning.
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SignalPositionProjection {
    /// One precise point, or two equally presented alternatives. No true-point flag.
    pub positions_m: Vec<Vec3>,
    pub precise: bool,
    pub uncertainty_radius_m: f32,
}
fn precise(position: Vec3) -> SignalPositionProjection {
    SignalPositionProjection { positions_m: vec![position], precise: true, uncertainty_radius_m: 0.0 }
}
fn finite(position: Vec3) -> bool {
    [position.x_m, position.y_m, position.z_m].into_iter().all(f32::is_finite)
}

/// `mode` comes from controller admission and effective Rust capability rules.
/// `can_present` is a read-only geometry/standing predicate supplied by the scene.
/// No frame clock or RNG is accepted: a phase's ambiguity remains stable on pause,
/// repeated snapshots and Continue. Cast/stagger warnings always remain precise.
pub fn project_signal_positions(
    actor_id: &str,
    actual_position: Vec3,
    state: ActorAiState,
    phase_serial: u64,
    mode: SignalProjectionMode,
    config: SignalPerceptionConfig,
    can_present: impl Fn(Vec3) -> bool,
) -> Result<Option<SignalPositionProjection>, String> {
    if mode == SignalProjectionMode::Disabled { return Ok(None); }
    if actor_id.trim().is_empty() || actor_id.len() > 256 || !finite(actual_position)
        || !config.echo_distance_m.is_finite() || !(0.0..=2.0).contains(&config.echo_distance_m)
        || config.echo_distance_m == 0.0 {
        return Err("E_SIGNAL_PROJECTION_INPUT".into());
    }
    if !can_present(actual_position) { return Err("E_SIGNAL_PROJECTION_GEOMETRY".into()); }
    if mode == SignalProjectionMode::AcousticMapping
        || matches!(state, ActorAiState::Windup | ActorAiState::Active | ActorAiState::Stagger | ActorAiState::Dead) {
        return Ok(Some(precise(actual_position)));
    }
    let mut seed = 0xcbf29ce484222325_u64 ^ phase_serial.rotate_left(17);
    for byte in actor_id.bytes() { seed = (seed ^ u64::from(byte)).wrapping_mul(0x100000001b3); }
    let d = config.echo_distance_m;
    let offsets = [[d, 0.0], [0.0, d], [-d, 0.0], [0.0, -d]];
    for step in 0..offsets.len() {
        let [dx, dz] = offsets[((seed as usize) % offsets.len() + step) % offsets.len()];
        let alternative = Vec3 { x_m: actual_position.x_m + dx, z_m: actual_position.z_m + dz, ..actual_position };
        if !finite(alternative) || !can_present(alternative) { continue; }
        let positions_m = if seed & 4 == 0 { vec![actual_position, alternative] } else { vec![alternative, actual_position] };
        return Ok(Some(SignalPositionProjection { positions_m, precise: false, uncertainty_radius_m: d }));
    }
    // A wall/edge must not manufacture an impossible second body. Geometry can
    // make an unmapped actor locally unambiguous without granting a capability.
    Ok(Some(precise(actual_position)))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn p(x:f32,y:f32,z:f32)->Vec3 { Vec3::new(x,y,z).unwrap() }
    const ID:&str="mh_signal_yard_signal_wraith_01";
    fn project(state:ActorAiState,mode:SignalProjectionMode)->SignalPositionProjection {
        project_signal_positions(ID,p(10.,0.,8.),state,7,mode,SignalPerceptionConfig::default(),|_|true).unwrap().unwrap()
    }
    #[test]
    fn legacy_disabled_projection_has_no_effect_or_geometry_side_effects(){
        assert_eq!(project_signal_positions("",Vec3{x_m:f32::NAN,y_m:0.,z_m:0.},ActorAiState::Idle,0,SignalProjectionMode::Disabled,
            SignalPerceptionConfig{echo_distance_m:f32::NAN},|_|panic!("disabled projection consulted geometry")).unwrap(),None);
    }
    #[test]
    fn unmapped_actor_has_two_stable_bounded_alternatives_for_one_entity(){
        let actual=p(10.,0.,8.);let expected=project(ActorAiState::Cooldown,SignalProjectionMode::Unmapped);
        assert!(!expected.precise);assert_eq!(expected.positions_m.len(),2);assert_eq!(expected.uncertainty_radius_m,0.65);
        assert_eq!(expected.positions_m.iter().filter(|point|**point==actual).count(),1);
        let alternative=expected.positions_m.iter().find(|point|**point!=actual).unwrap();
        assert!(((alternative.x_m-actual.x_m).hypot(alternative.z_m-actual.z_m)-0.65).abs()<0.0001);assert_eq!(alternative.y_m,actual.y_m);
        for _ in 0..100 {assert_eq!(project(ActorAiState::Cooldown,SignalProjectionMode::Unmapped),expected);}
        assert_eq!(actual,p(10.,0.,8.));
    }
    #[test]
    fn acoustic_mapping_and_every_threat_or_stagger_phase_reveal_exact_position(){
        for state in [ActorAiState::Idle,ActorAiState::Investigate,ActorAiState::Chasing,ActorAiState::Cooldown]{
            assert_eq!(project(state,SignalProjectionMode::AcousticMapping),precise(p(10.,0.,8.)));
            assert!(!project(state,SignalProjectionMode::Unmapped).precise);
        }
        for state in [ActorAiState::Windup,ActorAiState::Active,ActorAiState::Stagger,ActorAiState::Dead]{
            assert_eq!(project(state,SignalProjectionMode::Unmapped),precise(p(10.,0.,8.)));
        }
    }
    #[test]
    fn geometry_selects_only_supported_alternatives_and_falls_back_without_fabrication(){
        let actual=p(10.,2.,8.);let config=SignalPerceptionConfig::default();
        let view=project_signal_positions(ID,actual,ActorAiState::Idle,7,SignalProjectionMode::Unmapped,config,|point|point.y_m==2.&&point.x_m>=10.&&point.x_m<=11.&&point.z_m==8.).unwrap().unwrap();
        assert!(!view.precise);assert!(view.positions_m.iter().all(|point|point.y_m==2.&&point.x_m>=10.&&point.x_m<=11.&&point.z_m==8.));
        assert_eq!(project_signal_positions(ID,actual,ActorAiState::Idle,7,SignalProjectionMode::Unmapped,config,|point|point==actual).unwrap(),Some(precise(actual)));
        assert_eq!(project_signal_positions(ID,actual,ActorAiState::Idle,7,SignalProjectionMode::AcousticMapping,config,|_|false).unwrap_err(),"E_SIGNAL_PROJECTION_GEOMETRY");
    }
    #[test]
    fn malformed_identity_position_or_config_reject_before_geometry(){
        for actor_id in ["", " "] {assert!(project_signal_positions(actor_id,p(10.,0.,8.),ActorAiState::Idle,0,SignalProjectionMode::Unmapped,SignalPerceptionConfig::default(),|_|panic!()).is_err());}
        assert!(project_signal_positions(&"x".repeat(257),p(10.,0.,8.),ActorAiState::Idle,0,SignalProjectionMode::Unmapped,SignalPerceptionConfig::default(),|_|panic!()).is_err());
        for distance in [0.,-0.1,2.01,f32::NAN,f32::INFINITY]{assert!(project_signal_positions(ID,p(10.,0.,8.),ActorAiState::Idle,0,SignalProjectionMode::Unmapped,SignalPerceptionConfig{echo_distance_m:distance},|_|panic!()).is_err());}
        for point in [Vec3{x_m:f32::NAN,..p(10.,0.,8.)},Vec3{y_m:f32::INFINITY,..p(10.,0.,8.)},Vec3{z_m:f32::NEG_INFINITY,..p(10.,0.,8.)}]{assert!(project_signal_positions(ID,point,ActorAiState::Idle,0,SignalProjectionMode::Unmapped,SignalPerceptionConfig::default(),|_|panic!()).is_err());}
    }
    #[test]
    fn projected_wire_contains_no_health_ai_counter_or_true_alternative_marker(){
        let value=serde_json::to_value(project(ActorAiState::Idle,SignalProjectionMode::Unmapped)).unwrap();
        assert_eq!(value.as_object().unwrap().keys().map(String::as_str).collect::<Vec<_>>(),["positionsM","precise","uncertaintyRadiusM"]);
        assert_eq!(value["positionsM"].as_array().unwrap().len(),2);
        assert!(!value.to_string().contains("Hp"));assert!(!value.to_string().contains("actual"));
        assert!(serde_json::from_value::<SignalPositionProjection>(serde_json::json!({"positionsM":[],"precise":true,"uncertaintyRadiusM":0,"actualIndex":0})).is_err());
    }
}
