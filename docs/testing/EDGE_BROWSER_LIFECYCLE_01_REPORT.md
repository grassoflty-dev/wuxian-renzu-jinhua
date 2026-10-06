# EDGE-BROWSER-LIFECYCLE-01 — failed draft preservation

Status: **PARTIAL / FAIL / NOT_READY_FOR_CANDIDATE**.

This report preserves the current failed draft under the user's 2026-10-06 requirement to upload the previous round before clearing verified build caches and starting another round. It is not acceptance of the lifecycle repair. The latest preservation decision stops further code changes and test reruns. No candidate integration or push was performed.

## Code identities and scope

- Untouched reproduction: `a5dcc63cddd491d7e656369bd01ce3421f9eba39`, tree `25b4ab7f6ebe888c22b5c835eefd819c33df2024`.
- Draft base: `48c2dcbb5ef6f69b7b369c3eec7c321aa3cd53e2`, tree `5c7d5169f42691fa174011d7766acbeaaf31fc67`. The a5-to-base difference is three documents, not runtime code.
- Preserved implementation draft: `e647a5b133cd3bdca3170ba65d23454a8d811561`; parent is the draft base. Commit message: `wip: preserve edge lifecycle draft [skip ci]`.
- Temporary branch: `codex/edge-browser-lifecycle-01-20261006`; public origin: `https://github.com/grassoflty-dev/wuxian-renzu-jinhua.git`.
- Label dependency implementation `66fa8a708f7a03183cbb3054054b76da31b31a25` and report/head `fa6abb981f4763b335b66b60cf6997e8e6583343` remain **unmerged** into this draft. Prior dependency acceptance was deferred by the preservation decision.
- This report is a separate documentation commit directly after the draft. Its final commit and verified remote SHA are recorded in the delivery to the coordinator.

Eight draft files, plus this report, are the complete base-to-final change scope:

1. `apps/web/tests/accessibility-scale-browser.mjs`
2. `apps/web/tests/hub-lifecycle-regression.mjs`
3. `apps/web/tests/hub-viewport-fit.mjs`
4. `apps/web/tests/inventory-panel-browser.mjs`
5. `apps/web/tests/support/browser-cleanup.mjs`
6. `apps/web/tests/browser-session.mjs`
7. `apps/web/tests/support/browser-process-ownership.mjs`
8. `apps/web/tests/support/browser-session.mjs`
9. `docs/testing/EDGE_BROWSER_LIFECYCLE_01_REPORT.md`

No production source, lockfile, resource or original cleanup-test file was changed. Existing assertions for focus, keyboard, inventory authority, render epochs, layout readiness, viewport limits and HUD labels remain active.

## Draft changes

The four browser suites use a shared Windows Edge session. The draft verifies process creation time, executable, command, unique profile, debug port and descendant lineage rather than treating launcher exit as browser-session exit. CDP connection/request/close operations and server/cleanup operations have finite budgets. Process termination rechecks identity; unknown or live ownership prevents profile removal. Body and cleanup errors remain inspectable together. New helper regressions cover process ownership, PID reuse, CDP failures, retained profiles and disposal failures.

The hub mock gained `formal_has_save` with separate default-save and named-save behavior, an isolated regression, and timeout diagnostics. No HUD assertion was weakened. This work remains a failed, partially validated draft.

## Source acquisition and environment

The initial remote clone failed with curl 56 / abrupt Schannel close / early EOF / invalid index-pack, exit 128. It was not a game failure or successful remote clone. The later explicitly approved local-source route cloned the verified integration source with `--no-hardlinks` into independent normal Git directories. No alternates, shared clone, junction or dependency/build-cache reuse was introduced. Connectivity checks passed, clones were non-shallow and clean, and the reproduction checkout was detached at fixed a5.

Acquisition RunIDs: remote failure `20261006T092651Z-e3809afa6c4249ca91d3c90dab8b4990`; independent reproduction clone `20261006T093343Z-176ffba901134065938908f5b5a7e624`; independent slot1 clone `20261006T093357Z-011f031a91ee43ca845c6c868efb773c`.

Windows; Node 24.14.0; npm 11.9.0; actual installed Edge 154.0.4258.53. Runtime commands used unique external evidence directories and TEMP/TMP boundaries. Full raw logs, profiles and process inventories remain local evidence and are not published. All measured runtime starts exceeded the package's 12 GiB disk / 6 GiB RAM thresholds.

## Evidence interpretation

