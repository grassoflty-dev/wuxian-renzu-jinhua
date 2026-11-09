# Grey Hive player-entry recipe
Status: PIN-BOUND DOCUMENT CANDIDATE; NOT IMPLEMENTED, NOT RUN, NOT RELEASED.
Target: docs/releases/grey-hive-player-entry.md. This is maintained recipe text, not a launcher or permission to execute it.
The local integrator must requalify the fixed source identities below; the unavailable local checkout was not substituted with public bytes.

## Scope and release identity
Use the normal chain: project.godot -> scenes/main.tscn -> scripts/ui/v1_main.gd.
Do not pass a test --script, use Dummy audio, or automatically submit New/Continue RPCs.
No product process/profile launcher or maintained export preset is supplied by this document.
Bind the native executable only to a future actually qualified R-B output and its final source cut.
The old native 6f5e077a does not contain the current Continue A fix; it is not qualified for this recipe.
Project-name suffix 139 and userdata suffix 141 are not release-version evidence.
Chinese player-facing localization remains unchanged.

## Fixed contract inputs
Paths are relative to the game checkout; these are supplied planning pins, not a fresh local qualification.
| Path | Bytes | SHA256 |
|---|---:|---|
| apps/godot_playable/project.godot | 556 | 9f6c26d26037b0f61bc07f36db40f451f544e7f5542f49ca6d53055f5b5758a3 |
| apps/godot_playable/scenes/main.tscn | 177 | 88aa656c8cd5e9226ae97b391630756f5558e845fc13a734b89db2de05727941 |
| apps/godot_playable/scripts/bridge/rpc_client.gd | 9729 | e70e9eaa2c68d3e3f050e7c42a27c75bbcd633c10327d10bb5bca7f791d3d129 |
| server-rs/src/godot_main.rs | 10027 | c5636f04574b19a2167593a289b51fea7a2bc0188987e64736fa1ecb1ed963fe |
| apps/godot_playable/scripts/ui/v1_main.gd | 78216 | d038657ba8ab9bc2013ac001e8b2c5388e263ee0909381e8f3bfc1acb395d277 |
| apps/godot_playable/scripts/world/renderer.gd | 153627 | 2646ad9946d327d15d617ee80d31cf425569e1c4cf4e812871cc6bf31fa2dd9f |
| apps/godot_playable/scripts/world/scene_catalog.gd | 2152 | 2d993f8bf905a9d7b4cfeb071c87283b634a39b2dc877a4313225e526cd26efb |
| apps/godot_playable/data/scene_manifest.json | 5584 | 65d3e8b4d1ddecbded33bc4e46abcfc44d909279c321218f44673c2b55c9cff9 |
| apps/godot_playable/data/skill_audio.internal-use.json | 262 | e990acd56e75c46ace2efe5383104aa708e97c098d6df3f5a20f8ec20bf11ea6 |

## Literal argument contract, not an executable example
Every <ABSOLUTE_...> marker below must be replaced with an explicitly qualified absolute path before any future authorized implementation.
Markers are deliberately non-runnable; no personal path, actual token, attempt UUID, or process is allocated here.
The authority accepts only --session-dir, --save-dir and --session-token.
The RPC client accepts only --v1-ready and --v1-token after Godot's -- separator.
```powershell
$entryNative = '<ABSOLUTE_QUALIFIED_R_B_NATIVE_EXE>'
$entryGodot = '<ABSOLUTE_QUALIFIED_GODOT_EXE>'
$entryProject = '<ABSOLUTE_GAME_CHECKOUT>/apps/godot_playable'
$entryProfile = '<ABSOLUTE_PLAYER_OWNED_PROFILE>'
$entrySession = $entryProfile + '/session'
$entryReady = $entrySession + '/run/bridge-ready.json'
$entryGodotLog = '<ABSOLUTE_CURRENT_ATTEMPT_DIRECTORY>/godot.log'
$entryNativeArgv = @(
    '--session-dir', $entrySession,
    '--save-dir', ($entrySession + '/saves'),
    '--session-token', $entryToken
)
$entryGodotArgv = @(
    '--path', $entryProject,
    '--rendering-method', 'gl_compatibility',
    '--log-file', $entryGodotLog,
    '--',
    '--v1-ready', $entryReady,
    '--v1-token', $entryToken
)
```
entryToken must encode 32 CSPRNG random bytes as 64 lowercase hexadecimal characters.
Never print the token or archive an unredacted argument vector containing it.
entryGodotLog belongs to a future actually created attempt-specific directory; this text creates none.

## Required process and persistence behavior
Use ProcessStartInfo with UseShellExecute=false, item-by-item ArgumentList and an explicit WorkingDirectory for each child.
Retain the direct process objects and ownership evidence; hide the native console, show the normal Godot window.
Set APPDATA, LOCALAPPDATA, TEMP and TMP only in both child environments, under the player-owned profile.
Do not change the caller environment or inherit test WUXIAN_GH_* settings.
Expected Godot userdata is <player profile>/roaming/WuxianV1Campaign141; verify its actual binding before claiming isolation.
Do not invent --user-data-dir or fall back to a global/user profile.
Repeated launches must retain the same session/saves: authority requires saveDir == sessionDir/saves.
Creating a fresh empty session on each launch would hide this player's Continue data.
Before launch, obtain an exclusive profile lock and reject reparse paths, out-of-bound paths, occupied ready files and unknown leftover sessions.
Do not adopt another process or unrelated/historical saves; an uncertain leftover blocks launch pending explicit disposition.
Bound ready to 16384 bytes and strict UTF-8; require schema wuxian.godot-v1.ready/v1 and protocol 1.
Verify loopback host, port, retained native PID, exact session/save paths and sessionToken; reject the obsolete token alias.
Wait at most 90 seconds for the initial qualified ready; on failure, do not launch Godot or retry.
Errors enter closure of only the actually owned process identities, never a global/tree kill or process adoption.

## Player exit and safe closure
Use the existing localized Save and Exit action, or the existing confirmation after window-close intent.
In the Save and Exit flow, the UI sends session_close only after a successful save receipt and exits only after closing=true is confirmed.
Closing the client directly is not evidence of a successful save.
Only after both owned processes have actually exited naturally may ready be removed, and only if created by this attempt with unchanged identity.
Retain session/saves across successful launches and closure.
On failure or uncertain closure, preserve leftover state and block another launch; do not automatically delete ready, clear saves or kill unrelated processes.
Exit code 0 is only a closure fact, not proof of saving or new-process Continue acceptance.

## Acceptance limits
Only this new documentation source is proposed; launcher scripts, presets, package directories and executable artifacts are absent.
No parsing, tests, build, launch, profile/save creation, capture, import, export, publication or permission change was performed or authorized by this document.
Existing failed attempts, consumed allocations, protected dirty files and independent M1/M2 ownership remain unchanged.
N3/N4 and packaging retain their own actual authorization, source-qualification and acceptance gates; this recipe supplies no new execution right.
