import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const [fmtArg = "1", testArg = "1", runArg = "1"] = process.argv.slice(2);
const fmt = Number(fmtArg);
const test = Number(testArg);
const run = Number(runArg);
const resultPath = fileURLToPath(new URL("../results/tauri-world-routing-tp14c1.json", import.meta.url));
const report = JSON.parse(fs.readFileSync(resultPath, "utf8"));
const scenarios = Array.isArray(report.scenarios) ? report.scenarios : [];
const processScenario = scenarios.find(item => item.id === "process_normal_exit");
if (processScenario) {
  processScenario.actual = { exitCode: run };
  processScenario.pass = run === 0;
  processScenario.error = run === 0 ? null : `E_PROCESS_EXIT_${run}`;
}
const scenarioPass = scenarios.length >= 14 && scenarios.every(item => item.pass === true);
const ackPass = !report.callbacks?.ackErrors?.length && (report.callbacks?.ackRecords?.length || 0) > 0;
const machinePass = fmt === 0 && test === 0 && run === 0 && scenarioPass && ackPass;
report.commandExitCodes = { cargoFmt: fmt, cargoTest: test, cargoRun: run };
report.validationRunner = {
  cargoFmt: { exitCode: fmt },
  cargoTest: { exitCode: test },
  cargoRun: { exitCode: run },
  finalizer: "external process observed by PowerShell runner"
};
report.machinePass = machinePass;
report.status = machinePass ? "passed_real_webview_candidate" : "blocked_probe_failure";
report.stop = {
  ...(report.stop || {}),
  processExitObservedByHarness: true,
  processExitCode: run,
  processExitObservation: "cargo run exit code finalized by external runner"
};
report.evidenceLevel = machinePass ? "real_windows_webview_candidate" : "real_windows_webview_probe_incomplete";
fs.writeFileSync(resultPath, `${JSON.stringify(report, null, 2)}\n`);
console.log(JSON.stringify({ status: report.status, machinePass, scenarios: scenarios.length, fmt, test, run }));
