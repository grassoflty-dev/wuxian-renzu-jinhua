# EDGE-BROWSER-LIFECYCLE-03-DIAGNOSTIC-01

Status: **DIAGNOSTIC_ONLY / NOT_READY_FOR_CANDIDATE**.

The one authorized real inventory diagnostic passed naturally and did not emit an identity mismatch. The prior EDGE02 cleanup failure was **NOT REPRODUCED** in this run; its root cause remains **UNKNOWN**. This package adds observation only and does not establish a lifecycle repair, full Web acceptance or native readiness. Further browser runs stopped after this one attempt.

## Sources, gate and identities

The complete current AGENTS/coordination rules, release source register and ENG authority, formal diagnostic package, fixed public engineering handoff and unchanged full EDGE01/EDGE02 failure reports were read. The formal package's precise scope governs this stage. Private source material was not copied.

The preceding implementation, merge and report were remotely preserved with coordinator-verified backups. Slot2 cache gate evidence `20261006T111821Z-18d3beee157642fd80547b171b81b2c0` records exact slot1 apps/web/dist deletion: 260 files, 244,623,490 logical bytes, zero errors, no residual target and unchanged clean Git states. Observed C free-space increase was 245,260,288 bytes. Preflight and final process checks recorded no current engineering consumer, and current-task use was stopped. Slot1 verified this gate before changing source. Dependencies, failed profiles, logs, source and all other caches remain protected. The coordinator independently confirmed this gate.

- Fixed clean starting HEAD: `0232cba8f8c0ced234219a24097cdce13c68571a`, parent `04671aba6e76be039589ef8c40d90cbc2fc724ec`, then dependency merge `81ceebf363722b427596feb9c4e6d2172c16d299` with original EDGE01 and label parents.
- Diagnostic implementation: `3bf66da0a938dff0acbda0958ccd1dbf6c24df51`, parent fixed0232; tree `c54938e5b55a040ce8ab4d43127b5bec60f9858b`. Commit includes `[skip ci]`.
- Existing independent slot1 clone, new branch and sole temporary remote `codex/edge-browser-lifecycle-03-20261006`; public origin `https://github.com/grassoflty-dev/wuxian-renzu-jinhua.git`.
- Initial remote inspection encountered a Schannel SSL handshake failure; it was not proof of ref absence. A later successful inspection found the new ref absent before branch creation/push. Implementation was pushed normally and ls-remote matched its complete SHA. No force, main/candidate changes, PR, tag, Actions or native stage.
- This report is a separate second commit. Its final HEAD and exact remote match are delivered to the coordinator after push.

## Exact change scope and behavior

Only three authorized files changed relative to fixed0232:

1. `apps/web/tests/support/browser-process-ownership.mjs`: a private diagnostic constructor and its use at the existing refresh known-PID mismatch and pre-stop second-query mismatch branches.
2. `apps/web/tests/browser-session.mjs`: two parameterized regression groups and SHA-256 expectations.
3. `docs/testing/EDGE_BROWSER_LIFECYCLE_03_DIAGNOSTIC_REPORT.md`: this new report.

The original E_PROCESS_IDENTITY code, message prefix, exact sameProcess comparison, admission, registry expected identity, hard refusal, retries and termination behavior remain. No additional process query, stop, deletion or credential collection was introduced. An observation failure cannot replace the original refusal. The actual row is the row returned by that failing internal query, not an external later inventory. Expected and actual summaries are captured synchronously into frozen objects without modifying input rows.

Enumerable error.diagnostic and the message suffix `identityDiagnostic=<compact JSON>` contain stage (`refresh` or `pre-stop`), UTC, PID, expected/actual PID/parent/created and changedFields among pid/created/exe/command. Exe and command are represented only by state (null/missing/invalid/empty/present), string length and SHA-256 over the entire UTF-8 string. Empty strings have a zero length and the empty-string hash; nonstrings have null hash/length. No raw executable, command or profile path is included. Hashes are observations only; the ownership decision still uses exact original comparisons.

The two groups exercise real createProcessOwnership paths, creation changes, executable/command changes and null/empty/missing values, simultaneous changes, parent observation, repeated mismatch against unchanged expected identity, immutable snapshots, parsable message JSON, full-string hashes and lengths, absence of private fixture strings, and second-query pre-stop refusal with zero stop calls. Existing 53 session, 20 cleanup and 8 layout regressions remain unchanged. Total focused tests: 83.

## Ordered validation ledger

All commands were strictly serial, each with its own immutable RunID, TEMP/profile and metadata. Every runtime preflight met at least 12 GiB disk and 6 GiB RAM. All three runs exited naturally, Intervened=false, TimedOut=false and Survivors=[]. Counts are tests/pass/fail/skipped/cancelled/todo. Node 24.14.0, npm 11.9.0 and the existing installed Edge/dependencies were retained; dependency provenance is unchanged lockfile/node_modules and EDGE02 npm ci exit0. No install or production build was run in this stage.

The helper run is dirty evidence above fixed0232 (tree `9b2f09619f14a7675239cb82a405553ecae03634`), with exactly the two implementation/test files modified. Its base tree does not identify dirty contents. After helper PASS, the implementation was committed/pushed, and the tsc and inventory commands both recorded clean implementation3bf66da/treec54938 with Dirty=[].

