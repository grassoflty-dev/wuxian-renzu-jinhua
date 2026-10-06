# EDGE-BROWSER-CONNECTION-SAFETY-04

Local status: **IMPLEMENTED / UNIT_TESTED_CANDIDATE / NOT_READY_FOR_CANDIDATE**.

Pure mock regressions pass on the clean implementation commit. This is local evidence for the fixed static F02/D01/D02 branches, not Windows transport or candidate acceptance. The original dynamic cleanup identity failure remains unresolved: EDGE03's one inventory run did not reproduce it and its root cause remains UNKNOWN. No real browser or build ran in EDGE04.

## Authority, prerequisite and exact identities

Read the current AGENTS/coordination rules including the three execution slot limit, release source register/ENG authority, fixed public engineering handoff, unchanged complete EDGE01/02/03 reports, two fixed static audits (slot2 evidence111114 and111821), actual diagnostic source and formal EDGE04 package. The planning-issued EDGE04-CORRECTION-01 authorized only the immediate handshake status guard and one synchronous-open regression after the first mock failure. Private supplement material was not copied.

The coordinator verified previous03 remote preservation and backed up its original implementation/report. Before this package changed source, slot2 cache gate `20261006T113057Z-79ca76b627a946c0867d226933ee7f29` deleted exactly slot1 apps/web/dist:97 generated files/731,760 logical bytes, zero errors/no residual path, unchanged clean Git states. C free-space observation increased950,272 bytes. Main independently read the result, checked target absent and fixed clean HEAD, then explicitly confirmed the gate. Slot1 did not perform deletion. Sources/dependencies/evidence/profiles and other paths remain protected.

- Fixed clean base/report: `5d42696fe5d01660b06777800f32ebf88695b515`, tree `e56b2d5b96ae7dce728d89d9b29c4d58ef002bbb`.
- Parent diagnostic implementation: `3bf66da0a938dff0acbda0958ccd1dbf6c24df51`, then fixed02 `0232cba8f8c0ced234219a24097cdce13c68571a`.
- EDGE04 implementation: `aebabd80c33f42caabde7d98dfa2e421798cf045`, parent fixed5d426, tree `8153d2a761c931a9b03944628a01274187ff5d72`. Commit includes `[skip ci]`.
- Existing independent slot1 clone; new local/sole temporary remote `codex/edge-browser-connection-safety-04-20261006`. Origin remains `https://github.com/grassoflty-dev/wuxian-renzu-jinhua.git`. Local/remote ref was absent before branch creation; implementation normal push and ls-remote matched the full SHA.
- Report is a separate second commit after implementation. Final report HEAD and exact verified remote identity are supplied in delivery. The clean implementation test identity below is distinct from the later report HEAD.

## Complete autonomous change scope

1. `apps/web/tests/support/browser-session.mjs`: attempted transport registry, immediate registration/bounded close confirmation, real session and fallback wiring, last page-query cause propagation and external cancellation forwarding.
2. `apps/web/tests/support/browser-process-ownership.mjs`: listener-after-query fresh identity recheck with remaining budget/signal guards and final owned snapshot.
3. `apps/web/tests/browser-session.mjs`:19 new exported-path mock tests; original83 expanded tests unchanged except the import of the new registry factory.
4. `docs/testing/EDGE_BROWSER_CONNECTION_SAFETY_04_REPORT.md`: this new report.

No browser-cleanup helper, third-slot consumer test, actual browser suites, original cleanup/layout, package/lock, production source, Rust/assets/private/workflow/old report changed. No third-slot branch merged. No candidate/main/old refs/PR/tag/Release/Actions/visibility changes, reset/stash/force, clone or new worktree.

## F02: attempted transports and disposal

connectCdp constructs the CDP handle and passes it to onAttempt before any handshake wait. Real createBrowserSession registers each attempted handle in createAttemptedTransportRegistry, included in its cleanup resources, for both normal connect and fallback browser transport. Successful onConnection/onReady wiring remains. Default exported calls remain compatible.

