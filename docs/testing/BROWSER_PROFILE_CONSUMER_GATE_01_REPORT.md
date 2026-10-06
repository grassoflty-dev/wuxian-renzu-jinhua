# BROWSER-PROFILE-CONSUMER-GATE-01

Status: **IMPLEMENTED / UNIT_TESTED_CANDIDATE**. This closes the registered
profile-consumer F01 branch in pure mock regression coverage. It is not real
browser, complete Web, native, or candidate integration acceptance.

## Fixed identity and scope

- Base: `0232cba8f8c0ced234219a24097cdce13c68571a` (fixed EDGE02 report).
- Verified parent implementation: `04671aba6e76be039589ef8c40d90cbc2fc724ec`;
  its parent merge: `81ceebf363722b427596feb9c4e6d2172c16d299`, with parents
  `cbb92ad771448cca64d2ca585f094479636309a1` and
  `fa6abb981f4763b335b66b60cf6997e8e6583343`.
- Implementation: `3a662d83a0197a0288e8f20d459a116a1475fa59`, direct child of
  base; commit message includes `[skip ci]`. It contains only the helper and
  new test listed below. Normal push and ls-remote verified this exact SHA.
- Branch: `codex/browser-profile-consumer-gate-01-20261006`, public origin
  `https://github.com/grassoflty-dev/wuxian-renzu-jinhua.git`.
- This report is the separate second `[skip ci]` commit. Its full report SHA
  and final verified remote SHA are recorded in the coordinator delivery and
  local delivery manifest after committing; the clean test SHA above is the
  implementation SHA, not the later report SHA.

The existing independent public clone was reused after confirming clean
detached `a5dcc63cddd491d7e656369bd01ce3421f9eba39`, complete Git history,
no object alternates or reparse ancestors/Git directory, exact origin, no
matching active project process, and absent local/remote task branch. Fetch
verified exact base and parent chain before creating the task branch. Its
historical directory name does not identify the new source as an a5 reproduction.

Read completely: global AGENTS and current collaboration rules; old release
source register and engineering supplement authority; fixed public handoff at
`48c2dcbb5ef6f69b7b369c3eec7c321aa3cd53e2`; fixed EDGE02 report; static audits
`20261006T111114Z-1c97cc5b0c3742c2ab9e8dd3546a4a04` and
`20261006T111821Z-18d3beee157642fd80547b171b81b2c0`. No private supplement
was copied or published. Slot1/2 files and current03 branch were not integrated.

Complete autonomous tracked changes relative to base:

1. `apps/web/tests/support/browser-cleanup.mjs`
2. `apps/web/tests/browser-profile-consumer-gate.mjs` (new)
3. `docs/testing/BROWSER_PROFILE_CONSUMER_GATE_01_REPORT.md` (new)

Original `apps/web/tests/accessibility-browser-cleanup.mjs` remains byte-identical
to base, Git blob `6a39d6e08a049112cab482f5b7266a644a1f3c37`. No original
assertions, expectations, skips, production files, dependencies or workflows
were changed. Diff checks and staged whitelist review passed.

## Behavior and regression coverage

Cleanup records successful or failed closure for each registered otherResource.
Each resource is considered in order even after an earlier rejection. Original
error objects and causes remain in the existing AggregateError. Profile
verification and removal begin only when browser exit is confirmed and every
registered close succeeded. Rejection, explicit bounded close timeout or an
exhausted close budget leaves the consumer unconfirmed and blocks even calling
the profile verifier/removal helper. A late completion of timed-out close work
cannot change the failed record or start deletion later in this cleanup call.

The original exact-child/ownership termination, CDP/socket/HTTP cleanup,
dispose attempt, profile verification, removal retry and combined body/cleanup
error behavior remain. An empty consumer list preserves the original workflow;
ordinary socket/server errors alone do not acquire the consumer gate.
`bounded`, `removeOwnedProfile`, and CDP interfaces were not changed.

