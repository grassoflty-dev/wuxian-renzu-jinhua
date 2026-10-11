import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const root = path.resolve(__dirname, '..', '..', '..');
const resultPath = path.join(root, 'tools/continuous-tps-architecture-probe/results/tauri-input-runtime-tp06b1.json');
const result = JSON.parse(fs.readFileSync(resultPath, 'utf8'));
const failures = [];
const check = (name, condition, detail = '') => { if (!condition) failures.push({ name, detail }); };

check('schema', result.schema === 'continuous_tauri_input_runtime_tp06b1_v1');
check('status', result.status === 'blocked_isolated_transport_candidate');
check('machinePass', result.machinePass === true);
check('environment', result.environment?.tauri === '2.11.5' && result.environment?.platform === 'windows-webview2' && result.environment?.os === 'windows' && Boolean(result.environment?.windowsWebViewVersion));
check('fixedInputCommand', result.api?.fixedInputCommand === 'tp06b_input(sessionId,payload:String)');
check('formalParser', result.api?.formalParser === 'continuous_input::InputFrame::from_json' && result.api?.parserBypass === false);
check('ownerContract', result.owner?.oneInputReceiverPerSession && result.owner?.oneFixedStepSchedulerPerSession && result.owner?.oneWorldViewChannelPerGeneration && result.owner?.typedChannel === 'Channel<world_v3::WorldView>');
check('ownerBounds', result.owner?.bounded?.inputMaxBytes === 1024 && result.owner?.bounded?.maxEdgesPerFrame === 8 && result.owner?.bounded?.maxPendingFrames === 32 && result.owner?.bounded?.maxQueuedEdges === 256 && result.owner?.bounded?.schedulerMaxTicksPerFrame === 5);
check('lockAndReplacementBoundary', result.owner?.locks.includes('outside registry lock') && result.owner?.replacementBarrier.includes('stop/join completes'));

const expectedScenarios = [
  'normal_held_edge_short_flow',
  'duplicate_out_of_order_gap',
  'oversize_invalid_protocol',
  'held_expiry',
  'pause_edge_discard_and_resume_baseline',
  'hidden_edge_discard_and_resume_baseline',
  'epoch_replacement_barrier',
  'stop_and_stable_rejection'
];
const scenarios = new Map((result.scenarios || []).map(s => [s.name, s]));
check('scenarioCount', result.scenarios?.length === expectedScenarios.length);
check('scenarioNames', expectedScenarios.every(name => scenarios.has(name)));
check('allScenariosPass', expectedScenarios.every(name => scenarios.get(name)?.pass === true));
check('normalEdgeExactlyOnce', scenarios.get('normal_held_edge_short_flow')?.edgeConsumedCountFromFormalReceiver === 1 && scenarios.get('normal_held_edge_short_flow')?.edgeExactlyOnce === true);
check('duplicateGap', scenarios.get('duplicate_out_of_order_gap')?.duplicateOutcome?.errorCode === 'duplicate_or_old_sequence' && scenarios.get('duplicate_out_of_order_gap')?.gap9Outcome?.status === 'buffered' && scenarios.get('duplicate_out_of_order_gap')?.contiguousRecoveryAck === true);
check('invalidPaths', scenarios.get('oversize_invalid_protocol')?.oversizedError === 'payload_too_large' && scenarios.get('oversize_invalid_protocol')?.invalidProtocolOutcome?.errorCode === 'wrong_protocol');
check('heldExpiry', (scenarios.get('held_expiry')?.expiredHeldSamplesDelta || 0) > 0);
check('pauseBarrier', scenarios.get('pause_edge_discard_and_resume_baseline')?.staleEdgeDiscarded && scenarios.get('pause_edge_discard_and_resume_baseline')?.resumeSeqBaselineAck);
check('hiddenBarrier', scenarios.get('hidden_edge_discard_and_resume_baseline')?.staleEdgeDiscarded && scenarios.get('hidden_edge_discard_and_resume_baseline')?.resumeSeqBaselineAck);
check('epochBarrier', scenarios.get('epoch_replacement_barrier')?.newEpochAckReset && scenarios.get('epoch_replacement_barrier')?.newEpochEdgeExactlyOnce && scenarios.get('epoch_replacement_barrier')?.newGeneration > scenarios.get('epoch_replacement_barrier')?.oldGeneration);
check('stopBarrier', scenarios.get('stop_and_stable_rejection')?.stopClean && scenarios.get('stop_and_stable_rejection')?.inputRejectedAfterStop && scenarios.get('stop_and_stable_rejection')?.stopRejectedAfterStop);

const callbacks = result.callbacks || [];
check('callbackEvidence', callbacks.length > 0 && callbacks.every(record => record.protocol === 'continuous-ipc' && record.version === 1 && record.worldId === 'tp06b1-synthetic' && record.payloadBytes > 0));
for (const [name, channel] of Object.entries(result.channels || {})) {
  const records = channel.records || [];
  check(`channel:${name}:count`, channel.callbackCount === records.length);
  check(`channel:${name}:tickMonotonic`, records.every((record, index) => index === 0 || record.tick > records[index - 1].tick));
  check(`channel:${name}:ackMonotonic`, records.every((record, index) => index === 0 || record.ackSeq >= records[index - 1].ackSeq));
}
check('nativeLifecycleBoundary', result.nativeLifecycle?.syntheticDispatchUsed === false && result.nativeLifecycle?.nativeBlur === 'not_measured' && result.nativeLifecycle?.nativeVisibilityHidden === 'not_measured');
check('processAndCommandExit', result.commandExitCodes?.cargoTest === 0 && result.commandExitCodes?.cargoRun === 0 && result.stop?.processExitObservedByHarness === true);
check('formalGatesBlocked', result.formalGate?.tp06 === 'blocked_isolated_transport_only' && result.formalGate?.gateD === 'blocked/no_go' && result.formalGate?.architecture === 'blocked/no_go');

const report = {
  schema: 'continuous_tauri_input_runtime_validator_tp06b1_v1',
  allChecksPassed: failures.length === 0,
  checks: {
    schemaEnvironment: failures.every(f => !['schema', 'status', 'machinePass', 'environment'].includes(f.name)),
    apiOwner: failures.every(f => !f.name.includes('Command') && !f.name.includes('formalParser') && !f.name.includes('owner') && !f.name.includes('Bounds') && !f.name.includes('lock')),
    scenarios: failures.every(f => !f.name.includes('scenario') && !f.name.includes('Scenario') && !['normalEdgeExactlyOnce', 'duplicateGap', 'invalidPaths', 'heldExpiry', 'pauseBarrier', 'hiddenBarrier', 'epochBarrier', 'stopBarrier'].includes(f.name)),
    callbacks: failures.every(f => !f.name.includes('callback') && !f.name.includes('channel')),
    lifecycleAndExit: failures.every(f => !f.name.includes('native') && !f.name.includes('process') && !f.name.includes('formalGates'))
  },
  failures
};
console.log(JSON.stringify(report, null, 2));
if (failures.length) process.exitCode = 1;