Handshake error/close/deadline/abort uses the same handle.close as connected cleanup. Close rejects pending requests and waits for the socket close event or checks explicit CLOSED, with an independent budget capped at1500ms. A synchronous close throw or an unconfirmed close timeout stays visible alongside the original handshake error/cause through withBrowserCleanup. A failed close remains in the registry for later cleanup; handles are removed only after successful confirmation. Listeners/timers owned by the handshake and close wait are removed on settlement; pending request timers are rejected/cleared. Late open cannot authorize a returned connection after cancellation. The registry operates only on helper-created sockets.

The production connection default still uses one deadline capped at15000ms across page/handshake/fresh sampling. Failed transport disposal has a separate cleanup budget: error delivery may occur after the connection deadline plus disposal time. No claim that total failure output is always within exactly15s. Registry cleanup may contain multiple attempts and is also subject to the existing outer cleanup budget; a bounded wait does not prove an uncooperative native operation has drained. Fake transport evidence is not proof of native WebSocket shutdown.

## D01: page timeout cause

waitForPage reports each latest retryable ownership failure synchronously to connectOwnedBrowser's lastError. If the actual outer timer aborts during page polling, E_BROWSER_CONNECT_TIMEOUT retains that last original error object in cause. The standalone waitForPage API remains compatible. Only E_PROCESS_INCOMPLETE/E_PROCESS_QUERY/E_PROCESS_QUERY_TIMEOUT retain retry status. Hard identity/foreign listener/wrong URL/spawn/transport errors are not admitted or reclassified; fresh-phase last-cause behavior remains covered.

## D02: listener identity sampling

inspectPort first refreshes registered identities, queries the selected port, checks that numeric PID against the first owned root snapshot, then refreshes again. The last snapshot must contain the same exact registered pid/created/exe/command with valid session root/profile/executable/debug-port conditions. It is this last verified owned snapshot that ready returns. Replacement creation/executable/command and field disappearance keep hard E_PROCESS_IDENTITY with the inherited03 internal mismatch diagnostic; missing root and foreign listener reject E_CDP_OWNER. No stop call is performed by verification.

Each refresh/listener receives AbortSignal and remaining deadline; per-query waits are bounded and post-await guards reject late success. A standalone inspection has a3000ms default sample budget; real connection passes its remaining total budget. sameProcess, registry admission, pre-stop refresh/inner Windows handle checks remain unchanged. These sequential observations are not an atomic OS transaction: listener state can change after sampling. No claim to eliminate all races, exhaustively attribute orphans, drain signal-ignoring actions or authorize unknown process termination/profile deletion.

## Regression coverage and first failure

Preserved baseline:55 session +20 cleanup +8 layout=83. New18 tests cover failed handshake close throws/delayed confirms/never confirms with a real attempted registry and later retry; cancellation/deadline with late open; the page outer timer's latest original three retryable causes; hard page identity without polling; listener-side same-PID creation/executable/command replacement, missing fields and disappearing root; stable last-snapshot/remaining-budget/signal; late/aborted listener refusal. The existing connected socket/body+cleanup dual-error regressions still pass. No copied ownership/connection algorithm or altered old assertion.

The first run had101 tests/100PASS/1FAIL, natural exit1. The existing NotOpening fixture has neither an OPEN constant nor readyState; new immediate status code compared undefined===undefined and incorrectly completed handshake. Its unchanged assertion at browser-session.mjs190 reported Missing expected rejection for E_CDP_CONNECT_TIMEOUT. This is a newly introduced mock compatibility bug, not the earlier dynamic cleanup failure. Source/testing stopped and the exact evidence was sent to planning.

EDGE04-CORRECTION-01 explicitly authorized Number.isInteger on both state/OPEN/CLOSED followed by exact comparison. No truthy check or fixture/constants/assertion alteration. One additional real connectCdp test registers the handle then synchronously sets valid OPEN and emits open in onAttempt, proving the pre-listener open path still works and later close removes listeners. Corrected dirty run and clean implementation run both pass102/102. The original first FAIL remains immutable.

## Actual ordered pure mock ledger

Every run used the same command: node --test --test-concurrency=1 tests/browser-session.mjs tests/accessibility-browser-cleanup.mjs tests/hub-layout-readiness.test.mjs. All120s budgets, strictly serial on this slot, fresh evidence/TEMP, disk>=12GiB and RAM>=6GiB. NaturalExit=true, Intervened=false, TimedOut=false, Survivors=[], ProfileDirectories=[] for all three. Natural command exit establishes no persistent test worker in the wrapper's observed boundary; it does not certify native cancellation/draining. Node24.14.0, existing tools/dependencies retained; no install.

