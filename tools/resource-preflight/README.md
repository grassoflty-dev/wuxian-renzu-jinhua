# Current Godot resource-byte preflight (read-only)

Status: **SOURCE CANDIDATE / NOT_RUN**. This first block provides resource diagnostics only, before any resource mapping or game launch. It does not change gameplay, build code, saves, rights, admission, or existing files. No private artwork is included.

## Files and fixed scope

- `resource_integrity_preflight.py`: Windows-only Python standard-library CLI. Python 3.10+ availability must be confirmed by local Codex; no installation is authorized by this document.
- `resource-map-v1.json`: public-safe mapping, 31 target PNG paths, 27 corresponding source paths already in the public source tree, and four targets with no public source. The four target checksums and paths are already in public `apps/godot_playable/data/skill_icons.internal-use.json`. No private repository name, source path, revision, or source commit is disclosed here.

Reference source commit: `0d96883c6a0b661b711d5b94bcfdd3acadbf0ddb`, tree `64430cd9d27e2942a4d9cccd4d627dc45043e539`. The manifest is exactly 11373 bytes, SHA256 `5190db6d874183ae965ba639b04412cda2df28761eefab21435aa2503755811b`. The script binds those original bytes before JSON parsing. Changing the manifest requires a separately reviewed identity update.

**The script does not establish checkout Git identity.** Local Codex must separately report the exact checkout/commit and its own changes. A reference SHA written in this manifest is not proof of the current checkout. The diagnostic can run against the ordinary current checkout, provided it is quiescent; it does not require adopting an old protected checkout or reopening a historical task.

## Input layout

`--candidate-root` is the full checkout root. The 27 sources are read directly at `<root>/<publicSource>` (under `assets/`), not from a `public-png` intake folder. Each target is read at `<root>/<target>` (under `apps/godot_playable/assets/`). The original intake staging directory is not a full checkout.

`--manifest` names this exact public-safe manifest, not the private intake mapping. Both CLI path arguments must be absolute ordinary Windows drive paths without trailing separators, dot components, reserved names, or case aliases. The operator must ensure no concurrent writers and acknowledge this with the flag below. No raw personal path or exception traceback is written into the JSON report.

```text
python tools/resource-preflight/resource_integrity_preflight.py --candidate-root "X:\checkout" --manifest "X:\checkout\tools\resource-preflight\resource-map-v1.json" --no-concurrent-writers
```

This command is a proposed local validation entry, **not an execution result or an instruction to launch the game**. Existing repository monitoring/ownership and current user-authorized test handoff govern actual execution.

## Output and exit meanings

- `0 RESOURCE_BYTES_MATCH_ONLY`: all 27 public source checks and all 31 target checks match. This is not game startup, import, gameplay, redistribution or runtime admission acceptance.
- `3 BLOCKED_MISSING_RESOURCES`: one or more source/target paths are absent; no mismatched or unsafe paths were found. Actual missing counts are reported. Four external targets are not assumed missing if they actually exist and match.
- `2 ERROR`: malformed input, altered manifest, path/link/case/hardlink issue, unreadable file, size/hash/Git-blob mismatch, or observed file change. ERROR takes priority over missing resources. No source failure can be hidden by matching targets.

The report lists safe relative paths, measured sizes/hashes when available, controlled reason codes, and totals. For the unmodified referenced public tree, the expected evidence pattern is 27 matching public sources and 31 missing targets, including four without public source. That expected pattern is **NOT_RUN** here and must be checked locally. Nothing is copied or fetched. Existing private target bytes may be hashed locally without granting publication rights; private source metadata locations are never accessed.

## Safety and known limits

The tool performs no copy, write, mkdir, delete, download, subprocess, build or engine invocation. It only prints to stdout. It checks original path syntax before `Path` normalization, rejects symlink/reparse ancestors and nonordinary/hardlinked files, checks exact case spelling for each traversed component, and hashes one read-only file descriptor with before/after identity checks. It scans only requested ancestor directories, not unrelated descendant trees.

This assumes an operator-controlled filesystem with no concurrent writers. Portable standard-library path checks are **not hostile-race-proof** against parent replacement between checks and opens. The flag is an operator acknowledgement, not an OS lock. Do not run while other tasks are writing these paths. A failure requires investigation, not suppression of checks. Extremely strict spelling/metadata rules can reject otherwise usable files; report the reason instead of relaxing them automatically.

## Local Codex validation scope — all NOT_RUN

1. Confirm Python availability/version without installing anything; statically review the three files and fixed manifest identity.
2. On the current fixed source generation, run the read-only preflight and report exact commit, tool version, exit code and sanitized JSON. Confirm no repository/resource changes attributable to the tool.
3. In separately owned disposable fixtures only, exercise missing source; missing target; source hash/size mismatch with all targets matching; target mismatch; four external targets missing/present/mismatched; mixed missing+error (must exit 2); all 58 checks matching (exit 0 bytes-only).
4. Check altered manifest bytes/hash, duplicate keys and invalid counts/path strings at helper level or a test-only harness; the production fixed hash should reject altered files before JSON schema validation. Do not change production constants to bypass binding.
5. Exercise absolute/UNC/relative/dot/dotdot/ADS/reserved/trailing-space/control paths, symlink/reparse ancestors, hardlinked files, exact-case aliases, non-directory ancestors, permission failure and observed file-change conditions. Link fixture creation requires applicable local permissions; unsupported cases remain NOT_RUN, not PASS.
6. Capture no session token or personal absolute path in shared reports. Build, Godot import, launch, return/save/re-entry/Continue tests are outside this block and remain NOT_RUN.
