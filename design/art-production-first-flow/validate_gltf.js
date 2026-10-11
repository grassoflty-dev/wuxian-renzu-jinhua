/* Khronos glTF-Validator runner; install the pinned package outside the repo. */
const fs = require("node:fs");
const path = require("node:path");
const validatorPath = process.env.GLTFS_VALIDATOR_MODULE || require.resolve("gltf-validator");
const validator = require(validatorPath);
const root = path.resolve(__dirname, "../..");
const assets = path.join(root, "server-rs/release-ui/assets/grey-hive-first-flow");
const files = ["field-operative-rig.glb", "gate-a.glb", "grey-hive-first-flow-room.glb", "power-console.glb"];

async function main() {
  const results = [];
  for (const file of files) {
    const report = await validator.validateBytes(new Uint8Array(fs.readFileSync(path.join(assets, file))), { uri: file });
    results.push({ file, errors: report.issues.numErrors, warnings: report.issues.numWarnings, infos: report.issues.numInfos, messages: report.issues.messages });
  }
  const output = JSON.stringify({ validator: "gltf-validator@2.0.0-dev.3.10", results }, null, 2) + "\n";
  if (process.argv[2]) fs.writeFileSync(path.resolve(process.argv[2]), output);
  process.stdout.write(output);
  if (results.some((item) => item.errors || item.warnings)) process.exitCode = 1;
}

main().catch((error) => { console.error(error); process.exitCode = 1; });
