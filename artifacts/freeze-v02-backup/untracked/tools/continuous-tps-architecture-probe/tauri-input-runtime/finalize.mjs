import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const resultPath = path.resolve(__dirname, '../results/tauri-input-runtime-tp06b1.json');
const cargoTest = Number(process.argv[2]);
const cargoRun = Number(process.argv[3]);
const result = JSON.parse(fs.readFileSync(resultPath, 'utf8'));
result.commandExitCodes = { cargoTest, cargoRun };
result.stop = result.stop || {};
result.stop.processExitObservedByHarness = cargoRun === 0;
result.stop.processExitObservation = `cargo run exit code ${cargoRun}`;
result.validationRunner = {
  finalizedAtUtc: new Date().toISOString(),
  cargoTestExitCode: cargoTest,
  cargoRunExitCode: cargoRun,
  processExitObserved: cargoRun === 0
};
fs.writeFileSync(resultPath, `${JSON.stringify(result, null, 2)}\n`);
console.log(JSON.stringify({ resultPath, cargoTest, cargoRun, processExitObserved: cargoRun === 0 }));
