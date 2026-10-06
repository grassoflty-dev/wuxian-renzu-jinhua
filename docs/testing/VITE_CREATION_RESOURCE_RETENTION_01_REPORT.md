# VITE-CREATION-RESOURCE-RETENTION-01

Status: **PARTIAL / FAIL / NOT_READY_FOR_CANDIDATE**. Source and tests are
preserved as unverified WIP. The local evidence wrapper failed after launching
the syntax-only check and before persisting its exit code/metadata. Required
mock tests and clean verification did not run. No unit-tested or real-browser
acceptance is claimed.

## Identity and sources

- Fixed base/report: `2cefffac3e4efdd0e14700ad51ba1d1ab0a3cfdf`.
- Parent F01 implementation: `3a662d83a0197a0288e8f20d459a116a1475fa59`;
  its parent: `0232cba8f8c0ced234219a24097cdce13c68571a`.
- Preserved source WIP: `82d89f091686fd46e8d6b698f8c7f449be008f5e`, direct
  child of base, `[skip ci]`. Its normal push and ls-remote matched exactly.
- Unique public temporary branch:
  `codex/vite-creation-resource-retention-01-20261006`.
- Origin: `https://github.com/grassoflty-dev/wuxian-renzu-jinhua.git`.
- This report is a separate second `[skip ci]` commit. Its full report SHA and
  final verified remote SHA are delivered in the local manifest/coordinator
  handoff after commit; they are distinct from source WIP and base identities.

The existing independent clone was reused. Preflight verified exact clean
base, current F01 branch, origin, parent chain, absent local/remote new branch,
and F01 temp/report backup at2cefff and implementation backup at3a662d. No new
clone/worktree, merge, reset, stash, clean or force push was performed.
Available disk `61591851008` bytes and RAM `13565161472` bytes at initial check
exceeded 12 GiB / 6 GiB; the syntax wrapper checked the same minimum again
before launch. These are resource checks, not process ownership queries.

Fully read current global AGENTS/collaboration rule, old release source register
and ENG authority, fixed public handoff at48c2dc, both fixed02 static audits
(`20261006T111114Z-1c97cc5b0c3742c2ab9e8dd3546a4a04` and
`20261006T111821Z-18d3beee157642fd80547b171b81b2c0`), F01 report, fixed local
hub/browser-session/cleanup interfaces. No private supplement was copied.
No current04 or other-slot dirty implementation was read or integrated.

## Complete source scope and intended behavior

Exactly four tracked files differ from base including this report:

1. `apps/web/tests/support/vite-session-resource.mjs` (new,94 lines).
2. `apps/web/tests/vite-session-resource.mjs` (new,249 lines).
3. `apps/web/tests/hub-lifecycle-regression.mjs` (4 lines added/3 removed).
4. `docs/testing/VITE_CREATION_RESOURCE_RETENTION_01_REPORT.md` (new).

The new adapter registers synchronously before the factory is released. It
retains creation, listen settlement and one shared close task; diagnostic state
is inspectable. Ready defaults to15000ms and rejects late results/close requests.
Ready/listen failure requests closure, without cancelling the underlying Vite
operation. Close waits for creation and started listen settlement, then uses
the actual server close. Listen and close errors are combined through existing
withBrowserCleanup. A retained rejection observer prevents automatically
requested close failures becoming unhandled while the same rejecting close
task/error remains available to cleanup and later callers.

Hub zero-context diff is confined to base lines2,190,207 (new207-208): replace
bounded import with the new helper; wrap the unchanged createServer factory;
register through `resource => session.trackVite(resource)` before allocation;
await adapter.ready; pass that same adapter to unchanged session.listenVite.
Existing session trackVite/listenVite double registration remains, intentionally
handled by the adapter's shared close promise so actual server.close executes
once. No session method/API was edited or monkeypatched. Worker capture and
closeVite ownership interfaces remain unchanged.

