#!/usr/bin/env node
import assert from "node:assert/strict";
import { runCorpus } from "./validator.mjs";
const result = runCorpus(process.cwd());
assert.equal(result.positive.total, 6); assert.equal(result.positive.passed, 6);
assert.equal(result.negative.total, 25); assert.equal(result.negative.rejected, 25); assert.equal(result.negative.allRejected, true);
assert.equal(result.libRsProof.hashUnchanged, true); assert.equal(result.libRsProof.statusUnchanged, true);
assert.equal(result.runtimeValidated, false); assert.equal(result.formalAccepted, false); assert.equal(result.blockerClosed, false);
console.log(JSON.stringify({ positive: result.positive.passed, negative: result.negative.rejected, allNegativeExpectationsMet: result.negative.allRejected }));
