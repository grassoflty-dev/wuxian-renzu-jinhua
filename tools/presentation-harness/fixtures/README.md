# Scenario Runner handoff fixture

The final fixture is intentionally absent until package A produces and commits its Rust replay JSON. Do not replace it with hand-written snapshots or the inline synthetic preflight fixture.

Expected files:

- `scenario-runner-presentation.json`: byte-for-byte copy of the committed package A presentation replay JSON.
- `scenario-runner-presentation.meta.json`: `{ "sourceCommit": "<40-hex A commit>", "sourcePath": "<repository-relative A output path>", "fixtureSha256": "<sha256 of the exact JSON bytes>" }`.

The replay JSON must normalize actual Rust-produced IPC values into `schema: "pivot-presentation-scenario-fixture/1"` and include `scenarios.hub`, `scenarios.powerBefore`, `scenarios.powerAfter`, `scenarios.gateBefore`, and `scenarios.gateAfter`, each with a real `view`. It must also carry `ipc.formalSnapshot` (WorldSnapshot), `ipc.formalListSaveSlots` (array), `ipc.formalNew` (CommandReceipt), `ipc.formalInteractPower` (FormalInteractionResponse), and `ipc.formalSubmitInput.gateBefore` / `.gateAfter` (CommandReceipt). The ordered input receipts must advance protocol clocks consistently so the real presentation protocol accepts them. The source commit and exact fixture hash are recorded in final evidence. The B-side browser test must never claim that mocked IPC is a real Tauri session.
