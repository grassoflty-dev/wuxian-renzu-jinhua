# Windows native evidence harness

Run this with PowerShell 7 (`pwsh`) on the user's Windows desktop with a visible Tauri build. The script does not build the game or certify its provenance.

```powershell
pwsh -NoProfile -File tools/native-e2e/run.ps1 -ExecutablePath 'E:\path\to\wuxian-horror-ch1.exe' -EvidenceDirectory 'E:\native-evidence\run-01'
```

It records the repository HEAD, executable path/size/SHA-256, process ID, live main-window title and handle, primary display size, requested versus actual outer-window bounds, and a SHA-256 for each screenshot it actually captures. Default requested sizes are 1280×720, 1920×1080 and 2560×1440. A size larger than the current primary display, a failed resize, missing foreground focus or failed screenshot is marked `BLOCKED` or `FAIL`; the script never substitutes a browser screenshot. Evidence goes outside the repository, under a user-selected path or a unique directory beneath `%TEMP%`. A requested evidence path inside the repository is redirected outside it.

For an already running game, use `-ProcessId <pid>`. The harness does not close a process it did not start. A process started with `-ExecutablePath` is closed at the end unless `-KeepOpen` is supplied. It sends no gameplay input. Use [checklist.md](checklist.md) to record actual player path, saves, visual quality and handling by a human. `PASS` in `evidence.json` means only that the requested live-window captures succeeded; it does not mean native gameplay, build identity, three-world route, visual quality, installation or public release passed.

The executable SHA-256 and Git HEAD are separate observations. Until an embedded build identity is independently verified, `buildIdentity.status` remains `UNSATISFIED`, even if screenshots exist. The release build must later identify Git SHA, build/content/save version, asset manifest SHA and scene bundle SHA before provenance can be closed.

Run the negative checks without launching the game:

```powershell
pwsh -NoProfile -File tools/native-e2e/self-test.ps1
```

The self-test creates only a uniquely named temporary directory and verifies missing-executable and invalid-size cases. It does not test real screenshot quality. The script uses Windows desktop APIs and `System.Drawing`; run it in an interactive Windows session with the game unobscured. Windows display limits or desktop security policy may block capture. Do not mark those cases passed without real evidence.

## Real native menu interaction probe

`interaction-run.ps1` probes one menu flow at a time against an existing EXE. It never rebuilds the game. Supply the exact expected EXE SHA-256 and a new, empty evidence directory outside the repository:

```powershell
pwsh -NoProfile -File tools/native-e2e/interaction-run.ps1 -ExecutablePath 'E:\path\to\wuxian-horror-ch1.exe' -ExpectedSha256 '<64-character SHA-256>' -EvidenceDirectory 'E:\native-evidence\unique-run' -Scenario JourneyRepeat
pwsh -NoProfile -File tools/native-e2e/interaction-run.ps1 -ExecutablePath 'E:\path\to\wuxian-horror-ch1.exe' -ExpectedSha256 '<64-character SHA-256>' -EvidenceDirectory 'E:\native-evidence\another-unique-run' -Scenario SaveContinue
```

The probe launches only the verified executable, sets `WUXIAN_FORMAL_SAVE_DIR` and `WUXIAN_NATIVE_DIAGNOSTICS_DIR` to isolated directories beneath that run, checks its PID, module path, hash, window title and foreground HWND, and uses the live Windows accessibility tree to identify a unique enabled control. It stops before sending input if any of those checks fail. Each step has a time limit, records a status and writes `interaction.json`, an event log and available screenshots. It stops only its own game process. Keep the game unobscured and do not interact with another window while it runs.

`PASS` applies only to the specific observed menu flow. `BLOCKED` includes a Windows foreground focus denial or unavailable accessibility control; it is not evidence that the game action failed. The script does not verify EXE embedded bundle identity, sound, combat effects, full world routes, subjective visual quality or release readiness. Preserve each run's evidence, including blocked runs, for diagnosis.

## F3 entry identity observation only

`identity-run.ps1` is a separate, identity-only probe. It does not build the game, call Rust IPC itself, read `bundle-identity.json`, scan source text, or infer EXE identity from a repository SHA. Supply one exact absolute EXE path, that file's expected SHA-256, and a unique absolute evidence directory path that is either new or empty, outside the repository and outside reparse-point parent paths:

```powershell
pwsh -NoProfile -File tools/native-e2e/identity-run.ps1 `
  -ExecutablePath 'E:\builds\wuxian-horror-ch1.exe' `
  -ExpectedSha256 '<64-character EXE SHA-256>' `
  -EvidenceDirectory 'E:\native-evidence\identity-unique-01'
```

The tool hashes the EXE before launch. It refuses any nonempty evidence directory and reserves its evidence JSON and screenshot files without overwriting existing paths. It points the game's formal-save and diagnostics environment variables at new subdirectories beneath this evidence folder and restores the caller's environment afterward. Before sending a real F3 key it independently requires the launched PID's `MainModule` path and SHA-256 to match, a single visible window for that PID, a single visible top-level window with the same exact title, the process `MainWindowHandle` to match that HWND, and that HWND to be the foreground window. It rechecks those observations immediately before input. A failed precondition records `BLOCKED` where evidence can safely be written and sends no F3. It never attaches focus to another process.

After F3, `identity-evidence.json` records the direct UI Automation text exposed by the target window, the observed time/status, process/window/focus facts, screenshot path and SHA-256, and any errors. The screenshot is a visible desktop capture of the verified window. The UI Automation text must stably expose either the F3 entry-byte match sentence or an explicit `UNSATISFIED`; otherwise the entry result is `UNREADABLE` and the run is `BLOCKED_UIA_UNREADABLE`. OCR and source-string searches are not used. A focus or screenshot failure is recorded as `BLOCKED`; it is separate from the entry-byte result.

`OBSERVED_NARROW_ENTRY_PASS` means only that this running WebView visibly reported the narrow entry-byte match and this harness captured its current window. It does not establish full bundle closure, certify that the visible text is truthful, replace EXE SHA evidence, prove installer contents, or grant `native-verified` or public-release eligibility. An explicit F3 `UNSATISFIED` remains unsatisfied. Preserve blocked evidence for review. The harness stops only the PID it started, after rechecking its executable path and SHA; it does not touch other processes or user save locations.

Run its negative checks without starting a game:

```powershell
pwsh -NoProfile -File tools/native-e2e/identity-self-test.ps1
```

The self-test parses both scripts, injects missing focus/multiple-window and other pre-key states through a mock key callback, then checks missing/wrong EXE SHA and existing/repository evidence paths. It starts no game and sends no real F3. A uniquely named scratch directory is preserved under `%TEMP%` so existing files are not deleted by cleanup.