Ten new tests execute the real cleanup helper and wrapper: rejected consumer
identity/cause and disposal; failed first/successful second consumer; multiple
close errors plus dispose error; bounded timeout with late completion; fully
exhausted budget; budget exhausted between consumers; strict all-closes before
verification/removal order; no-consumer flow; unconfirmed browser exit; and
body assertion plus consumer cleanup failure. All removal/verification/browser
state/resource operations are injected mocks. Fake timers reset after each
timer test; the deferred timed-out operation is settled and drained in its test.
No processes, servers, ports, browsers or real profiles are created or removed
by these tests. The original 20 tests also pass unchanged.

## Actual runs

Node `24.14.0`; both commands run sequentially in apps/web:

```text
node --test --test-concurrency=1 tests/accessibility-browser-cleanup.mjs tests/browser-profile-consumer-gate.mjs
```

| Phase / RunID | Tested identity | UTC start / end | JST start / end (+09:00) | Natural seconds / budget | Exit | Tests/pass/fail/skipped/cancelled/todo |
| --- | --- | --- | --- | --- | --- | --- |
| dirty / `20261006T113117Z-d2d86d0541804d5481b5c2e602a5bb67` | base0232 plus precisely two dirty helper/test files | 11:31:17.4635113 / 11:31:17.7185613 | 20:31:17.4635113 / 20:31:17.7185613 | 0.2550500 / 60 | 0 | 30/30/0/0/0/0 |
| clean / `20261006T113155Z-e0c28ae0b201496c99e1e2ce0f3d6874` | clean implementation `3a662d83a0197a0288e8f20d459a116a1475fa59` | 11:31:55.7345321 / 11:31:55.9878833 | 20:31:55.7345321 / 20:31:55.9878833 | 0.2533512 / 60 | 0 | 30/30/0/0/0/0 |

All timestamps are 2026-10-06. Both runs exited naturally with no intervention,
no SKIP, no cancelled/todo and no failures. The clean run had empty complete
porcelain before and after and unchanged implementation SHA. No clean rerun or
expanded test command was needed. Resource gates exceeded 12 GiB disk/6 GiB RAM
before both runs (about 57.4 GiB disk and 12.5 GiB RAM).

Local raw stdout/stderr/exit/metadata are preserved independently per RunID.
Public fingerprints of unchanged original logs:

| Run | stdout SHA-256 | stderr SHA-256 |
| --- | --- | --- |
| dirty | `19A7BC70795DF399AAA0513C53A906AFDB8B986F4334A876DB06495BF0089073` | `E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855` |
| clean | `B544A22D0B2C23679A66F9B3AED2185FFC98BF70F59D89F40180C8FE1CAF6C2B` | `E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855` |

An initial local evidence wrapper failed before Node launch because multiple
Node application paths were passed as an array to FilePath. No test started.
The wrapper was corrected to select one path. After the dirty PASS, its count
parser was corrected to accept Node's default spec summary prefix as well as
TAP; initially null metadata counts were populated from the unchanged raw
30/30 output without rerunning tests. Both wrapper issues and original console
output are retained locally; neither is a hidden regression failure.

## Limitations and handoff

F01 protection covers **registered** otherResources only and trusts successful
close completion according to their existing contracts. It does not prove real
Vite/worker shutdown or solve an unregistered late Vite allocation. F02 handshake
socket retention/confirmed disposal and F03 late create/listen/uncancellable
native rm remain unresolved. Once native removal has started, this change does
not cancel or drain it. Unknown/active browser ownership still blocks deletion;
no latest PID failure identity or root cause is inferred here.

Complete Web, real browser suites/repeats, typecheck/TS compilation, production
build, bundle identity, Rust, EXE/native and visual acceptance: **NOT_RUN**.
Old ignored dist/node_modules were retained; old dist was neither read nor
reused as a build identity. No tools installed, caches/branches/worktrees/
profiles/evidence cleaned, other slots integrated, main/candidate/tag/Release
pushed, Actions dispatched or settings changed.

Return implementation and report commits to the main coordinator for planning
backup and review, followed by authorized integration with03 and later identity
repairs. Other blockers continue to prevent complete candidate acceptance.
