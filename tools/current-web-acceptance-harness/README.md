# CURRENT-WEB-ACCEPTANCE-HARNESS

Bounded browser acceptance for the **current production `apps/web/dist`**.
This is explicitly `current-web-production-bundle-mocked-ipc` evidence, not
native Tauri or gameplay acceptance. The historical presentation harness and all
its fixtures/goldens remain unchanged.

## Authority and scope

The six-source implementation matrix and UI authority remain in force:

- [Implementation matrix](../../docs/RELEASE_V1_IMPLEMENTATION_MATRIX_2026_10_01.md)
- [PLAN-V1 CI and release gates](../../docs/RELEASE_V1_THREE_WORLDS_PLAN_V1_SOURCE.md)
- [UI-V1 acceptance](../../docs/RELEASE_V1_UI_VISUAL_SPEC_V1_SOURCE.md)
- [Visual specification](../../docs/RELEASE_V1_VISUAL_SPEC_V1_SOURCE.md)
- [Engineering supplement](../../docs/RELEASE_V1_ENGINEERING_SUPPLEMENT_V1_SOURCE.md)
- [Mist Harbor frozen geometry/pump rules](../../docs/MIST_HARBOR_V1_GEOMETRY_PUMP_FREEZE_SOURCE.md)
- [Gameplay details](../../docs/RELEASE_V1_GAMEPLAY_ENGINEERING_SPEC_2026_09_28_SOURCE.md)

The suite tests the built entry, real HTML/CSS Core UI, real Pixi renderer and
bundled scene/assets. Only Tauri IPC is replaced, using `window.isTauri` and
`window.__TAURI_INTERNALS__.invoke`, flattened Snapshot v3 and production receipt
shapes. There is no Vite alias, renderer stub, alternate entry, external asset
fetch, route unlock, generated art, or implicit successful unknown command.
New Journey fixtures start at Return Station `rs_core_room`. An in-memory saved
Grey Hive `gh_entry_maintenance` snapshot tests Continue presentation; it does not establish
that the player traversed the route or that any disk save is valid.

## Commands

Use Node 24 and installed Microsoft Edge on Windows. No alternate-browser
fallback and no successful skip are permitted for browser acceptance.

```powershell
npm ci --prefix apps/web
npm run build --prefix apps/web
npm ci --prefix tools/current-web-acceptance-harness
npm test --prefix tools/current-web-acceptance-harness
npm run verify:bundle --prefix tools/current-web-acceptance-harness
npm run test:browser --prefix tools/current-web-acceptance-harness
```

Build from a clean committed checkout. Run the final production build **after**
`npm test --prefix apps/web`, since that command emits unbundled TypeScript into
`dist`. The server fails closed on a stale HEAD, dirty build identity, missing or
modified payload, mismatched manifest hashes, wrong entry HTML, or missing
sidecar. It reuses the production SHA verifier and serves its verified bytes
from memory at `127.0.0.1:4174`; it cannot serve repository source or the old UI.
The `__current-web-harness__/identity` endpoint and per-test attachments record
the actual build SHA, bundle/entry digests and explicit mock-only evidence kind.

`npm test` runs dependency-free Node contract/server tests on any platform. It
uses Node 24's TypeScript transform to exercise the real protocol validator and
Tauri client without overwriting the production output. Those checks are not a
browser pass.

## Browser coverage

- All eight shared Core UI panels, keyboard navigation/Escape/focus return,
  settings persistence and always-available accessibility
- Only valid authority-granted capability rows; absent inventory/equipment and
  ability-tree data remain unavailable; no invented unlock actions
- New Journey rejection/recovery, duplicate suppression, return and repeated
  entry through real renderer/asset loading
- Valid/legacy/corrupt Continue selection, canceled migration confirmation,
  deferred duplicate suppression, rejected request recovery, timeout lockout
  and late receipt rejection
- Pause acknowledgement and reversal, disabled save before confirmation, Core
  UI Escape without unpausing, read-only overwrite refusal/cancellation, and
  in-memory save/return/Continue
- Live 1280×720, 1920×1080 and 2560×1440 resize, canvas lifecycle, reachable
  controls, no document scroll; common-resolution Hub also needs no inner scroll

Failures retain Playwright action/console/source traces, actual failure screenshots,
and IPC/build/browser/request-timing evidence in
the `functional` subdirectory of the printed evidence directory. By default each
run reserves a new OS temporary directory. Set `CURRENT_WEB_ARTIFACTS_DIR` to an
absolute directory outside the repository to choose another destination. The
runner and Playwright config share this path; in-repository output is rejected
so generated evidence cannot invalidate the clean-source bundle identity.
Functional traces omit automatic DOM/ARIA/screen snapshots and continuous
screenshots. Per-action ARIA capture consumed seconds before actions began in
the 2de Windows run; DOM snapshots also archive all 240+ MB of verified images.
Actual failure screenshots, errors, console and request timings remain enabled.
The production loader still hashes every one of the 114 outputs.

