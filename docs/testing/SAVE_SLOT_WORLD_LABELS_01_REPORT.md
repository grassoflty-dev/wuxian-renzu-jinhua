# SAVE-SLOT-WORLD-LABELS-01

## Change and code identity

Save selectors and ordinary writable-slot recovery hints now use the existing
`worldDisplayName` function for nonempty world IDs. The supported names are
归航站、灰巢设施、雾港余烬、钟骨工厂. Empty values retain the original
“未知世界” / “当前世界” fallbacks. Unknown nonempty values retain the existing
name function's behavior. Slot names, IDs, ordering, selection and save policy
are unchanged.

- Fixed base: `48c2dcbb5ef6f69b7b369c3eec7c321aa3cd53e2`.
- Implementation: `66fa8a708f7a03183cbb3054054b76da31b31a25`.
- Temporary branch: `codex/save-slot-world-labels-01-20261006`.
- Production diff: only the two world-label expressions in `apps/web/src/main.ts`.
- Added regression: `apps/web/tests/save-slot-world-labels.mjs`.
- This report is a separate documentation commit after implementation validation.

## Trigger and regression coverage

The preceding browser diagnostic run
`20261006T095520Z-3aa9453769594ab1a074b620ae0b2541` reported 3 tests:
2 PASS / 1 FAIL / 0 SKIP, natural exit 1. At the second New Journey (epoch 4),
the save option text contained `第一段旅程 · grey_hive · 可读写`, matching the
existing rendered-journey assertion's internal-ID prohibition. This report
does not infer that a hidden pause panel was visible at that moment; the same
option-rendering function also supplies player-facing paused save choices.
The browser assertion and fixture IDs were not changed by this package.

The new test extracts and executes the real TypeScript functions
`worldDisplayName`, `renderSlotOptions` and `slotStatusText` in a VM, using
the actual compiled save-slot-policy exports and a minimal DOM surface.
Its 18 cases cover all four names in option text and normal recovery hints,
null/empty fallbacks, unknown IDs, exact slot names and values, unchanged
metadata, valid/invalid/dead/unknown-HP/read-only/recoverable policy, input
order, explicit selection, missing selection and empty lists.

## Executed validation

All commands ran serially within the assigned execution period. Before each
command, available C: space met 12 GiB and available RAM met 6 GiB. Commands
finished within their budgets without timeout or termination.

| Phase | Command | Budget | Result |
| --- | --- | --- | --- |
| Initial diagnostic, dirty base | `npm ci` | 600 s | Natural exit 0 |
| Initial diagnostic, dirty base | `npm run typecheck` | 120 s | Natural exit 0 |
| Initial diagnostic, dirty base | `npx tsc -p tsconfig.json` retry | 120 s | Natural exit 0 |
| Initial diagnostic, dirty base | `node --test tests/save-slot-world-labels.mjs tests/save-slot-policy.mjs tests/death-recovery-ui.mjs` | 120 s | Natural exit 0; 63 PASS / 0 FAIL |
| Clean implementation `66fa8a708f7a03183cbb3054054b76da31b31a25` | `npm run typecheck` | 120 s | Natural exit 0; clean |
| Same clean implementation | `npx tsc -p tsconfig.json` | 120 s | Natural exit 0; clean |
| Same clean implementation | `node --test tests/save-slot-world-labels.mjs tests/save-slot-policy.mjs tests/death-recovery-ui.mjs` | 120 s | Natural exit 0; clean; 63 PASS / 0 FAIL |

Both focused runs have 63 tests: 18 new label regressions, 4 existing save-slot
policy tests and 41 existing death-recovery UI tests. Both have 0 cancelled,
0 skipped and 0 todo. No existing test was removed or weakened.

Raw output and metadata are retained separately from the public repository:

- Initial diagnostic identity:
  `20261006T100624Z-ab583869076f40a9a521c5a04d262a18`.
- Clean implementation validation identity:
  `20261006T100820Z-a44bf3eb1e7043bc9e6d9da15e8ddfde`.

The first compilation's evidence wrapper failed while parsing empty stdout;
that attempt is `CAPTURE_ERROR`, not a demonstrated compilation PASS or FAIL.
Its original command exit/PID were not persisted and are marked unknown.
The successful compilation retry is separate. The wrapper now accepts both
TAP `#` and Node's `ℹ` statistics and rejects missing test counts. It records
known process-exit data before statistics parsing; npm/tsc do not require test
counts. The dirty focused metadata was augmented before preservation feedback
arrived; its prior fields were retained separately from the original tool
output with that provenance. Original stdout/stderr were preserved. A separate
statistics supplement references the original stdout SHA-256
`A123176830965E2CC75FFCFF4CC64CA9820FEE128622606C8087EE63D27F1CF3`.
The clean focused run directly records complete counts.

## Validation boundary

This is a local product-label fix with focused unit regression evidence.
Browser tests, the full Web suite, production Web build, Rust/native builds
and native acceptance were not run for this package. It does not close
EDGE-BROWSER-LIFECYCLE-01 or establish a complete Web/native pass. All package
validation commands have finished naturally; none remains running.
