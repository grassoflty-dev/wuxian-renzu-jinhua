# EDGE-BROWSER-LIFECYCLE-02

Status: **PARTIAL / FAIL / NOT_READY_FOR_CANDIDATE**.

The package's dirty minimum verification reached real inventory connection and behavior setup, then failed in cleanup identity verification. Further runtime validation and source changes stopped immediately. This is failed implementation preservation, not clean Web acceptance or candidate integration.

## Sources and code identity

Current AGENTS/coordination rules, the release source register and ENG/UI authority, the full UI source, fixed public engineering handoff, the original EDGE01 failure report, existing helper source and reviewed label source/report were read. The later user instruction to continue repairs applies after the coordinator's recorded previous-round remote preservation and seven incremental-cache cleanup. Private handoff modules were not copied or published.

- Fixed starting HEAD: `cbb92ad771448cca64d2ca585f094479636309a1`.
- Its original draft parent: `e647a5b133cd3bdca3170ba65d23454a8d811561`, then base `48c2dcbb5ef6f69b7b369c3eec7c321aa3cd53e2`.
- Reviewed label implementation: `66fa8a708f7a03183cbb3054054b76da31b31a25`; label report/head: `fa6abb981f4763b335b66b60cf6997e8e6583343`. Fetch verified this exact SHA and both required ancestors.
- Authorized dependency merge: `81ceebf363722b427596feb9c4e6d2172c16d299`, parents `cbb92ad771448cca64d2ca585f094479636309a1` and `fa6abb981f4763b335b66b60cf6997e8e6583343`. No conflict. This is the comparison baseline for autonomous changes.
- Preserved implementation: `04671aba6e76be039589ef8c40d90cbc2fc724ec`, parent `81ceebf363722b427596feb9c4e6d2172c16d299`; message `wip: preserve bounded edge connection implementation [skip ci]`.
- New temporary branch: `codex/edge-browser-lifecycle-02-20261006`, on the existing independent clone. Origin remains `https://github.com/grassoflty-dev/wuxian-renzu-jinhua.git`. Previous refs and backups remain intact.
- This report is committed separately after implementation. Final report HEAD and the exact verified temporary remote SHA are delivered to the coordinator; no main/candidate push or native stage was performed.

All six runtime runs below were on evolving dirty files above merge `81ceebf`, tree `8f1a5cabe8d89311d3c9c8a1252bda16a85ed143`. **None is validation on clean implementation `04671aba` or final report HEAD.** The original EDGE01 report and all its 22 runs, untouched a5 reproduction and historical Round4 908/904 PASS/4 FAIL plus external intervention remain unchanged.

## Complete change scope

Inherited EDGE01 changes are documented in its unchanged report. The dependency merge changes only:

- `apps/web/src/main.ts`: the two previously reviewed label expressions.
- `apps/web/tests/save-slot-world-labels.mjs`: reviewed label regressions.
- `docs/testing/SAVE_SLOT_WORLD_LABELS_01_REPORT.md`: original dependency report.

Autonomous changes relative to the dependency merge are exactly:

- `apps/web/tests/support/browser-session.mjs`: shared bounded connection path and cancellable discovery/handshake/fresh identity verification; registered socket and combined errors.
- `apps/web/tests/support/browser-process-ownership.mjs`: remaining-time/cancellation parameters and a port inspection that returns the same freshly verified owned snapshot; ownership admission and termination checks remain unchanged.
- `apps/web/tests/browser-session.mjs`: 17 new behavioral regressions through the real exported connection path.
- `apps/web/package.json`: only scripts.test changed to `tsc -p tsconfig.json && node --test --test-concurrency=1 tests/*.mjs`.
- `docs/testing/EDGE_BROWSER_LIFECYCLE_02_REPORT.md`: this report.

The four real-browser suite files, browser-cleanup, original cleanup/layout regressions and all lockfiles were not edited in this package. No autonomous production changes, resources, Rust, workflow or private files were added.

## Implementation and regression boundary

Real session.connect calls connectOwnedBrowser. Its default total deadline is 15,000 ms across page discovery, socket handshake and the fresh owned-process/port sampling used for the CDP-ready diagnostic. Only E_PROCESS_INCOMPLETE, E_PROCESS_QUERY and E_PROCESS_QUERY_TIMEOUT are retried while time remains. A ready diagnostic uses a fresh complete owned snapshot with a verified listener root. Hard identity, port, URL and transport failures remain failures. Late discovery or ownership results cannot return ready. Query/fetch/poll cancellation stays within the helper-created operations; no global process termination was added.