The first two runs are dirty above base5d426/treee56b2; metadata names exactly the three implementation/test files. That base tree does not identify their evolving dirty contents. The final run is clean aebabd80/tree8153d2a with Dirty=[]. Counts are tests/pass/fail/skipped/cancelled/todo.

| RunID | Phase | UTC start / end | JST start / end | Seconds / budget | Exit | Counts |
| --- | --- | --- | --- | --- | --- | --- |
| 20261006T113829Z-fa93e3ba6f5e4025a8193e787f11934e | edge04-dirty-puremock | 2026-10-06T11:38:29.9980585Z / 2026-10-06T11:38:31.7477013Z | 2026-10-06T20:38:29.9980585+09:00 / 2026-10-06T20:38:31.7477013+09:00 | 1.75 / 120 | 1 | 101/100/1/0/0/0 |
| 20261006T114134Z-86441fed4fc24af0b75297c9eb6b972d | edge04-dirty-correction01-puremock | 2026-10-06T11:41:34.6733941Z / 2026-10-06T11:41:36.3616249Z | 2026-10-06T20:41:34.6733941+09:00 / 2026-10-06T20:41:36.3616249+09:00 | 1.688 / 120 | 0 | 102/102/0/0/0/0 |
| 20261006T114210Z-e6836bb1ad20414493c6896c49d1c859 | edge04-clean-puremock | 2026-10-06T11:42:10.5738874Z / 2026-10-06T11:42:12.3204376Z | 2026-10-06T20:42:10.5738874+09:00 / 2026-10-06T20:42:12.3204376+09:00 | 1.747 / 120 | 0 | 102/102/0/0/0/0 |

## Remaining gates and preservation

Original EDGE02 inventory cleanup FAIL remains intact; EDGE03 only had one non-reproducing inventory PASS, root cause UNKNOWN. F01 third-slot consumer patch was not merged or validated here. F03 late createServer/native rm cancellation and early session-setup lifetime gaps remain outside this package. No extra runtime or safety relaxation was made.

Real browsers, complete npm test, focused browser combination, browser repeats, npm ci/tsc/typecheck/production build, final bundle identity, Rust/Cargo/EXE, Windows native and visual acceptance are all **NOT_RUN** in EDGE04. Unit evidence cannot close those gates. Planning must back up original implementation/report, review this patch alongside the independently delivered third-slot patch, and issue a formal next package before further work. All old logs/profile/save evidence, branches, tools, source and assets are retained; slot1 did not clear caches in this package and dist was not regenerated.

## Retained evidence fingerprints

Complete stdout/stderr/metadata and process inventories remain local in immutable evidence/slot1 Run directories. No raw personal paths, process commands, saves or private supplement are public. SHA-256:

| RunID | stdout.log | stderr.log | metadata.json |
| --- | --- | --- | --- |
| 20261006T113829Z-fa93e3ba6f5e4025a8193e787f11934e | CB0E48DD30BBDF6019B08A5EF6E2B25F3138F7A1D80876FD163CE23210E67DA1 | 7EB70257593DA06F682A3DDDA54A9D260D4FC514F645237F5CA74B08F8DA61A6 | E96DCCE0AA8F6ADE70D46C7EA6DDDE36DF63B38C009A7F66BDB8CB505A5C2418 |
| 20261006T114134Z-86441fed4fc24af0b75297c9eb6b972d | 070323E4E581082A5DEB68F6CEBD08F4FF4CA5E816FDA4739DF0FA5E724C1459 | 7EB70257593DA06F682A3DDDA54A9D260D4FC514F645237F5CA74B08F8DA61A6 | 55818D52A8F579EBB9DFC56387139DAB44B3CE34AE6600FC5918915297FBEDCA |
| 20261006T114210Z-e6836bb1ad20414493c6896c49d1c859 | 4AFEB0725D26ACA59AC5FAB7248996DF0D9D84C55B12077B6E826AA9FBA0E7C5 | 7EB70257593DA06F682A3DDDA54A9D260D4FC514F645237F5CA74B08F8DA61A6 | 1D886EA2F75A1F9B4D171AE6D7DEA2D49FF1ED94883E9813843DC6AADCA25A44 |
