# Current production Web goldens: pending review

This directory deliberately contains no fabricated baseline images. The old
`tools/presentation-harness/goldens` belongs to the historical release UI and is
neither copied nor modified here.

On the same fixed Windows/Edge/render environment intended for visual CI:

1. Build `apps/web` from a clean committed source tree; run the functional suite.
2. Run `npm run capture:candidate --prefix tools/current-web-acceptance-harness`.
3. Inspect every candidate at 1280×720 and 1920×1080. Candidate generation is not
   a visual pass. Record the exact `visual-environment` attachment, source SHA,
   and verified bundle sidecar SHA in `baseline.json` after review.
4. Copy the reviewed PNGs here using Playwright's exact
   `{name}-{width}x{height}-msedge-win32.png` names; set status to `reviewed`.
5. `npm run test:visual --prefix tools/current-web-acceptance-harness` compares
   existing images with zero changed pixels and refuses automatic updates,
   missing baselines, another platform/browser, or a changed environment.

The default browser command runs functional tests only. No current-Web visual
pass is claimed until the separate reviewed baseline comparison actually passes.
Neither functional nor visual mock-IPC evidence proves Rust gameplay, native
Tauri operation, on-disk save migration, cross-process Continue, or full campaign.