Baseline commands ran on clean a5. Every `dirty-*` run below ran on evolving uncommitted files above base48; the recorded base/tree do **not** identify those dirty file contents. No dirty PASS is a clean-commit PASS. Older metadata's JST field used an incorrect Z suffix; UTC is authoritative and JST is UTC +09:00. RunID timestamps identify the directory, while exact command times are in metadata.

`N` means test command exited naturally; `I` means external run-specific intervention occurred; `T` means outer deadline expired. A natural test exit followed by orphan cleanup still has I=yes and is not successful natural cleanup. All runs ended with recorded Survivors=[] after any indicated intervention. Test counts are tests/pass/fail/skipped; commands without Node test totals use an em dash. No cancelled or todo tests were reported.

## Complete runtime ledger

| RunID | Command / phase | Counts | Exit | N / I / T | Seconds / budget |
| --- | --- | --- | --- | --- | --- |
| 20261006T093430Z-d36ff4a6c9ac4359a75784d40eb8087f | baseline-npm-ci | — | 0 | yes / no / no | 8.719 / 600 |
| 20261006T093506Z-f00179da03a74d63a69b92ae3cf306dc | baseline-typecheck | — | 0 | yes / no / no | 3.837 / 120 |
| 20261006T093511Z-a255492992154114ac6fa346235f7663 | baseline-build | — | 0 | yes / no / no | 8.18 / 180 |
| 20261006T093520Z-f5e9ef9778f84daea29a37f776d4e3cb | baseline-tsc | — | 0 | yes / no / no | 3.672 / 120 |
| 20261006T093543Z-b365c0a22fe040e48610c0aaa3543f69 | baseline-accessibility | 1/0/1/0 | 1 | yes / yes / no | 9.395 / 180 |
| 20261006T093607Z-58297c390c614bc3956df3b3ea3e59ae | baseline-hub-lifecycle | 2/1/1/0 | 1 | no / yes / yes | 183.112 / 180 |
| 20261006T093921Z-d4c32f55caba4cf9b8344a6d8de40b5d | baseline-hub-viewport | 1/0/1/0 | 1 | no / yes / yes | 181.355 / 180 |
| 20261006T094237Z-998ba774654d4b11a4652eb3bfef9326 | baseline-inventory | 1/0/1/0 | 1 | yes / yes / no | 2.406 / 180 |
| 20261006T094647Z-4cee1ecaf2a74770afd798023a2436f2 | dirty-npm-ci | — | 0 | yes / no / no | 12.903 / 600 |
| 20261006T094658Z-fd0b14b29e5f4ec2a604f92a827315b9 | dirty-cleanup-tests | 9/8/1/0 | 1 | yes / no / no | 1.331 / 120 |
| 20261006T094722Z-5d04a6c329464fedafccd6af5bc5b5b1 | dirty-cleanup-tests-02 | 28/28/0/0 | 0 | yes / no / no | 1.242 / 120 |
| 20261006T094809Z-c86d8ee64fc246aaa06483e6bed15981 | dirty-tsc | — | 0 | yes / no / no | 3.699 / 120 |
| 20261006T094813Z-82669f78581245969a47c30c5cf92aa9 | dirty-accessibility | 1/0/1/0 | 1 | yes / no / no | 5.237 / 180 |
| 20261006T094859Z-d4ea366393f74d8cb7ed66ee93834439 | dirty-accessibility-02 | 1/1/0/0 | 0 | yes / no / no | 7.812 / 180 |
| 20261006T095030Z-e2e593a4f1644d01a12db487b780696b | dirty-cleanup-tests-03 | 28/28/0/0 | 0 | yes / no / no | 1.171 / 120 |
| 20261006T095106Z-a420c9e3f9984fc9a177bee98ab89ee7 | dirty-hub-lifecycle | 2/1/1/0 | 1 | yes / no / no | 23.285 / 180 |
| 20261006T095434Z-8d3df7a0b053472ca03f94c6443e76ce | dirty-hub-lifecycle-diagnostic | 2/1/1/0 | 1 | yes / no / no | 22.512 / 180 |
| 20261006T095520Z-3aa9453769594ab1a074b620ae0b2541 | dirty-hub-lifecycle-fixture-fixed | 3/2/1/0 | 1 | yes / no / no | 7.759 / 180 |
| 20261006T100346Z-8ca27703196e41c6ac712912ec0c428a | dirty-session-regressions | 60/59/1/0 | 1 | yes / no / no | 1.438 / 120 |
| 20261006T101201Z-081a2953a145457098bc8e89923c0777 | dirty-session-regressions-02 | 64/64/0/0 | 0 | yes / no / no | 1.28 / 120 |
| 20261006T101207Z-5fad278ba94c4eafa17258875fb8bdbf | dirty-hub-viewport | 1/1/0/0 | 0 | yes / no / no | 14.044 / 180 |
| 20261006T101228Z-8707e08838ad49b285a449b723c9dcce | dirty-inventory | 1/0/1/0 | 1 | yes / no / no | 7.726 / 180 |

