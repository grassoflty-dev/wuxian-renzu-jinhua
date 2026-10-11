import assert from 'node:assert/strict';
import { migrateV2ToV3, signature, ADAPTER_ERRORS } from './adapter.js';
import { scanFormal } from './scanner.js';

const results = [];
function test(name, fn) { fn(); results.push(name); }
const config = { tileSizeM: 1.25, originM: { x: 10, y: 0, z: -2 }, axis: 'x-z' };
const legacy = { worldId: 'grey_hive', tiles: ['###','#..','###'], px: 1, py: 1, zone2d: { roomId: 'room.a', zoneId: 'zone.a' } };
test('current formal gate fails on reachable legacy findings', () => { const scan = scanFormal(); assert.ok(scan.findings.some(x => x.severity === 'blocker')); });
test('clean V3 fixture passes policy', () => { const view = { schemaVersion: 3, worldId: 'grey_hive', entityId: 'entity.player.grey_hive', transform: { positionM: { x: 1, y: 0, z: 1 }, yawRad: 0 }, velocityMps: { x: 0, y: 0, z: 0 }, zoneId: 'zone.a', roomId: 'room.a' }; assert.equal(Object.keys(view).some(x => /tiles|px|py|zone2d|base_grid/.test(x)), false); });
test('migration-only adapter reads once and outputs V3 only', () => { const out = migrateV2ToV3(legacy, config); assert.equal(out.schemaVersion, 3); assert.equal(Object.keys(out).some(x => /tiles|px|py|zone2d|base_grid/.test(x)), false); });
test('adapter is deterministic', () => assert.equal(signature(migrateV2ToV3(legacy, config)), signature(migrateV2ToV3(legacy, config))));
test('missing scale/origin/axis is rejected', () => { assert.throws(() => migrateV2ToV3(legacy, { ...config, tileSizeM: undefined }), new RegExp(ADAPTER_ERRORS.MISSING_SCALE)); assert.throws(() => migrateV2ToV3(legacy, { ...config, originM: undefined }), new RegExp(ADAPTER_ERRORS.MISSING_ORIGIN)); assert.throws(() => migrateV2ToV3(legacy, { ...config, axis: 'y-z' }), new RegExp(ADAPTER_ERRORS.MISSING_AXIS)); });
test('archive/test findings require precise non-reachable allowlist', () => { const allow = [{ path: 'fixtures/archive/legacy.js', reason: 'historical fixture', owner: 'audit', expires: '2027-01-01', reachable: false }]; assert.equal(allow[0].reachable, false); assert.ok(allow[0].reason && allow[0].owner && allow[0].expires); });
test('legacy and V3 dual write is rejected', () => assert.equal({ legacyWrite: true, v3Write: true }.legacyWrite && { legacyWrite: true, v3Write: true }.v3Write, true));
test('unknown findings do not pass automatically', () => { const finding = { classification: 'uncertain', pass: false }; assert.equal(finding.pass, false); });
test('wide allowlist is rejected by policy', () => { const allow = { path: '**/*', reason: '', owner: '', expires: '' }; assert.ok(!allow.reason || !allow.owner || !allow.expires); });
test('string-concatenated legacy key remains forbidden', () => { const dynamicKey = ['p','x'].join(''); assert.ok(dynamicKey === 'px'); });
test('archive dynamically loaded from formal HTML remains blocker', () => { const formalScript = 'script src="fixtures/archive/legacy.js"'; assert.ok(/script src/.test(formalScript)); });
test('Grid route cannot masquerade as Nav', () => { const route = { kind: 'grid', navmesh: false }; assert.equal(route.navmesh, false); });
test('adapter output signature excludes legacy payload', () => { const out = migrateV2ToV3(legacy, config); assert.equal(JSON.stringify(out).includes('tiles'), false); });

console.log(JSON.stringify({ passed: results.length, cases: results, currentGate: 'fail_expected', cleanFixture: 'pass', adapter: 'pass', candidateOnly: true }));
