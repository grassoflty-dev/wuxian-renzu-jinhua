import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';

const source = readFileSync(new URL(
  '../../apps/godot_playable/tests/campaign/grey_hive_first_playable.gd',
  import.meta.url
), 'utf8').replace(/\r\n/g, '\n');
const runnerSource = readFileSync(new URL('./run-owned.ps1', import.meta.url), 'utf8').replace(/\r\n/g, '\n');

function literal(name) {
  const escaped = name.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  const re = new RegExp(
    '^const\\s+' + escaped + '\\s*=\\s*"""\\n([\\s\\S]*?)\\n"""', 'm'
  );
  const match = re.exec(source);
  assert.ok(match, 'complete multiline JSON literal: ' + name);
  return JSON.parse(match[1]);
}
const I = literal('GH_INTERFACE_JSON');
const R = literal('GH_ROUTE_JSON');
const runnerInterfaceMatch = runnerSource.match(/^\$GH_INTERFACE_JSON\s*=\s*@'\n([\s\S]*?)\n'@/m);
assert.ok(runnerInterfaceMatch, 'complete runner interface literal');
const runnerInterface = JSON.parse(runnerInterfaceMatch[1]);
assert.deepEqual(I, runnerInterface, 'runner and adapter share one exact interface');
const lines = source.split('\n');

function body(name) {
  const escaped = name.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  const re = new RegExp('^func\\s+' + escaped + '\\s*\\(');
  const begin = lines.findIndex(line => re.test(line));
  assert.ok(begin >= 0, 'function exists: ' + name);
  let end = begin + 1;
  while (end < lines.length && !/^func\s/.test(lines[end])) end++;
  return lines.slice(begin, end).join('\n');
}
function ordered(text, anchors) {
  let cursor = 0;
  for (const anchor of anchors) {
    const at = text.indexOf(anchor, cursor);
    assert.ok(at >= 0, 'positive scoped anchor: ' + anchor);
    cursor = at + anchor.length;
  }
}
function has(text, anchor) {
  assert.ok(text.includes(anchor), 'scoped anchor: ' + anchor);
}
function routeModel(trace) {
  let next = 0;
  for (const scene of trace) {
    if (!R.visits.includes(scene)) throw Error('out-of-scope');
    if (scene === R.visits[next]) next++;
  }
  return next === R.visits.length;
}
function onceModel(rows, phase) {
  const entries = rows.filter(r =>
    r.method === 'formal_new' || r.method === 'formal_continue');
  return entries.length === 1 &&
    entries[0].method ===
      (phase === 'new' ? 'formal_new' : 'formal_continue') &&
    entries[0].outcome === 'CONFIRMED';
}
function bindingModel(a, b) {
  return R.captureBinding.every(key =>
    JSON.stringify(a[key]) === JSON.stringify(b[key]));
}