A connected socket is registered before later sampling can throw. Sampling failure closes it, and socket close failures are combined with the original failure. Session-owned process/profile cleanup continues through the existing wrapper. Unknown ownership still retains the profile and fails.

The 17 new regressions cover incomplete-then-complete sampling; persistent incomplete/query/query-timeout causes; immediate identity/listener/URL failures; actual registry PID reuse and foreign listener; wrong discovered port before allocation; expired phase deadlines; late discovered/owned results; transport close/error during sampling; original failure plus socket-close failure; and cancellation reaching an in-flight sample. These execute the same exported path that real session.connect invokes. The 36 prior session, 20 prior cleanup and 8 layout regressions remain, giving 81 tests in this focused run. This focused PASS does not satisfy the full acceptance gate.

## Executed dirty validation

Windows, Node 24.14.0, npm 11.9.0, installed Edge 154.0.4258.53. Commands were serial and each received a new immutable evidence RunID/TEMP boundary. Resource checks before every command passed 12 GiB disk / 6 GiB RAM. Metadata contains UTC and correctly offset JST (+09:00). No run timed out or required external intervention; all ended with recorded Survivors=[]. Counts are tests/pass/fail/skipped/cancelled/todo.

| RunID | Phase | UTC start / end | Seconds / budget | Exit | Counts |
| --- | --- | --- | --- | --- | --- |
| 20261006T111126Z-ea252c561bd34d0493c88229aa4cb0bd | edge02-dirty-npm-ci | 2026-10-06T11:11:26.3304588Z / 2026-10-06T11:11:41.4581919Z | 15.128 / 600 | 0 | — |
| 20261006T111146Z-2685b4e607b44a31b22ec1e1b36c9271 | edge02-dirty-typecheck | 2026-10-06T11:11:46.5667026Z / 2026-10-06T11:11:50.0347995Z | 3.468 / 120 | 0 | — |
| 20261006T111155Z-ffe02378e47f4f6c9ffc2534f244a20e | edge02-dirty-build | 2026-10-06T11:11:55.4382095Z / 2026-10-06T11:12:02.4514812Z | 7.013 / 180 | 0 | — |
| 20261006T111208Z-f284cad5c9c245db93ba71cd26c38b60 | edge02-dirty-tsc | 2026-10-06T11:12:08.5007740Z / 2026-10-06T11:12:13.1143241Z | 4.614 / 120 | 0 | — |
| 20261006T111218Z-36984468fd0949dda5e94b33cd1ca60e | edge02-dirty-helper | 2026-10-06T11:12:18.3400057Z / 2026-10-06T11:12:19.3925647Z | 1.053 / 120 | 0 | 81/81/0/0/0/0 |
| 20261006T111224Z-3f056479f3d44e8ea1e45cd69c3d347b | edge02-dirty-inventory | 2026-10-06T11:12:24.8396346Z / 2026-10-06T11:12:30.9144674Z | 6.075 / 180 | 1 | 1/0/1/0/0/0 |

Commands, in order: npm ci; npm run typecheck; npm run build; npm exec -- tsc -p tsconfig.json (the npm exec equivalent of npx); node --test --test-concurrency=1 with browser-session/accessibility-browser-cleanup/hub-layout-readiness; node --test --test-concurrency=1 with inventory-panel-browser. No dependent command started before the preceding wrapper completed.

### Latest real inventory failure

Run `20261006T111224Z-3f056479f3d44e8ea1e45cd69c3d347b` emitted CDP-ready with 13 verified owned-process identities, then `inventory-browser: connected`, `page ready` and `fixture ready`. The test body returned without a separately reported behavior/assertion error; the resulting error is solely `AggregateError: Browser cleanup failed`. This establishes body reach beyond the prior pre-body failure, but **the test overall remains FAIL**.

Its member error says `Owned PID was reused or changed`, thrown by ownership refresh during hasExited / waitForExit after Browser.close in cleanup. The raw Node inspection does not print that member's code or the differing identity fields. No particular PID change, actual reuse, field disappearance or final root cause can be concluded from that message alone. No retry or weakening of the hard identity checks was applied to erase it. There was no external intervention, and the command naturally exited 1 after approximately 6.075 seconds with 1 test / 0 PASS / 1 FAIL / 0 SKIP / 0 CANCELLED / 0 TODO. The registered profile boundary remains retained as local failure evidence; no profile-released milestone appeared. Survivors=[] does not turn a failed ownership cleanup into PASS.

This is not evidence of an inventory product defect. The earlier run's missing fresh-sample fields and this cleanup failure are distinct observations. The reviewed label dependency has been merged, but hub real-browser validation was not reached in this package and is not claimed fixed by this report.

