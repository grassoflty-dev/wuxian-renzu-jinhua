//! v3 receipt boundary for authored scene route commands.

use crate::{formal_runtime::FormalRuntime, world_v3::CommandReceipt};

/// Preserve the command's atomic interaction snapshot, including rejections.
/// Missing source epochs remain a legacy path; strict native CW rejects them.
pub fn interaction(
    runtime: &FormalRuntime,
    interaction_id: &str,
    request_id: &str,
    world_epoch: Option<u64>,
) -> Result<crate::formal_runtime::FormalInteractionResponse, String> {
    let mut response = match world_epoch {
        Some(epoch) => runtime.activate_scene_interaction(interaction_id, request_id, epoch)?,
        None => runtime.interact(interaction_id, request_id)?,
    };
    response.receipt = CommandReceipt::outcome(
        request_id, response.applied, response.already_applied,
        response.error_code.clone(), response.view.clone(),
    );
    Ok(response)
}

pub fn transition(
    runtime: &FormalRuntime,
    transition_id: &str,
    request_id: &str,
    world_epoch: u64,
) -> Result<CommandReceipt, String> {
    receipt_for(
        request_id,
        runtime,
        runtime.transition_scene(transition_id, request_id, world_epoch),
    )
}

pub fn checkpoint(
    runtime: &FormalRuntime,
    checkpoint_id: &str,
    request_id: &str,
    world_epoch: u64,
) -> Result<CommandReceipt, String> {
    receipt_for(
        request_id,
        runtime,
        runtime.activate_scene_checkpoint(checkpoint_id, request_id, world_epoch),
    )
}

pub fn trigger(
    runtime: &FormalRuntime,
    trigger_id: &str,
    request_id: &str,
    world_epoch: u64,
) -> Result<CommandReceipt, String> {
    receipt_for(
        request_id,
        runtime,
        runtime.activate_scene_trigger(trigger_id, request_id, world_epoch),
    )
}

pub fn world_gate(
    runtime: &FormalRuntime,
    gate_id: &str,
    request_id: &str,
    world_epoch: u64,
) -> Result<CommandReceipt, String> {
    receipt_for(
        request_id,
        runtime,
        runtime.use_world_gate(gate_id, request_id, world_epoch),
    )
}

pub fn mission_terminal_status(
    runtime: &FormalRuntime,
    terminal_id: &str,
    request_id: &str,
    world_epoch: u64,
) -> Result<CommandReceipt, String> {
    receipt_for(
        request_id,
        runtime,
        runtime.mission_terminal_status(terminal_id, request_id, world_epoch),
    )
}

pub fn capability_terminal_status(
    runtime: &FormalRuntime,
    terminal_id: &str,
    request_id: &str,
    world_epoch: u64,
) -> Result<CommandReceipt, String> {
    receipt_for(
        request_id,
        runtime,
        runtime.capability_terminal_status(terminal_id, request_id, world_epoch),
    )
}

pub fn save_rest_terminal(
    runtime: &FormalRuntime,
    terminal_id: &str,
    request_id: &str,
    world_epoch: u64,
) -> Result<CommandReceipt, String> {
    receipt_for(
        request_id,
        runtime,
        runtime.save_rest_terminal(terminal_id, request_id, world_epoch),
    )
}

fn receipt_for(
    request_id: &str,
    runtime: &FormalRuntime,
    result: Result<crate::world_v3::WorldView, String>,
) -> Result<CommandReceipt, String> {
    match result {
        Ok(view) => Ok(CommandReceipt::from_view(request_id, view)),
        Err(error_code) => Ok(CommandReceipt::outcome(
            request_id,
            false,
            false,
            Some(error_code),
            runtime.snapshot()?,
        )),
    }
}

/// The mutation/rejection and snapshot are captured under the same runtime locks.
pub fn environment_control(
    runtime: &FormalRuntime,
    id: &str,
    request_id: &str,
    world_epoch: u64,
) -> Result<CommandReceipt, String> {
    let response = runtime.activate_environment_control(id, request_id, world_epoch)?;
    Ok(CommandReceipt::outcome(
        request_id,
        response.applied,
        response.already_applied,
        response.error_code,
        response.view,
    ))
}
