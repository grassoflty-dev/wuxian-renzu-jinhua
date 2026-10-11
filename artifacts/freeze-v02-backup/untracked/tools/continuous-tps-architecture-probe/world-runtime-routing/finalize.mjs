import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const resultPath = path.resolve(__dirname, '../results/world-runtime-routing-tp14b1.json');
const cargoFmt = Number(process.argv[2]);
const cargoTest = Number(process.argv[3]);
const cargoRun = Number(process.argv[4]);
const result = JSON.parse(fs.readFileSync(resultPath, 'utf8'));
result.commandExitCodes = { cargoFmt, cargoTest, cargoRun };
result.validationRunner = {
  finalizedAtUtc: new Date().toISOString(),
  cargoFmtExitCode: cargoFmt,
  cargoTestExitCode: cargoTest,
  cargoRunExitCode: cargoRun,
  allCommandsPassed: cargoFmt === 0 && cargoTest === 0 && cargoRun === 0
};
fs.writeFileSync(resultPath, `${JSON.stringify(result, null, 2)}\n`);
console.log(JSON.stringify({ resultPath, cargoFmt, cargoTest, cargoRun, allCommandsPassed: result.validationRunner.allCommandsPassed }));