Base hub lines3-189,191-206 and208-end are unchanged (the latter becomes209-end).
Config/root/cacheDir/plugins/host/port/strictPort and the15000ms creation budget
retain original values. IPC fixture/state/renderer probes, all selectors and
behavior assertions remain byte-identical outside those three diff ranges:
rendered epochs `[2,4,6]`, world `grey_hive`, scene `gh_entry`, HUD ID prohibition,
new/save/return/continue counts3/1/2/1 and saved world `grey_hive` remain.
No real hub assertion was executed in this package.

The protected cleanup/session/ownership and original tests have no diff from
base. Original cleanup blob remains `6a39d6e08a049112cab482f5b7266a644a1f3c37`;
F01 consumer10 test blob remains `454542d3aee8de8dad0d76a406ab895f28a2623e`.
Staged source scope and diff --check passed before WIP preservation.

## Authored regression coverage — NOT_RUN

Twelve new tests are authored, not counted as executed PASS: register-first and
successful close/removal; failed registration/no allocation; creation timeout,
bounded cleanup and late server closure; late factory rejection with body and
cleanup errors; rejection after cleanup finishes; listen timeout/late success;
listen+close double error identities/cause; concurrent/serial close plus double
registration; failed close unchanged by second registration; injected clock
deadline; close while creation/listen wait is pending; and pending actual close
blocking profile removal. They import the real closeVite, F01 cleanup and
withBrowserCleanup paths with injected server/resource/removal mocks.
Deferred operations are intended to settle/drain under test control and fake
timers reset. This has not yet been dynamically verified.

Original20+consumer10 were not rerun here. Their previous F01 package's30/30
evidence remains historical evidence on that implementation only.

## Actual failed validation evidence

Only attempted command, from apps/web:

```text
node --check tests/hub-lifecycle-regression.mjs
```

Dirty RunID: `20261006T114818Z-4bdea4311aab4c63953f02cc46166cfb`.
Identity at attempt: base2cefff plus the three source/helper/test dirty files.
Budget30s; wrapper exit1. stdout/stderr both0 bytes, each SHA-256
`E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855`.

The wrapper reached Start-Process/WaitForExit/Refresh, then line25 Regex.Match
received null from empty syntax stdout. That exception prevented metadata.json
and exit.txt writes. Exact Node exit/natural flag/process start/end/elapsed
were held only in the failed shell and are **UNKNOWN / NOT_VERIFIED**, not
invented as PASS from empty logs. Local failure record preserves this gap.
Run directory creation was `2026-10-06T11:48:18.3324579Z` /
`2026-10-06T20:48:18.3324579+09:00`; this filesystem timestamp is not substituted
for command execution timing. Counts/noSKIP are not applicable to syntax-only.

This is a local evidence wrapper failure, not an observed Node syntax error or
game regression. Source writes stopped and no command was rerun after failure.
Required dirty three-file mock run: **NOT_RUN**. Clean syntax and clean
three-file mock run: **NOT_RUN**. No acceptance statistics are claimed.

## Limits and handoff

Registration is not settlement or shutdown. The adapter has no internal Vite
handles before factory return and cannot cancel factory/listen internals.
If creation/listen/actual close remains pending, existing closeVite must time
out and the F01 gate must retain profile/FAIL; that intended composition awaits
the authored behavioral tests. A late result should remain reachable and be
closed through the retained task, without restarting ready/listen or initiating
late profile removal. Actual process/worker shutdown is not proven here.

F03 native rm continuing after timeout is untouched. Existing dynamic browser
cleanup identity failure root cause remains **UNKNOWN**. F02 and04 changes are
outside this package. Browser/fullWeb/final production bundle/TS compile/Rust/
EXE/native/visual acceptance are **NOT_RUN**. Old ignored dist/node_modules
were preserved; dist was not read or used as a new identity. No production,
locks, dependencies, resources, cache/profile/evidence cleanup, OS process
query, port/server/browser startup or other-slot modification occurred.

Return unverified source and accurate FAIL report to main for planner backup,
review and a minimal follow-up decision about the empty-stdout wrapper and
required validation. This failed package does not authorize candidate push,
integration with04, additional reruns, agents or cleanup. Only its unique temp
ref was pushed; main/candidate/Release/tag/Actions/settings/old refs are unchanged.