Readiness polls return a primitive monotonic count of real IPC invocations.
New/Continue/release/waits and passive observers cannot increment input counts.
Diagnostics retain exact bounded counters/timestamps, at most 512 recent IPC
records, eight examples per low-rate command, three first-input examples, and 64 frame/long-task
samples. Teardown transmits at most 107 sampled calls, explicitly marked sampled;
use `observations[command].count`, never sample length, for invocation totals.
Passive timing never drives input or changes renderer settings. All existing
readiness/freshness assertions remain in force. Scheduling budgets are explicit below.

## Explicit functional execution profiles

The default `strict` profile retains a 60-second test and 15-second expectation
budget. Visual and candidate runs only accept this default. No application IPC,
60 FPS, visual-baseline or native-gameplay deadline is changed.

The CI `current_web` job explicitly selects
`CURRENT_WEB_EXECUTION_PROFILE=windows-edge-swiftshader-functional`: 180 seconds
per independent flow and 60 seconds per automation action/expectation. It fails
closed unless public WebGL parameters identify SwiftShader on Windows Edge at
DPR 1. It does not force software graphics. Actual OS/CPU/memory/browser and
preflight/real-canvas graphics fingerprints plus the selected budgets are saved.
A software functional pass is **not a native, performance or visual pass**.

[Measured budget rationale and preserved coverage](../../docs/CURRENT_WEB_SOFTWARE_FUNCTIONAL_PROFILE_06.md)
records the dedicated profile and earlier action traces. Eleven independent
flows retain the original assertion expressions, with only two documented fresh
context counter resets. Every entry still requires actual fresh IPC, including
cold 1440p entry, and all three resize resolutions remain. The application New
and Continue deadlines stay 15 seconds; uncertain-Continue feedback still has
its explicit 20-second assertion. There are no retries or successful skips.

## Separate, fixed visual baseline

Playwright is pinned to 1.63.0. Browser runs use Windows, `msedge`, DPR 1, zh-CN,
UTC, dark scheme and reduced motion. Pixel comparison additionally requires the
exact reviewed OS release, browser version and WebGL vendor/renderer fingerprint.
Do not silently refresh Edge, change fonts/render hardware, or adopt another
machine's golden. An environment change requires a fresh reviewed baseline.

`capture:candidate` writes unreviewed current-app screenshots and environment
evidence separately. `test:visual` requires `baseline.json` status `reviewed` and
existing current-app images; until then it fails explicitly with
`E_CURRENT_WEB_VISUAL_BASELINE_PENDING`. See [golden review](goldens/README.md).
It does not modify historical goldens or promote candidates automatically.

## CI insertion (after final web production build)

```yaml
- name: Current production Web harness dependencies
  run: npm ci --prefix tools/current-web-acceptance-harness
- name: Current production Web harness contracts
  run: npm test --prefix tools/current-web-acceptance-harness
- name: Current production Web UI with mocked IPC (Windows Edge)
  env:
    CURRENT_WEB_EXECUTION_PROFILE: windows-edge-swiftshader-functional
    CURRENT_WEB_ARTIFACTS_DIR: ${{ runner.temp }}/current-web-acceptance-harness
  run: npm run test:browser --prefix tools/current-web-acceptance-harness
```

Add the harness lockfile to the npm cache input. Keep the old visual gate and the
native pending notice separate. This insertion deliberately does not claim the
unreviewed current-app golden or native gate has passed.
Upload `${{ runner.temp }}/current-web-acceptance-harness` with `if: always()` to
preserve failure evidence. Do not copy that generated output into the checkout
before later Rust or bundle identity checks.

### Passive accepted-input readiness

Positive New/Continue input-readiness checks use a passive Playwright binding rather than repeated `page.evaluate` round trips during software rendering. The mock emits at most one small immutable record per epoch, only after a real `formal_submit_input` invocation has produced a successful matching receipt and its request qualifies as a fresh valid v2 sample. Observation does not change the mock response, invocation counts, input loop, renderer or authority. Each document receives a fresh browser-generated identity. A browser-side checkpoint is read once before the action while the hub is idle and explicitly arms that document in the Node store; the check then requires accepted input in a strictly newer epoch and a greater actual invocation count. Delayed old-document/old-epoch, wrong-stream, foreign-frame and regressing records cannot satisfy it.

The transport keeps at most one notification in flight and one latest pending record, with no timers or synthetic input. It exposes only disposal and diagnostic counts, not a test-facing publish control. Teardown disables the browser publisher and clears the page-scoped store; pending rejections are handled. The direct authoritative count checks for paused/no-input behavior remain unchanged. Final evidence includes both the raw exact invocation counts and the observed progress/transport diagnostics, so missing or delayed notifications remain auditable.

The original strict60s/15s and explicitly selected software-functional180s/60s budgets, native request limits, all eleven flows, screenshot/visual rules and real-renderer fingerprint requirements remain unchanged. This is mocked-IPC functional evidence only, not native gameplay or performance acceptance.