## Not executed after failure

| Required clean implementation stage | Status |
| --- | --- |
| Clean npm ci/typecheck/initial build | NOT_RUN |
| Complete npm test, all tests/*.mjs serially | NOT_RUN |
| Clean focused accessibility/hub/world-renderer-demand combination | NOT_RUN |
| Clean individual viewport/inventory | NOT_RUN |
| Two complete four-browser repeats | NOT_RUN |
| Final production build after all tests | NOT_RUN |
| Final bundle-identity exact clean implementation SHA / resource hashes | NOT_RUN |
| Rust build/tests, candidate EXE and native acceptance | NOT_RUN |
| Visual acceptance / new screenshots | NOT_RUN |

The dirty initial production build exited 0, then tsc wrote test modules into dist. It is not the required final production bundle, does not prove the preserved implementation's identity, and was not used for native work. The implementation/report are saved as failed WIP under the package's failed-DoD preservation rule. No clean verification rerun was attempted after the failure.

## Evidence fingerprints

Full raw logs, process command lines, personal paths, profile contents and saves are local only. Below are SHA-256 fingerprints of the retained originals.

| RunID | stdout.log | stderr.log | metadata.json |
| --- | --- | --- | --- |
| 20261006T111126Z-ea252c561bd34d0493c88229aa4cb0bd | B06F9DB7150EBE5B04B9F1A3FA37597247BF8DE9054F61DC37EF8380C58B96A3 | 375FC32EE77B4307E78BC9400B5D71BA74515FFE5EC544E8FE6EE333A2C6F473 | 1F01AD98051C7B1B929343B34C128410A6272BEC58322D53F35514BDB4130A1E |
| 20261006T111146Z-2685b4e607b44a31b22ec1e1b36c9271 | 87F8C34F441C6581D881706A4FFD588E985F4F0D99F61BABCD1201D6107F7395 | 7EB70257593DA06F682A3DDDA54A9D260D4FC514F645237F5CA74B08F8DA61A6 | 0F10D73511BDB48FE031CDF1BB48FFC472953E3589E9628CA2B10986BAC2DD52 |
| 20261006T111155Z-ffe02378e47f4f6c9ffc2534f244a20e | 7B9ED6245B643C8E953CB6A54517DF3F24D7A17C2F1CAD9C2391B9799C20F6F1 | 796961E4D2D019A6195333A50B382AA7D8E6476921B182261C7156D174DBF5AE | D4030601EDCA063BF144747ECD50CCDB8F1654ED750E3F4F4FD8AC0681007298 |
| 20261006T111208Z-f284cad5c9c245db93ba71cd26c38b60 | 7EB70257593DA06F682A3DDDA54A9D260D4FC514F645237F5CA74B08F8DA61A6 | 7EB70257593DA06F682A3DDDA54A9D260D4FC514F645237F5CA74B08F8DA61A6 | B06A36EA88C58A83EE69AEEB4B06E02EF9D4D50DE9D627DF5A25B0D9785806B6 |
| 20261006T111218Z-36984468fd0949dda5e94b33cd1ca60e | 6991BA57C61BD4C809087F775E84CCD1FBC3DC5804AD039C6CB7FD583431EBAC | 7EB70257593DA06F682A3DDDA54A9D260D4FC514F645237F5CA74B08F8DA61A6 | 41F060CAD745961AAB00BB152F9CD0A5DF94FFC7C10820352BB50F21D7B57FC3 |
| 20261006T111224Z-3f056479f3d44e8ea1e45cd69c3d347b | B0B31BCC0100A66C15F9D05049AA206C3493963BA988B1CD121648C209BA1ECE | 7EB70257593DA06F682A3DDDA54A9D260D4FC514F645237F5CA74B08F8DA61A6 | 44362776BF3F373227C68CFEE6393E95701733F8A1099650903591EADE8643FA |

## Preservation and cache handoff

All validation commands have stopped. No cache was deleted by slot1 in this package. The original reproduction dist/dependencies and the active slot1 dist/dependencies remain. Slot1 dist now includes dirty test-module output, not a final native-ready bundle. Both isolated clones' server-rs/target remain absent. The latest inventory retained failure profile and all previous logs/profiles are evidence, not cleanup candidates.

After exact remote implementation/report preservation, the coordinator/planning review must determine the next repair package and audit precise stopped, reproducible build-cache paths and dependencies before deletion. No old targets, shared Git storage, worktrees, source, tools, EXEs, gen directories, resources, saves or evidence were touched. The previous seven incremental-cache cleanup record remains separate; this report does not claim that any new cache cleanup or candidate acceptance has occurred.