| RunID | Phase | UTC start / end | JST start / end | Seconds / budget | Exit | Counts |
| --- | --- | --- | --- | --- | --- | --- |
| 20261006T112508Z-512d8a46bb764455a640b40c7f59ba35 | edge03-dirty-helper | 2026-10-06T11:25:08.9368404Z / 2026-10-06T11:25:10.0189278Z | 2026-10-06T20:25:08.9368404+09:00 / 2026-10-06T20:25:10.0189278+09:00 | 1.082 / 120 | 0 | 83/83/0/0/0/0 |
| 20261006T112535Z-b3eaab430be04c7da149b7c0867a3d93 | edge03-clean-tsc | 2026-10-06T11:25:36.1356805Z / 2026-10-06T11:25:40.8164748Z | 2026-10-06T20:25:36.1356805+09:00 / 2026-10-06T20:25:40.8164748+09:00 | 4.681 / 120 | 0 | — |
| 20261006T112548Z-0423f728e2704a78836aed21cf603390 | edge03-clean-inventory-diagnostic | 2026-10-06T11:25:48.9168316Z / 2026-10-06T11:25:53.8525528Z | 2026-10-06T20:25:48.9168316+09:00 / 2026-10-06T20:25:53.8525528+09:00 | 4.936 / 180 | 0 | 1/1/0/0/0/0 |

Commands, in order: node --test --test-concurrency=1 tests/browser-session.mjs tests/accessibility-browser-cleanup.mjs tests/hub-layout-readiness.test.mjs (120s); npm exec -- tsc -p tsconfig.json (120s); node --test --test-concurrency=1 tests/inventory-panel-browser.mjs (180s). tsc regenerated necessary imported dist/ui modules after the audited dist deletion. This is a fixed diagnostic run, not a final production bundle or clean full-Web sequence.

## One real inventory result and evidence limits

Inventory Run `20261006T112548Z-0423f728e2704a78836aed21cf603390` observed launcher exit0, a CDP-ready snapshot containing 13 owned identities, then connected/page-ready/fixture-ready milestones and profile-released. Its unchanged authoritative rows, keyboard focus, pending state and close/epoch safety behavior test passed, 1/1, naturally, exit0. Suite duration was 3969.9261 ms; outer run duration 4.9357212 s. No error tree or identityDiagnostic JSON was emitted. Metadata profile directories are empty and Survivors=[]; no external intervention occurred.

This provides one successful cleanup observation only. It supplies no internal expected/actual mismatch for the prior failure, so no PID reuse, field disappearance, same-creation race or final root cause is established. No further inventory attempt was made. EDGE02 Run `20261006T111224Z-3f056479f3d44e8ea1e45cd69c3d347b` remains a natural cleanup FAIL with retained profile and an AggregateError member saying Owned PID was reused or changed; its original output, metadata and report are untouched. No later snapshot replaces that evidence. Earlier EDGE01 and historical Round4 failures/intervention remain unchanged. F01/F02/F03 and lifecycle acceptance remain pending and block integration/full/native.

## Not run and preservation

Complete npm test, focused real-browser combination, browser repeats, final production build/bundle verification, Rust build/tests, candidate EXE, Windows native acceptance and visual acceptance are all **NOT_RUN**. The diagnostic does not authorize safety reclassification, retries, candidate integration or expanded source edits. All runtime commands have stopped; no cache or evidence was deleted by slot1. Current dist contains only this diagnostic's regenerated test modules and is not a native-ready bundle. Dependencies, original failure profiles, source, old refs and all raw evidence remain.

The coordinator/planning must first back up the original implementation/report commits, then combine the fixed EDGE02 static audit with the retained evidence to determine a formal next package. Single-run PASS does not close the preceding cleanup failure.

## Evidence fingerprints

Full raw stdout/stderr, process command lines, paths, profile contents and saves remain local only. The following fingerprints identify retained originals; no full system commands or private source were published.

| RunID | stdout.log SHA-256 | stderr.log SHA-256 | metadata.json SHA-256 |
| --- | --- | --- | --- |
| 20261006T112508Z-512d8a46bb764455a640b40c7f59ba35 | 41CD6E08A8557C0397C1876C6172AE88291D95AA0489F70BEBF5B1B09F1D3049 | 7EB70257593DA06F682A3DDDA54A9D260D4FC514F645237F5CA74B08F8DA61A6 | 7F638F7638A809C6C7A05448802584337EBED2FC8BF78ABC2669D9E8C225C4A0 |
| 20261006T112535Z-b3eaab430be04c7da149b7c0867a3d93 | 7EB70257593DA06F682A3DDDA54A9D260D4FC514F645237F5CA74B08F8DA61A6 | 7EB70257593DA06F682A3DDDA54A9D260D4FC514F645237F5CA74B08F8DA61A6 | 2C542F29F6989DF0D1BF303536EF9AC72B20B1AAEB9DA790895DCB627F4D5BD6 |
| 20261006T112548Z-0423f728e2704a78836aed21cf603390 | 2E5F0037AEBA1F44D8F75F7D745498F3DCEB54EAFB8F26B6643C5A9E70616A7D | 7EB70257593DA06F682A3DDDA54A9D260D4FC514F645237F5CA74B08F8DA61A6 | 179D64B624D3B461E27473D63444D6ACAC6F265008E0C3C80FFA29BFA3F52E6A |
