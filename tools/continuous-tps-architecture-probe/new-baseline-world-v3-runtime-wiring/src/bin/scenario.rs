use world_v3_runtime_wiring_candidate::*;

fn main() {
    let mut world = World::new();
    world.hive_lockdown = true;
    let _ = world.apply(WorldCommand::Move {
        seq: 1,
        delta: Vec3 {
            x_m: 1.0,
            y_m: 0.0,
            z_m: 0.5,
        },
    });
    world.step();
    let _ = world.apply(WorldCommand::ToggleDoor {
        seq: 2,
        door_id: "door.main".into(),
    });
    let _ = world.apply(WorldCommand::EnterTrigger {
        seq: 3,
        trigger_id: "trigger.gate_b".into(),
    });
    let save = world.save_v3();
    let restored = World::restore(save).expect("candidate save restore");
    println!("{{\"schemaVersion\":3,\"signature\":\"{}\",\"epoch\":{},\"tick\":{},\"ackSeq\":{},\"doorOpen\":{},\"triggerInside\":{},\"zoneId\":\"{}\",\"candidateOnly\":true}}", restored.signature(), restored.epoch, restored.tick, restored.ack, restored.door_open, restored.trigger_inside, restored.zone_id);
}
