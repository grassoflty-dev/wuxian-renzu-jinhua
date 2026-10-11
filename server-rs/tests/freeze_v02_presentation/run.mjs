import protocol from "./protocol.test.cjs";
import renderer from "./renderer-lifecycle.test.cjs";
import hud from "./hud.test.cjs";
const results = [protocol(), renderer(), hud()];
console.log(JSON.stringify({ suite: "freeze-v02-presentation", passed: results.every(x => x.passed), results }));
if (!results.every(x => x.passed)) process.exitCode = 1;