test('SOURCE_CONTRACT_NOT_GDSCRIPT_OR_RUNTIME: fixed interface and route', () => {
  assert.equal(I.schema, 'wuxian.gh-first-playable.interface/v2');
  assert.equal(I.manifestSchema, runnerInterface.manifestSchema);
  assert.deepEqual(I.phaseConfigKeys, runnerInterface.phaseConfigKeys);
  assert.deepEqual(I.eventKeys, runnerInterface.eventKeys);
  assert.equal(I.eventTransport, 'phase-output-file');
  assert.equal(I.eventFileName, 'events.ndjson');
  assert.equal(I.terminalKind, 'session_close');
  assert.equal(I.terminalOutcome, 'CONFIRMED');
  assert.deepEqual(I.terminalPayloadKeys, ['authenticated','association','closing','connectionGeneration']);
  assert.deepEqual(R, {
    visits: I.route,
    requiredEvents: I.requiredEvents,
    baizhiChoices: I.baizhiChoices,
    returnScene: I.returnScene,
    returnGate: I.returnGate,
    sentinelExit: I.sentinelExit,
    captureBinding: I.captureBinding
  });
  assert.equal(I.driver, 'operator');
  assert.deepEqual(I.phaseOrder, ['new','continue']);
  assert.deepEqual(I.rpcArguments, ['--v1-ready','--v1-token']);
  assert.deepEqual(R.visits, [
    'rs_core_room','gh_entry_maintenance','gh_power_room','gh_gate_a',
    'gh_central_shaft','gh_lockdown','gh_bio_isolation','gh_gate_b',
    'gh_deep_decon','gh_sentinel_arena','gh_beacon','gh_exit'
  ]);
  assert.deepEqual(R.requiredEvents,
    ['hive_power','hive_lockdown','hive_extraction']);
  assert.deepEqual(R.baizhiChoices, ['taken','left','unresolved']);
  assert.equal(R.returnGate, 'gh_extraction_return_to_rs');
  assert.equal(routeModel(R.visits), true);
  assert.equal(routeModel(R.visits.slice(0, -1)), false);
  assert.throws(() => routeModel(['mh_fog_pier']));
  assert.equal(routeModel([
    ...R.visits.slice(0, 3), 'gh_entry_maintenance', ...R.visits.slice(3)
  ]), true);
});

