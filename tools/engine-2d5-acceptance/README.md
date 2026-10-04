# ENGINE-2D5-CORE-01 acceptance

Run `pwsh -File tools/engine-2d5-acceptance/run.ps1` from the repository root.
It records independent Rust owner ticks without client input, pause/resume,
presentation events, cross-process Save/Continue, save slots, and the strict
TypeScript web contracts. Logs and `verification.json` are written under
`artifacts/acceptance/engine-2d5-core-01/`.

This checks the engine boundary. It does not approve visual assets or claim
that the older placeholder renderer is final art.
