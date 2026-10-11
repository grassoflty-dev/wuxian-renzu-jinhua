import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const resultPath = path.resolve(__dirname, '../results/tauri-runtime-observability-tp17c1.json');
const cargoTest = Number(process.argv[2]);
const cargoRun = Number(process.argv[3]);
const report = JSON.parse(fs.readFileSync(resultPath, 'utf8'));
const processScenario = report.scenarios?.find(item => item.id === 'process_normal_exit');
if (processScenario) {
  processScenario.actual = { exitCode: cargoRun };
  processScenario.pass = cargoRun === 0;
  processScenario.error = cargoRun === 0 ? null : 'E_PROCESS_EXIT_NONZERO';
  processScenario.evidenceLevel = 'external_command_exit_code';
}
report.commandExitCodes = { cargoTest, cargoRun };
report.stop = report.stop || {};
report.stop.processExitObservedByHarness = cargoRun === 0;
report.stop.processExitObservation = `cargo run exit code ${cargoRun}`;
report.machinePass = Array.isArray(report.scenarios) && report.scenarios.length >= 12 && report.scenarios.every(item => item.pass === true);
report.status = report.machinePass ? 'passed_real_webview_candidate' : 'blocked_probe_failure';
report.validationRunner = {
  finalizedAtUtc: new Date().toISOString(),
  cargoTestExitCode: cargoTest,
  cargoRunExitCode: cargoRun,
  processExitObserved: cargoRun === 0
};
fs.writeFileSync(resultPath, `${JSON.stringify(report, null, 2)}\n`);
console.log(JSON.stringify({ resultPath, cargoTest, cargoRun, machinePass: report.machinePass }));