## Failures, body reach and cleanup

### Untouched a5 reproduction

- npm ci, typecheck, production build and test-module compilation exited 0 naturally. Test-module compilation subsequently added/overwrote dist output; this is not a final native production bundle.
- Accessibility failed at connection with `Browser exited 0`; behavior assertions were not reached. Its command exited naturally, then the wrapper cleaned only verified run-owned orphan Edge processes. The baseline profile remains evidence.
- Hub lifecycle had 1/2 PASS, then failed with `EBUSY` on `first_party_sets.db` in cleanup. Viewport failed with the same cleanup error. Both commands remained alive until the 180s outer deadline and required run-owned intervention. The original body error is masked by finally and cannot be recovered from these logs. Both profiles remain evidence.
- Inventory failed at connection with `Browser exited 0`, before behavior assertions. Its command exited naturally, then required run-owned orphan cleanup; the baseline profile remains evidence. Historical Round 4 inventory EBUSY was not reproduced exactly by this local run.
- Later CDP readiness and an independently verified Edge root were observed after launcher exit. That supports the liveness problem but does not prove the endpoint was ready at the original failing connection instant.
- Historical Round 4's 908 tests / 904 PASS / 4 FAIL and intervened hanging teardown remain unchanged. The new baseline is separate evidence, not a rewrite of that result.

### Dirty diagnostics in chronological order

- First cleanup/layout run: 9 tests / 8 PASS / 1 FAIL, cleanup module parse error (`await` in a non-async callback). After correction the same two files passed 28/28; later repeat also passed 28/28. The original 20 cleanup regressions were unchanged.
- The first helper command started before dirty npm ci completed (09:46:58 UTC versus install end 09:47:01 UTC). This was an ordering error; neither run is clean final-gate evidence. The helper did not start a browser or use installed project dependencies.
- Dirty test-module compilation exited 0. No final production build of this draft followed.
- First dirty accessibility failed before body assertions with `E_PROCESS_IDENTITY` during startup identity sampling. Subsequent startup re-reading was added without authorizing incomplete identity. The second dirty accessibility passed 1/1, naturally, with profile released and no external intervention. Helpers were changed afterwards; this is early diagnostic evidence only.
- Hub lifecycle runs at 09:51 and 09:54 failed waiting for pause receipt/save controls, naturally, with profile released and no external intervention. Diagnostic state reported `Unexpected mocked IPC command: formal_has_save`, disabled save and default-only has-save calls. This showed the missing test fixture response rather than proving a production pause failure.
- After fixture correction, hub reached new-journey render epoch 4. Its fixture regression and render probe passed, but the original real-browser HUD internal-ID assertion failed: a named-slot option under the HUD still included `grey_hive`. Counts: 3 tests / 2 PASS / 1 FAIL. Natural exit 1; profile released; no intervention or survivors. The separate label package remains unmerged; this failure remains active.
- First expanded session regression run: 60 tests / 59 PASS / 1 FAIL. An expired-cleanup-budget fixture lacked optional dispose; teardown threw a TypeError and hid the intended AggregateError. Optional disposal and disposal-error collection were corrected, and two sentinel regressions were added without weakening the original assertion. The next helper run passed **64/64**: 36 new session tests, 20 original cleanup tests and 8 layout tests, natural exit 0, no intervention or survivors.
- Latest dirty viewport passed **1/1**, natural exit 0, profile released, no intervention or survivors. Original 2s layout readiness limits remain.
- Latest dirty inventory failed **1 test / 0 PASS / 1 FAIL / 0 SKIP**, natural exit 1, with `E_PROCESS_INCOMPLETE` for a new process. It failed inside `connect()` while constructing the CDP-ready diagnostic through an extra fresh ownership query after WebSocket creation. No connected/page/fixture milestone appeared; **inventory behavior assertions did not execute**. Cleanup released the registered profile, with no external intervention or survivors. A later local process snapshot has a complete identity for that descendant; static evidence only establishes missing fields at that sample, not the final root cause or a production inventory fault. The exception and diagnostic are preserved unchanged.

A read-only draft/source snapshot also exists under RunID `20261006T095359Z-cc8a8cc82a8e48658908cd31619ec6ed`; it is not a test run.

## Final acceptance gates

