use campaign_authority::{Authority as CampaignAuthority, ProgressEvent};
use combat_spatial::{
    AttackInput, FootTransform, SpatialAuthority, SpatialWorld, TargetCapsule, Vec3M,
};
use interaction_authority::{
    Action as InteractionAction, Authority as InteractionAuthority,
    ChoiceState as InteractionChoice, ColliderStatus, DoorSpec, LineOfSight, Prerequisite,
    Transform as InteractionTransform, WorldFlags,
};
use navgraph_ai::{Graph, Planner, WorldState as NavWorldState};
use serde_json::{json, Value};
use std::collections::HashSet;
use std::hint::black_box;
use std::time::Instant;
use wuxian_horror_ch1::continuous_kcc::{KccState, MotionInput, StaticKccWorld};
use wuxian_horror_ch1::fixed_step::{FixedStepScheduler, MAX_TICKS_PER_FRAME};
use wuxian_horror_ch1::world_v3::{Transform as WorldTransform, Vec3 as WorldVec3, WorldStateV3};

const EVIDENCE: &str = "synthetic_candidate_tp17a1";
const WORLD_EPOCH: u64 = 7;
const SOAK_TICKS: u64 = 36_000;

fn fnv1a(value: &Value) -> String {
    let bytes = serde_json::to_vec(value).expect("canonical JSON");
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in bytes {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("fnv1a64-{hash:016x}")
}

fn percentiles(samples_ns: &[u128]) -> Value {
    if samples_ns.is_empty() {
        return json!({"count":0,"p50Ns":null,"p95Ns":null,"p99Ns":null,"maxNs":null});
    }
    let mut sorted = samples_ns.to_vec();
    sorted.sort_unstable();
    let at = |fraction: f64| -> u128 {
        let index = ((sorted.len() as f64 - 1.0) * fraction).round() as usize;
        sorted[index]
    };
    json!({
        "count":sorted.len(),
        "p50Ns":at(0.50),
        "p95Ns":at(0.95),
        "p99Ns":at(0.99),
        "maxNs":*sorted.last().unwrap()
    })
}

fn world() -> WorldStateV3 {
    WorldStateV3::new(
        "grey_hive",
        WORLD_EPOCH,
        0,
        0,
        WorldTransform::new(WorldVec3::zero(), 0.0).expect("finite transform"),
        WorldVec3::zero(),
        Vec::new(),
        Vec::new(),
    )
    .expect("world contract")
}

fn progress_event(
    id: &str,
    tick: u64,
    kind: &str,
    prerequisite: Option<&str>,
    choice: Option<&str>,
) -> ProgressEvent {
    ProgressEvent {
        event_id: id.to_string(),
        epoch: WORLD_EPOCH,
        tick,
        actor_id: "player".to_string(),
        kind: kind.to_string(),
        prerequisite: prerequisite.map(str::to_string),
        objective: None,
        choice: choice.map(str::to_string),
        npc_available: Some(true),
        numeric_value: None,
    }
}

fn run_campaign(choice: Option<&str>) -> (CampaignAuthority, Vec<Value>) {
    let mut authority = CampaignAuthority::new(WORLD_EPOCH);
    let mut records = Vec::new();
    let mut apply =
        |event: ProgressEvent| records.push(serde_json::to_value(authority.apply(event)).unwrap());
    apply(progress_event("airlock", 0, "complete_airlock", None, None));
    if let Some(choice) = choice {
        apply(progress_event(
            "baizhi-dialog",
            1,
            "first_conversation",
            None,
            None,
        ));
        apply(progress_event(
            "baizhi-choice",
            2,
            "choose_baizhi",
            None,
            Some(choice),
        ));
    } else {
        apply(progress_event(
            "baizhi-dialog",
            1,
            "first_conversation",
            None,
            None,
        ));
    }
    apply(progress_event("power", 3, "restore_power", None, None));
    apply(progress_event(
        "lockdown",
        4,
        "complete_lockdown",
        Some("hive_power"),
        None,
    ));
    apply(progress_event(
        "gate-b",
        5,
        "reach_gate_b",
        Some("hive_lockdown_completed"),
        None,
    ));
    apply(progress_event("deep", 6, "enter_deep", None, None));
    apply(progress_event("sentinel", 7, "defeat_sentinel", None, None));
    apply(progress_event("beacon", 8, "activate_beacon", None, None));
    apply(progress_event("exit", 9, "activate_exit", None, None));
    apply(progress_event("extract", 10, "extract", None, None));
    apply(progress_event("hub", 11, "complete_result", None, None));
    (authority, records)
}

fn interaction_specs() -> Vec<DoorSpec> {
    vec![DoorSpec {
        door_id: "gh_gate_a".to_string(),
        prerequisite: Prerequisite::HivePower,
        dynamic_connector: false,
        enabled: true,
    }]
}

fn interaction_event(tick: u64, id: &str) -> interaction_authority::InteractionEvent {
    interaction_authority::InteractionEvent {
        event_id: id.to_string(),
        world_epoch: WORLD_EPOCH,
        tick,
        actor_id: "player".to_string(),
        actor_transform: InteractionTransform {
            position_m: [1.0, 0.0, 0.0],
            yaw_rad: 0.0,
        },
        target_id: "gh_gate_a".to_string(),
        target_transform: InteractionTransform {
            position_m: [0.0, 0.0, 0.0],
            yaw_rad: 0.0,
        },
        interaction_radius_m: 3.0,
        vertical_half_height_m: 1.0,
        target_exists: true,
        target_enabled: true,
        los: LineOfSight::Clear,
        collider: ColliderStatus::Clear,
        action: InteractionAction::OpenDoor,
    }
}

fn component_flow(hz: u32, choice: Option<&str>) -> Value {
    let mut scheduler =
        FixedStepScheduler::with_frequency(hz, 0).expect("formal scheduler frequency");
    let mut formal_world = world();
    let kcc_world = StaticKccWorld::room_10m();
    let mut kcc_state = KccState::new(
        WorldTransform::new(WorldVec3::zero(), 0.0).unwrap(),
        WorldVec3::zero(),
    );
    let mut interaction = InteractionAuthority::new(
        WORLD_EPOCH,
        0,
        WorldFlags {
            hive_power: true,
            hive_lockdown_completed: true,
            hive_choice: match choice {
                Some("taken") => InteractionChoice::Taken,
                Some("left") => InteractionChoice::Left,
                _ => InteractionChoice::Unresolved,
            },
            npc_available: choice.is_some(),
        },
        interaction_specs(),
    );
    let interaction_record = interaction.process_event(interaction_event(1, "flow-door-open"));
    let mut planner = Planner::new(Graph::candidate());
    let nav_path = planner
        .request(
            WORLD_EPOCH,
            2,
            "GH-A01",
            "GH-B05",
            &NavWorldState::lockdown(),
            &HashSet::new(),
        )
        .expect("candidate nav path");
    let mut combat_world = SpatialWorld::new();
    combat_world
        .add_target(TargetCapsule {
            id: "sentinel".to_string(),
            foot_position_m: Vec3M {
                x: 1.0,
                y: 0.0,
                z: 2.0,
            },
            radius_m: combat_spatial::CAPSULE_RADIUS_M,
            half_segment_m: combat_spatial::HALF_SEGMENT_M,
        })
        .expect("candidate target");
    let mut combat = SpatialAuthority::new(WORLD_EPOCH, 0);
    let combat_result = combat
        .query(
            &combat_world,
            &AttackInput {
                world_epoch: WORLD_EPOCH,
                world_tick: 3,
                attack_event_id: 1,
                attacker: FootTransform {
                    position_m: Vec3M {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    },
                    yaw_rad: 0.0,
                },
                range_m: 3.0,
                half_angle_rad: std::f32::consts::PI,
                vertical_tolerance_m: 2.0,
            },
        )
        .expect("candidate combat query");
    let (campaign, campaign_records) = run_campaign(choice);
    let mut kcc_trace = Vec::new();
    for tick in 0..12_u64 {
        let now = (tick + 1) * scheduler.config().fixed_dt_nanos();
        let plan = scheduler.advance(now).expect("formal scheduler tick");
        formal_world
            .advance_clock(tick, tick)
            .expect("formal world clock");
        let step = kcc_world
            .move_foot(
                &kcc_state,
                MotionInput::stationary(1.0 / hz as f32).unwrap(),
            )
            .expect("formal KCC step");
        kcc_state = step.state;
        kcc_trace.push(json!({"tick":tick,"planned":plan.tick_count(),"grounded":step.grounded,"penetration":step.penetration}));
    }
    let world_hash = fnv1a(&serde_json::to_value(formal_world.view(12)).unwrap());
    let kcc_position = kcc_state.foot_transform.position_m();
    let kcc_hash = fnv1a(
        &json!({"x":kcc_position.x_m(),"y":kcc_position.y_m(),"z":kcc_position.z_m(),"yaw":kcc_state.foot_transform.yaw_rad()}),
    );
    let interaction_hash = fnv1a(&json!({
        "epoch":interaction.world_epoch,
        "tick":interaction.tick,
        "door":interaction.door("gh_gate_a"),
        "sideEffects":interaction.side_effect_count,
        "recordOutcome":interaction_record.outcome,
    }));
    let nav_hash =
        fnv1a(&json!({"nodes":nav_path.node_ids,"edges":nav_path.edge_ids,"cost":nav_path.cost_m}));
    let combat_hash =
        fnv1a(&json!({"targets":combat_result.target_ids,"audits":combat_result.audits}));
    json!({
        "hz":hz,
        "choice":choice.unwrap_or("unresolved"),
        "events":campaign_records,
        "finalHashes":{
            "world":world_hash,
            "campaign":campaign.state_hash(),
            "nav":nav_hash,
            "combat":combat_hash,
            "interaction":interaction_hash,
            "kcc":kcc_hash,
        },
        "mainline":{
            "campaignMode":campaign.mode,
            "campaignObjective":campaign.objective,
            "gateA":campaign.gate_a_open,
            "gateB":campaign.gate_b_open,
            "sentinel":campaign.sentinel_defeated,
            "beacon":campaign.beacon_active,
            "exit":campaign.exit_active,
            "rewardCount":campaign.reward_ledger.len(),
            "mapNull":campaign.result.as_ref().is_some_and(|result| result.map.is_none()),
            "navEdgeCount":nav_path.edge_ids.len(),
            "combatTargets":combat_result.target_ids,
            "kccTrace":kcc_trace,
        }
    })
}

fn scheduler_pause_resume() -> Value {
    let mut scheduler = FixedStepScheduler::with_frequency(60, 0).unwrap();
    let dt = scheduler.config().fixed_dt_nanos();
    let before = scheduler.advance(dt * 3).unwrap();
    let before_ticks = scheduler.total_ticks();
    scheduler.pause(dt * 3).unwrap();
    let paused = scheduler.advance(dt * 3 + 5_000_000_000).unwrap();
    let during_pause = scheduler.total_ticks() - before_ticks;
    scheduler.resume(dt * 3 + 5_000_000_000).unwrap();
    let after = scheduler.advance(dt * 3 + 5_000_000_000 + dt).unwrap();
    json!({"beforeTicks":before.tick_count(),"pausedTicks":paused.tick_count(),"duringPause":during_pause,"afterResumeTicks":after.tick_count(),"pass":before_ticks==3 && paused.tick_count()==0 && during_pause==0 && after.tick_count()==1})
}

fn catch_up_boundary() -> Value {
    let mut scheduler = FixedStepScheduler::with_frequency(60, 0).unwrap();
    let long = scheduler.advance(250_000_000).unwrap();
    json!({"longFrameNs":250_000_000,"executedTicks":long.tick_count(),"maxCatchUp":MAX_TICKS_PER_FRAME,"droppedNs":long.dropped_time_nanos(),"pass":long.tick_count() <= MAX_TICKS_PER_FRAME as usize && long.dropped_time_nanos() > 0})
}

fn soak() -> Value {
    let started = Instant::now();
    let mut scheduler = FixedStepScheduler::with_frequency(60, 0).unwrap();
    let dt = scheduler.config().fixed_dt_nanos();
    let mut formal_world = world();
    let kcc_world = StaticKccWorld::room_10m();
    let mut kcc_state = KccState::new(
        WorldTransform::new(WorldVec3::zero(), 0.0).unwrap(),
        WorldVec3::zero(),
    );
    let mut spatial_world = SpatialWorld::new();
    spatial_world
        .add_target(TargetCapsule {
            id: "soak-target".to_string(),
            foot_position_m: Vec3M {
                x: 1.0,
                y: 0.0,
                z: 2.0,
            },
            radius_m: combat_spatial::CAPSULE_RADIUS_M,
            half_segment_m: combat_spatial::HALF_SEGMENT_M,
        })
        .unwrap();
    let mut combat = SpatialAuthority::new(WORLD_EPOCH, 0);
    let mut planner = Planner::new(Graph::candidate());
    let mut samples = Vec::with_capacity(SOAK_TICKS as usize);
    let mut nav_calls = 0_u64;
    let mut combat_queries = 0_u64;
    let mut kcc_steps = 0_u64;
    for tick in 0..SOAK_TICKS {
        let tick_started = Instant::now();
        let now = (tick + 1) * dt;
        let plan = scheduler.advance(now).unwrap();
        formal_world.advance_clock(tick, tick).unwrap();
        let step = kcc_world
            .move_foot(&kcc_state, MotionInput::stationary(1.0 / 60.0).unwrap())
            .unwrap();
        kcc_state = step.state;
        kcc_steps += 1;
        let _ = combat
            .query(
                &spatial_world,
                &AttackInput {
                    world_epoch: WORLD_EPOCH,
                    world_tick: tick,
                    attack_event_id: tick + 1,
                    attacker: FootTransform {
                        position_m: Vec3M {
                            x: 0.0,
                            y: 0.0,
                            z: 0.0,
                        },
                        yaw_rad: 0.0,
                    },
                    range_m: 3.0,
                    half_angle_rad: std::f32::consts::PI,
                    vertical_tolerance_m: 2.0,
                },
            )
            .unwrap();
        combat_queries += 1;
        if tick % 6_000 == 0 {
            let _ = planner
                .request(
                    WORLD_EPOCH,
                    tick,
                    "GH-A01",
                    "GH-B05",
                    &NavWorldState::lockdown(),
                    &HashSet::new(),
                )
                .unwrap();
            nav_calls += 1;
        }
        black_box(plan);
        samples.push(tick_started.elapsed().as_nanos());
    }
    let wall = started.elapsed();
    json!({
        "ticksRequested":SOAK_TICKS,
        "ticksExecuted":scheduler.total_ticks(),
        "schedulerDroppedNs":scheduler.total_dropped_time_nanos(),
        "worldTick":formal_world.server_tick(),
        "kccSteps":kcc_steps,
        "combatQueries":combat_queries,
        "navCalls":nav_calls,
        "wallTimeMs":wall.as_secs_f64()*1000.0,
        "tickLatency":percentiles(&samples),
        "memory":{"rss":"unknown","allocator":"unknown","evidenceLevel":"unavailable_public_api"},
        "pass":scheduler.total_ticks()==SOAK_TICKS && formal_world.server_tick()+1==SOAK_TICKS,
    })
}

fn lifecycle_soak() -> Value {
    let started = Instant::now();
    let mut observed_campaign_baselines = Vec::new();
    for cycle in 0..30_u64 {
        let choice = match cycle % 3 {
            0 => Some("taken"),
            1 => Some("left"),
            _ => None,
        };
        let (campaign, _) = run_campaign(choice);
        observed_campaign_baselines.push(json!({"cycle":cycle,"processedEventCount":campaign.processed_event_ids.len(),"rewardCount":campaign.reward_ledger.len(),"activeWorld":campaign.active_world}));
        let _ = component_flow(60, choice);
    }
    json!({
        "enterCount":30,
        "exitCount":30,
        "elapsedMs":started.elapsed().as_secs_f64()*1000.0,
        "observedCampaignBaseline":{"newAuthorityProcessedEventCount":0,"newAuthorityRewardCount":0},
        "ownerCounters":{"world":"unknown","queue":"unknown","entity":"unknown","interactionProcessedEvents":"unknown_private","combatProcessedEvents":"unknown_private","navCache":"unknown_private"},
        "pass":false,
        "blockedReason":"E_NO_PUBLIC_OWNER_COUNTERS",
        "evidenceLevel":"blocked_public_api_boundary",
        "sampleCycles":observed_campaign_baselines.iter().take(3).collect::<Vec<_>>()
    })
}

fn event_once_and_ordering() -> Value {
    let mut campaign = CampaignAuthority::new(WORLD_EPOCH);
    let campaign_records = campaign.apply_batch(vec![
        progress_event("event-b", 0, "complete_airlock", None, None),
        progress_event("event-a", 0, "complete_airlock", None, None),
    ]);
    let campaign_order: Vec<String> = campaign_records
        .iter()
        .map(|record| record.event_id.clone())
        .collect();
    let campaign_duplicate =
        campaign.apply(progress_event("event-a", 0, "complete_airlock", None, None));
    let mut campaign_old_epoch_event =
        progress_event("old-epoch", 0, "complete_airlock", None, None);
    campaign_old_epoch_event.epoch = WORLD_EPOCH - 1;
    let campaign_old_epoch = campaign.apply(campaign_old_epoch_event);

    let mut planner = Planner::new(Graph::candidate());
    let _ = planner
        .request(
            WORLD_EPOCH,
            1,
            "GH-A01",
            "GH-B05",
            &NavWorldState::lockdown(),
            &HashSet::new(),
        )
        .unwrap();
    let nav_stale = planner.request(
        WORLD_EPOCH,
        0,
        "GH-A01",
        "GH-B05",
        &NavWorldState::lockdown(),
        &HashSet::new(),
    );

    let mut interaction = InteractionAuthority::new(
        WORLD_EPOCH,
        0,
        WorldFlags {
            hive_power: true,
            hive_lockdown_completed: true,
            hive_choice: InteractionChoice::Unresolved,
            npc_available: false,
        },
        interaction_specs(),
    );
    let interaction_input = interaction_event(0, "interaction-once");
    let _ = interaction.process_event(interaction_input.clone());
    let interaction_duplicate = interaction.process_event(interaction_input);

    let mut combat_world = SpatialWorld::new();
    combat_world
        .add_target(TargetCapsule {
            id: "once-target".to_string(),
            foot_position_m: Vec3M {
                x: 1.0,
                y: 0.0,
                z: 2.0,
            },
            radius_m: combat_spatial::CAPSULE_RADIUS_M,
            half_segment_m: combat_spatial::HALF_SEGMENT_M,
        })
        .unwrap();
    let attack = AttackInput {
        world_epoch: WORLD_EPOCH,
        world_tick: 0,
        attack_event_id: 77,
        attacker: FootTransform {
            position_m: Vec3M {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            yaw_rad: 0.0,
        },
        range_m: 3.0,
        half_angle_rad: std::f32::consts::PI,
        vertical_tolerance_m: 2.0,
    };
    let mut combat = SpatialAuthority::new(WORLD_EPOCH, 0);
    let _ = combat.query(&combat_world, &attack).unwrap();
    let combat_duplicate = combat
        .query(&combat_world, &attack)
        .err()
        .map(|error| error.to_string());
    let pass = campaign_order == vec!["event-a".to_string(), "event-b".to_string()]
        && campaign_duplicate.error_code.as_deref() == Some("E_DUPLICATE_EVENT")
        && campaign_old_epoch.error_code.as_deref() == Some("E_OLD_EPOCH")
        && nav_stale == Err(navgraph_ai::PlanError::StaleEpochOrTick)
        && interaction_duplicate.error_code.as_deref() == Some("E_DUPLICATE_EVENT")
        && combat_duplicate.as_deref() == Some("duplicate_attack_event:77");
    json!({
        "input":{"sameTickEventIds":["event-b","event-a"],"campaignDuplicate":"event-a","navStaleTick":0,"combatDuplicateId":77},
        "actual":{"campaignOrder":campaign_order,"campaignDuplicate":campaign_duplicate.error_code,"campaignOldEpoch":campaign_old_epoch.error_code,"navStale":format!("{:?}",nav_stale.err()),"interactionDuplicate":interaction_duplicate.error_code,"combatDuplicate":combat_duplicate},
        "expected":{"campaignOrder":["event-a","event-b"],"duplicate":"rejected_once","oldEpoch":"rejected","navStale":"rejected","combatDuplicate":"rejected_once"},
        "pass":pass,
        "evidenceLevel":EVIDENCE
    })
}

pub fn run_probe() -> Value {
    let flow30 = component_flow(30, Some("taken"));
    let flow60 = component_flow(60, Some("taken"));
    let hash_equal = flow30["finalHashes"] == flow60["finalHashes"];
    let choice_flows = [
        component_flow(60, Some("taken")),
        component_flow(60, Some("left")),
        component_flow(60, None),
    ];
    let choice_pass = choice_flows.iter().all(|flow| {
        flow["mainline"]["gateB"] == true
            && flow["mainline"]["sentinel"] == true
            && flow["mainline"]["beacon"] == true
            && flow["mainline"]["exit"] == true
            && flow["mainline"]["mapNull"] == true
            && flow["mainline"]["rewardCount"] == 1
    });
    let save_report = save_resume_resolution::run_all_scenarios();
    let save_pass = save_report.machine_pass;
    let pause = scheduler_pause_resume();
    let catch_up = catch_up_boundary();
    let soak_report = soak();
    let lifecycle = lifecycle_soak();
    let exactly_once = event_once_and_ordering();
    let queue = json!({
        "status":"blocked",
        "errorCode":"E_NO_PUBLIC_QUEUE_API",
        "inputEventQueue":"not exposed by completed public APIs",
        "fixedStepCatchUp":{"maxTicksPerFrame":MAX_TICKS_PER_FRAME,"overflowDropObserved":catch_up["droppedNs"]},
        "highWater":"unknown",
        "requiredMinimumApi":"bounded enqueue/dequeue with capacity, high-water, and overflow error"
    });
    let scenarios = vec![
        json!({"id":"mainline_no_ui_three_choice_flow","dependencies":["campaign-authority::Authority","interaction-authority::Authority","navgraph-ai::Planner","combat-spatial::SpatialAuthority","server-rs::fixed_step::FixedStepScheduler","server-rs::continuous_kcc::StaticKccWorld"],"input":{"choices":["taken","left","unresolved"]},"events":["Power","Gate A","lockdown","Gate B","navigation","melee","Sentinel","Beacon","Exit","result_then_hub"],"expected":{"allChoicesPass":true},"actual":{"allChoicesPass":choice_pass,"flows":choice_flows},"pass":choice_pass,"evidenceLevel":EVIDENCE}),
        json!({"id":"thirty_sixty_final_component_hashes","input":{"sameEventStream":true,"frequencies":[30,60]},"expected":{"sameHashes":true},"actual":{"sameHashes":hash_equal,"hz30":flow30["finalHashes"],"hz60":flow60["finalHashes"]},"pass":hash_equal,"evidenceLevel":EVIDENCE}),
        json!({"id":"pause_resume","input":{"formalApi":"server-rs::fixed_step::FixedStepScheduler"},"expected":{"pausedTicks":0,"resumeTicks":1},"actual":pause,"pass":pause["pass"],"evidenceLevel":"formal_api_plus_".to_string()+EVIDENCE}),
        json!({"id":"long_frame_catch_up_cap","input":{"longFrameMs":250},"expected":{"maxTicks":MAX_TICKS_PER_FRAME,"droppedTime":true},"actual":catch_up,"pass":catch_up["pass"],"evidenceLevel":"formal_api_plus_".to_string()+EVIDENCE}),
        json!({"id":"bounded_36000_tick_soak","input":{"ticks":SOAK_TICKS,"durationEquivalentMinutes":10},"expected":{"ticks":SOAK_TICKS,"bounded":true},"actual":soak_report,"pass":soak_report["pass"],"evidenceLevel":EVIDENCE}),
        json!({"id":"event_exactly_once_stable_order_stale_epoch","input":exactly_once["input"],"events":exactly_once["input"],"expected":exactly_once["expected"],"actual":exactly_once["actual"],"pass":exactly_once["pass"],"evidenceLevel":EVIDENCE}),
        json!({"id":"thirty_world_enter_exit_lifecycle","input":{"cycles":30},"expected":{"ownersQueuesEntitiesAndLedgersReturnToBaseline":true},"actual":lifecycle,"pass":false,"errorCode":"E_NO_PUBLIC_OWNER_COUNTERS","evidenceLevel":"blocked_public_api_boundary"}),
        json!({"id":"input_event_queue_capacity_overflow","input":{"requiredCapacity":256},"expected":{"boundedQueueAndOverflowError":true},"actual":queue,"pass":false,"errorCode":"E_NO_PUBLIC_QUEUE_API","evidenceLevel":"blocked_public_api_boundary"}),
        json!({"id":"save_resume_completed_evidence_reuse","input":{"api":"save-resume-resolution::run_all_scenarios"},"expected":{"completedAndReloadScenariosPass":true},"actual":{"machinePass":save_report.machine_pass,"candidateOnly":save_report.candidate_only,"scenarioCount":save_report.scenarios.len()},"pass":save_pass,"evidenceLevel":"completed_candidate_api_reused"}),
    ];
    let blocked = true;
    json!({
        "schema":"runtime_integration_soak_tp17a1_v1",
        "status":"blocked_public_api_boundary",
        "pass":false,
        "machinePass":false,
        "candidateOnly":true,
        "formalSceneReady":false,
        "reuseContract":{
            "formalFixedStep":"wuxian_horror_ch1::fixed_step::FixedStepScheduler",
            "formalKcc":"wuxian_horror_ch1::continuous_kcc::{StaticKccWorld,KccState,MotionInput}",
            "formalWorld":"wuxian_horror_ch1::world_v3::WorldStateV3",
            "campaign":"campaign-authority::Authority",
            "interaction":"interaction-authority::Authority",
            "nav":"navgraph-ai::Planner",
            "combat":"combat-spatial::SpatialAuthority",
            "saveResume":"save-resume-resolution::run_all_scenarios",
            "noShadowCore":true
        },
        "scenarios":scenarios,
        "queueBoundary":queue,
        "lifecycle":lifecycle,
        "soak":soak_report,
        "performanceEnvironment":{
            "os":std::env::consts::OS,
            "arch":std::env::consts::ARCH,
            "profile":"debug (cargo default; not a release budget)",
            "wallClock":"std::time::Instant",
            "rss":"unknown",
            "allocator":"unknown",
            "memoryEvidenceLevel":"unavailable_public_api"
        },
        "blockedInterfaces":[
            {"errorCode":"E_NO_PUBLIC_QUEUE_API","missing":"bounded event/input queue capacity, high-water and overflow result","minimumFollowup":"BoundedQueue::{try_push,pop,len,capacity,high_water,overflow_count}"},
            {"errorCode":"E_NO_PUBLIC_OWNER_COUNTERS","missing":"world/entity/queue owner lifecycle counters after drop","minimumFollowup":"RuntimeOwnerSnapshot::{world_count,entity_count,queue_count,reset_epoch}"},
            {"errorCode":"E_NO_PUBLIC_EVENT_LEDGER_COUNTER","missing":"interaction and combat consumed-event ledger length/reset; nav cache owner lifecycle","minimumFollowup":"processed_event_count/reset_epoch/cache_snapshot accessors on public authorities"},
            {"errorCode":"E_FORMAL_SCENE_RUNTIME_NOT_BOUND","missing":"candidate nav/interaction/combat cannot bind formal scene, Babylon, Tauri/WebView or production owner","minimumFollowup":"Explicit runtime adapter trait with owner and epoch lifecycle"},
            {"errorCode":"E_MEMORY_METRICS_UNAVAILABLE","missing":"RSS and allocator counters","minimumFollowup":"platform memory sampler or allocator instrumentation"}
        ],
        "blocked":blocked,
        "formalGaps":["Tauri/WebView/Babylon and real scene/load absent","formal published hardware and asset load absent","power-loss restore and UI absent","TP-17 and related Gates remain blocked; Architecture no_go"]
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn public_composed_flow_is_deterministic_at_both_rates() {
        let a = component_flow(30, Some("taken"));
        let b = component_flow(60, Some("taken"));
        assert_eq!(a["finalHashes"], b["finalHashes"]);
        assert_eq!(a["mainline"]["mapNull"], true);
    }

    #[test]
    fn formal_scheduler_pause_and_catch_up_are_bounded() {
        assert_eq!(scheduler_pause_resume()["pass"], true);
        assert_eq!(catch_up_boundary()["pass"], true);
    }

    #[test]
    fn soak_has_exact_tick_budget_but_public_lifecycle_is_blocked() {
        let result = soak();
        assert_eq!(result["ticksRequested"], SOAK_TICKS);
        assert_eq!(result["ticksExecuted"], SOAK_TICKS);
        assert_eq!(result["memory"]["rss"], "unknown");
        assert_eq!(
            lifecycle_soak()["blockedReason"],
            "E_NO_PUBLIC_OWNER_COUNTERS"
        );
    }
}
