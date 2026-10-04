# Pivot Presentation Harness 01

This package opens the checked-in `server-rs/release-ui` in a separate Playwright-controlled Microsoft Edge process. It does not attach to Tauri, use CDP, edit `release-ui`, or contact an external host. A local read-only static server serves the real UI files. The only browser-side runtime substitution is a deterministic Tauri IPC test fixture.

The Node test dependency is pinned to `@playwright/test` 1.63.0 and `pngjs` 7.0.0 with an npm lockfile. `.npmrc` disables install scripts so package installation cannot download browser binaries. Run against the already-installed Edge stable channel with:

```powershell
cd tools/presentation-harness
npm ci
npm test
```

If Edge cannot launch, report the original error and keep the scenario checks pending; do not run `playwright install` or download another browser.

`npm run test:preflight` is a synthetic harness-readiness smoke. It verifies Pixi 8.21.0 loading, separate Edge launch, the Hub/World DOM state, canvas dimensions and pixels, page overflow, external/Babylon requests, and frame-time sampling at 1280×720 and 1920×1080. Hub overflow is recorded in the run summary as a finding so screenshots and the World check still complete; World overflow remains a test failure. It captures provisional preflight screenshots and explicitly labels them non-acceptance evidence.

`npm run test:scenarios` is skipped until the exact Scenario Runner fixture and its A-commit/hash metadata are present under `fixtures/`. Once delivered, only that fixture can drive the final Hub, Power/Gate before/after snapshots. These mocked IPC images are technical browser baselines; they do not prove real Tauri behavior or formal art quality. Goldens are kept in `goldens/`; run artifacts and actual screenshots, canvas pixel samples, browser/Pixi versions, image hashes, FPS/frame-time data, and failure diffs belong under `artifacts/acceptance/pivot-presentation-harness-01/`.

After the nominated A commit is available locally, `npm run import:scenario-runner` verifies the exact A blob hashes and copies only the selected initial, closed-gate, power-applied, and passed-gate WorldViews/WorldSnapshots into `fixtures/`. It records the source commit, both original source hashes, the resulting fixture hash, selected JSONL rows, and all derived envelope rules in the metadata file. The importer writes nowhere else.