| Gate on clean preserved draft / final HEAD | Status |
| --- | --- |
| Clean formal implementation validation | NOT_RUN |
| Clean npm ci / typecheck / initial build / test-module compilation sequence | NOT_RUN |
| Complete npm test, natural exit with no intervention | NOT_RUN |
| Required focused combined suites | NOT_RUN |
| Two complete serial repeats of all four real-browser suites | NOT_RUN |
| Final production build after tests | NOT_RUN |
| Rust build/tests | NOT_RUN |
| Native build, launch and real Windows acceptance | NOT_RUN |

No new runtime validation was performed during preservation. Static diff checks passed before the draft commit. This does not establish browser-suite or native acceptance. A later formally assigned round must address the remaining failures and run the missing gates on a defined clean code identity.

## Local cache and preservation boundary

No build cache was deleted by slot1. The latest read-only inventory recorded:

| Isolated clone / cache relative path | Files | Bytes |
| --- | --- | --- |
| Reproduction: apps/web/dist | 260 | 244,623,402 |
| Reproduction: apps/web/node_modules | 4,370 | 142,090,785 |
| Slot1: apps/web/dist | 97 | 731,696 |
| Slot1: apps/web/node_modules | 4,405 | 147,932,367 |

Both clones' server-rs/target directories were absent. Slot1 dist contains test modules and is not a final production bundle. Root cache entries were ordinary directories; recursive reparse points, tracked/ignored state, task dependencies and complete occupancy still require the coordinator's exact cleanup audit. Runtime commands have stopped. Source and lockfiles permit future rebuilding, but this inventory alone does not authorize deleting evidence or certify cleanup completed. Retained baseline profiles, original logs, screenshots, source snapshots and acquisition evidence remain outside build-cache deletion scope. Shared Git storage, historical worktrees and user material were untouched. Next-round work waits for verified remote preservation and documented safe cache cleanup.

## Evidence fingerprints (SHA-256)

Fingerprints refer to private retained originals; only hashes and RunIDs are published here. Metadata includes full command identities, times, resource budgets, natural/intervened status and survivor checks. Full personal paths, process command lines and saves are intentionally omitted from this report.

