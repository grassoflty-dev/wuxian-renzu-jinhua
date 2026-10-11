'use strict';
// Isolated regression: real vendored Babylon math, no browser, server or save state.
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');
const B = require('../server-rs/release-ui/vendor/babylon.js');
const source = fs.readFileSync(path.join(__dirname, '../server-rs/release-ui/js/tps-renderer.js'), 'utf8');
const match = source.match(/function boxMatrix\(x, y, z, sx, sy, sz\)\s*\{[^}]+\}/);
assert.ok(match, 'Renderer boxMatrix helper must exist');
const boxMatrix = vm.runInNewContext('(' + match[0] + ')', { B });
function near(actual, expected, label) {
  actual.forEach((value, index) => assert.ok(Math.abs(value - expected[index]) < 1e-5,
    `${label}[${index}]: expected ${expected[index]}, got ${value}`));
}
for (const [label, position, dimensions] of [
  ['floor', [4, -0.06, 7], [0.995, 0.12, 0.995]],
  ['water', [3, -0.16, 5], [0.995, 0.06, 0.995]],
  ['wall', [0, 0.675, 0], [0.98, 1.35, 0.98]],
]) {
  const matrix = boxMatrix(...position, ...dimensions);
  const scale = new B.Vector3(), rotation = new B.Quaternion(), translation = new B.Vector3();
  assert.equal(matrix.decompose(scale, rotation, translation), true, label + ' must decompose');
  near(scale.asArray(), dimensions, label + ' dimensions');
  near(translation.asArray(), position, label + ' position');
  near(B.Vector3.TransformCoordinates(new B.Vector3(0.5, 0.5, 0.5), matrix).asArray(),
    position.map((value, index) => value + dimensions[index] / 2), label + ' corner');
  console.log('PASS ' + label + ': position, dimensions and transformed corner');
}