test('SOURCE_CONTRACT: isolation precedes real production scene', () => {
  ordered(body('_boot_main'), ['_read_config','_verify_profile','load(']);
  has(body('_boot_main'), 'instantiate(');
  has(body('_boot_main'), 'add_child(');
  const profile = body('_verify_profile');
  has(profile, 'OS.get_user_data_dir(');
  has(profile, 'ProjectSettings.globalize_path(');
  const config = body('_read_config');
  has(config, '_expected_user_dir = OS.get_environment("WUXIAN_GH_EXPECTED_USER_DIR")');
  has(config, 'if not _absolute_path(_expected_user_dir)');
  has(config, 'if not _same_path(str(phase_config.get("expectedUserDir", ""))');
  has(config, ', _expected_user_dir): return false');
  has(profile, 'return _same_path(user_dir, _expected_user_dir) and _same_path(global_user_dir, _expected_user_dir)');
  const clock = body('_phase_limit_ms');
  has(clock, 'limits.get("phaseSeconds", 0)');
  has(clock, 'float(seconds) * 1000.0');
  assert.doesNotMatch(clock, /phaseMs/);
  const attached = body('_attach_main');
  for (const value of ['rpc','response','failed','connection_changed',
    'panels','intent','frame_committed']) has(attached, value);
  assert.doesNotMatch(source, /OS\.set_environment\s*\(/);
  assert.doesNotMatch(source, /(?:StreamPeerTCP|TCPServer)\.new\s*\(/);
  assert.doesNotMatch(source, /\.request\s*\(/);
  assert.doesNotMatch(source, /Input\.parse_input_event\s*\(/);
  assert.doesNotMatch(source, /set_process\s*\(\s*false\s*\)/);
  assert.doesNotMatch(source, /reported_key\s*=/);
});

test('SOURCE_CONTRACT: one-shot receipts and unknown are terminal', () => {
  const receipt = body('_record_receipt');
  for (const value of ['mutation_ledger','formal_new','formal_continue',
    'formal_scene_ready','formal_save','session_close','_stop_unknown'])
    has(receipt, value);
  const pending = body('_observe_pending');
  has(pending, 'pending');
  has(pending, 'connection_generation');
  const unknown = body('_stop_unknown');
  has(unknown, 'UNKNOWN');
  has(unknown, '_write_event');
  assert.doesNotMatch(unknown,
    /formal_new|formal_continue|formal_scene_ready|formal_save/);
  const close = body('_request_close_once');
  ordered(close, ['close_requested','_submit_close']);
  const terminal = body('_record_close_if_valid');
  has(terminal, '_write_event("session_close", "CONFIRMED"');
  for (const key of I.terminalPayloadKeys) has(terminal, key);
  has(terminal, 'connection_generation');
  const route = body('_advance_route');
  has(route, '_write_event("route_complete", "CONFIRMED"');
  has(route, '"visits": _route.visits.duplicate()');
  assert.equal(onceModel([
    {method:'formal_new',outcome:'CONFIRMED'}
  ], 'new'), true);
  assert.equal(onceModel([
    {method:'formal_new',outcome:'UNKNOWN'}
  ], 'new'), false);
  assert.equal(onceModel([
    {method:'formal_continue',outcome:'CONFIRMED'},
    {method:'formal_continue',outcome:'CONFIRMED'}
  ], 'continue'), false);
});

test('SOURCE_CONTRACT: native context and completed-frame evidence', () => {
  const live = body('_validate_live');
  for (const value of ['P.snapshot','P.same','entryToken','catalog',
    'current_key','scene_hash','viewport_rect','get_viewport_rect'])
    has(live, value);
  const bind = body('_bind_capture');
  has(bind, 'duplicate(true)');
  for (const key of R.captureBinding) has(bind, key);
  ordered(body('_capture'), [
    '_bind_capture','queue_redraw','RenderingServer.frame_post_draw',
    '_validate_live','_bind_capture','_write_capture'
  ]);
  const write = body('_write_capture');
  has(write, 'FileAccess.file_exists');
  has(write, 'save_png');
  has(write, '"locator": "phases/" + _phase');
  assert.doesNotMatch(write, /resize\s*\(|crop\s*\(|blit_rect/);
  const a = Object.fromEntries(R.captureBinding.map(k => [k,{value:1}]));
  const b = structuredClone(a);
  assert.equal(bindingModel(a, b), true);
  b.snapshot.value = 2;
  assert.equal(bindingModel(a, b), false);
  const observe = body('_observe_snapshot');
  ordered(observe, ['_validate_live','_advance_route','_write_event']);
  const status = body('_observe_status');
  has(status, 'P.status');
  has(status, 'P.same');
  const eventWriter = body('_write_event');
  const prepare = body('_prepare_acceptance');
  for (const label of ['new-rs','gh-entry','sentinel-encounter','gh-exit-extraction','returned-rs-saved','continue-rs']) has(prepare, label);
  has(prepare, 'limits.captureCap != 6 or actual != expected');
  const stage = body('_capture_stage_ready');
  for (const anchor of ['_route_index == 1','_route_index == 2','_route_index == 10','_sentinel_defeat_confirmed','_extraction_confirmed','_return_gate_confirmed','_save_confirmed','_returned_to_station','_phase_b_comparison_matches_current']) has(stage, anchor);
  has(stage, 'gh_sentinel_arena_sentinel_01');
  has(stage, 'not encounter.get("phase") in ["dead", "inactive"]');
  has(body('_queue_captures_for_scene'), '_capture_stage_ready(label)');
  has(body('_capture_scene_matches'), 'if not _capture_stage_ready(label): return false');
  has(body('_phase_a_ready_for_save'), '_captures_complete(true)');
  has(body('_captures_complete'), 'before_save and item.id == "returned-rs-saved"');
  has(body('_register_mutation'), '_save_request_count != 1');
  has(body('_register_mutation'), 'association.get("exitAfter") != false');
  has(body('_record_save_if_valid'), 'association.get("exitAfter") != false');
  ordered(body('_record_save_if_valid'), ['_phase_a_ready_for_save','phase_summary','_save_confirmed = true','_queue_captures_for_scene']);
  has(body('_on_intent'), 'save_exit_precedes_post_save_frame');
  ordered(body('_maybe_finish_phase'), ['_captures_complete','_capture_running','_phase == "new"','_save_confirmed','_request_close_once']);
  has(body('_record_close_if_valid'), 'not _captures_complete()');
  assert.doesNotMatch(source, /_maybe_finish_phase_b/);
  has(eventWriter, 'EVENT_SCHEMA');
  has(eventWriter, 'sourceBaselineReference');
  has(eventWriter, '_events_path');
});