| RunID | stdout.log SHA-256 | metadata.json SHA-256 |
| --- | --- | --- |
| 20261006T093430Z-d36ff4a6c9ac4359a75784d40eb8087f | B7A72CAF2C8C486663D808C6B918D923B7A08EEBAA5F010384E59FBABD6644EE | 649405B8C8D658C69E411981BBC94755D038ED125E781A5D7A86D238E32FAC97 |
| 20261006T093506Z-f00179da03a74d63a69b92ae3cf306dc | 87F8C34F441C6581D881706A4FFD588E985F4F0D99F61BABCD1201D6107F7395 | 2BFA9C4A1C6D1195B2F538D1D6300753046C8C7AD900852A2406915E54758520 |
| 20261006T093511Z-a255492992154114ac6fa346235f7663 | A792EB527A8D1D5298A5D74700BA8F06D2A1D2CE3F2B7AF4D9F7D51B1AD3C03D | 109C0AD19D92367AB3889B0F2F1411C11A88943CF3FDD6D070B5BCC985C618F5 |
| 20261006T093520Z-f5e9ef9778f84daea29a37f776d4e3cb | 7EB70257593DA06F682A3DDDA54A9D260D4FC514F645237F5CA74B08F8DA61A6 | 93283F5ACF0809A06BFB2AF8BD99BA261C03CDC579357D66008DD96182CA4ED1 |
| 20261006T093543Z-b365c0a22fe040e48610c0aaa3543f69 | E57C83CF287A702C75EA2CE72D49075587D5337DF2C1F9C7CF4A1C7556CA1DE6 | 86BE5F7A8694331B5DBABC83745DC88A402E119310D095E79580AA990FE64E7E |
| 20261006T093607Z-58297c390c614bc3956df3b3ea3e59ae | DF956B040328BD745CBED6F51FC01D24E8580BC9DC65FF1E0A40AC5970E56F68 | DD2BE2EE846C4CD960C0199F6B51D173CD8581EBE6656E24CB1FD5701F851A6A |
| 20261006T093921Z-d4c32f55caba4cf9b8344a6d8de40b5d | 360B83D1B89FB158660863F7562E8743AC78DD574970C29B05D29D3588F981C4 | DD0CA8E735182F73E468B8FEB334265F10C4942982917BC0B6E62ACE519DEBFB |
| 20261006T094237Z-998ba774654d4b11a4652eb3bfef9326 | 3F0423F68F79B05779DEE3E7ED0164B158CE6CBD614A738DB811731D103044AD | B02BBCD2992B9009C8A42A236926C5FD34FD0B918B579DADED92252E0474A216 |
| 20261006T094647Z-4cee1ecaf2a74770afd798023a2436f2 | E9BE30D866D4F590D33788ECF67BCD8352575068F112C13385C4DC7FA82B0114 | 4F2E4DA4C796D210AFF4C754847CCAC1C8248C00DCFD1B5BFE04849AA998F68A |
| 20261006T094658Z-fd0b14b29e5f4ec2a604f92a827315b9 | F3944499A99E773ACC0F36412CBA1421791E56B4E747B303C934BE08B0F285D8 | F8AB6079B9A30D6182D47236B579932067A8785FB0F69D47A2566B36CC68C279 |
| 20261006T094722Z-5d04a6c329464fedafccd6af5bc5b5b1 | FAA51A2D25B272B26BE9E88852B41EBDACF9086100F9B0A49FD3FB0C68628713 | 53477B8FD768402A454D7794E7B629A807D7C8474BE8A74D07C1691D8DD26856 |
| 20261006T094809Z-c86d8ee64fc246aaa06483e6bed15981 | 7EB70257593DA06F682A3DDDA54A9D260D4FC514F645237F5CA74B08F8DA61A6 | 4F0AC2D4BAD10963E780A5ECDC8A789B5011E96AC8AC58AD1F3EB98624B94D08 |
| 20261006T094813Z-82669f78581245969a47c30c5cf92aa9 | 3EF295BFE7A16708D77F8AE792AAFF7884873953C2864CC28B913077AA25EA54 | C512FC2063EB05849CFFC3F6507D2F506384C98878DCDF4F976815C136999D98 |
| 20261006T094859Z-d4ea366393f74d8cb7ed66ee93834439 | 6D942B830A6B2E05AEF811BC9B8EA3458AB490A7E83B545CCED997F241244104 | 14B5858CF895F7A64BAC0390319B59F378977A0923CC14961846923EFCE214A1 |
| 20261006T095030Z-e2e593a4f1644d01a12db487b780696b | E4FD38AD315B4865CE3A10924DF58AAD7DAD9CFEFF50F62FBB89DFF4469F317C | 28C2579C9E4319E2EECC883929010B255C8AD1C827685207DA9A20B25C18B670 |
| 20261006T095106Z-a420c9e3f9984fc9a177bee98ab89ee7 | 15BF80DDB8274BA587A421AFAC0795ABB0875755EC560760D1EC2CBA18FAD587 | 4BEAECE4483843B153EEA3F6CBCFE7E33F52D6F3AEBE628F4E0696DF08005F44 |
| 20261006T095434Z-8d3df7a0b053472ca03f94c6443e76ce | C953C65A65D67FBC911C5633D898236A6EBA3361937150CF49D4520ADCB1924C | B018D2CB68ACAEFCAAA345195425F7E400F0913A0FEB7799727E1AAE9755FD4A |
| 20261006T095520Z-3aa9453769594ab1a074b620ae0b2541 | 34D4015B8F54102AF75E432865AC62DC0EA18CD94622AFB53CB3F53EFDA318CA | FDEAFD8CCBB0E50C9D95FA133DDBA3F6E97656AFCB7BFD0782AE1774F4EE8474 |
| 20261006T100346Z-8ca27703196e41c6ac712912ec0c428a | E7754D3CEBC07C1DDA87027AB0FF39468A6FBA2F56A3668049EEF9292A8CD2D2 | B1026EC6AAE23DF91E5B04B043AEDECDB7D2EAC0FE498ADC9C95F50413421790 |
| 20261006T101201Z-081a2953a145457098bc8e89923c0777 | 2CC22441904437EA9AFC66FE73E88F28DB28E4DF9C400D39938849B2FBAEE6A9 | DBFDAA94BFEED66B07E34DC0E631C065B5989042DEA0409E32159545ACF44071 |
| 20261006T101207Z-5fad278ba94c4eafa17258875fb8bdbf | 3FE45E302E7BCA6529FFB2D503DD05E25BC3D0EF82D5EF5E9E116DF312282A22 | 55EB582955A04337A3CC93A86D2664F1B38589ABBC7E7747E7D42F999B4A4E64 |
| 20261006T101228Z-8707e08838ad49b285a449b723c9dcce | E0464D949598845F4A6C35051380633DF00F1B10ACAD835E5C140FB17DEEE2D5 | 91137B71DD0A3A4AFCC244510B0174AC56C3CD274B82542F53B2B50907B3CEDA |

Initial failed remote clone log SHA-256: 1A1BCDDEA7D74CC2F23DE587789C30BF8DFCAD57B6500BAD57533C08CE8E4190.
