# Bounded command evidence recorder

Run with Node 24 (tested on Linux v24.19.0):

```
node tools/testing/record-command.mjs request.json
node --test --test-isolation=none --test-reporter=tap tools/testing/record-command.test.mjs
```

`request.json` contains `command` (exact executable/argv array; no shell), `cwd`,
`sourceSha` (40-character Git commit), `inputs` (nonempty array of relative `path`
and expected `sha256`), a new `outputDir` whose parent exists, and `timeoutMs`.
For example, command may be `["node", "--check", "tests/hub-lifecycle-regression.mjs"]`.
Use a resolved executable path when its exact identity matters. The caller must
verify that the input manifest was obtained from the stated source commit; the
recorder verifies bytes against that manifest, not Git ancestry or a full tree.
Include the complete transitive local input set for the chosen test. Do not
claim a clean repository from a manifest-only run.

Each run owns a new directory and refuses existing directories. This is
overwrite protection, not filesystem immutability; review hashes when copying. `metadata.json`
starts with UNKNOWN/null status before execution. Child exit/timing are saved
before post-command identity checks and log inspection. stdout.log/stderr.log
preserve raw bytes, including zero-byte output, and metadata gives byte counts
and SHA-256 hashes. No TAP/JSON parsing can erase command metadata. Start/end
are recorder observations around spawnSync, not measured OS process birth/exit
instants. EXIT_ZERO is command outcome only, not proof that tests ran or passed.
Review expected test counts, failures, cancellation, skip and todo separately.

Only use with bounded trusted syntax/unit/mock commands that do not start
servers, browsers, worker trees or persistent descendants. For these Node 24
mock tests use --test-isolation=none to avoid default test-worker processes.
A timed-out command is never cleanup or resource-release evidence. A timeout
terminates the direct child only and does not establish descendant shutdown.
Logs are streamed to disk without an output-size cap; choose commands with
bounded output and provision sufficient disk. Never put secrets in argv or
known-sensitive commands/logs in evidence destined for public publication.
Do not publish local absolute paths or raw logs without review. Metadata records
no environment variables. If the recorder itself is interrupted, an existing
UNKNOWN/partial run is evidence of incomplete recording and must not become
PASS. Durable preservation through host/power failure is not guaranteed.

This helper does not repair the unavailable PowerShell wrapper described by
VITE_CREATION_RESOURCE_RETENTION_01_REPORT.md and does not recover its lost
historical exit code or timings. That old run remains UNKNOWN/NOT_VERIFIED.
