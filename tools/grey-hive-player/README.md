# Grey Hive persistent player launcher

Source candidate only. No host execution, userdata binding, save, or new-process Continue acceptance has been observed.
Requires Windows and PowerShell Core 7.4 or newer, not Windows PowerShell 5.1. Select the explicit absolute pwsh.exe; -PwshExe must identify that same running host. Do not invoke through PATH or a console wrapper.
Keep launch.ps1, paths.ps1, profile.ps1, process.ps1, ready.ps1, lifetime.ps1 and PlayerWindows.cs together. C# compilation and interop checks occur only during a later authorized invocation, before profile mutation.
Replace every angle-bracket placeholder below with a qualified value; this is not an executable example:

```powershell
& '<ABSOLUTE_PWSH_EXE>' -NoLogo -NoProfile -NonInteractive -File '<ABSOLUTE_LAUNCH_PS1>' -PwshExe '<SAME_ABSOLUTE_PWSH_EXE>' -NativeExe '<QUALIFIED_ABSOLUTE_NATIVE_EXE>' -NativeSha256 '<NATIVE_SHA256_64_HEX>' -GodotExe '<QUALIFIED_ABSOLUTE_DIRECT_GODOT_EXE>' -GodotSha256 '<GODOT_SHA256_64_HEX>' -ProjectDir '<ABSOLUTE_PROJECT_DIR>' -ProfileRoot '<ABSOLUTE_PERSISTENT_PROFILE_ROOT>' -AttemptDir '<SAME_PROFILE_ROOT>/attempts/<NEW_CANONICAL_UUID>'
```

All paths must be explicit drive-absolute ordinary paths, without reparse ancestors. Choose a profile separate from project, launcher and executable trees; its first use requires a nonexistent leaf or empty directory. Do not allocate a replacement session between launches.
Reuse the same ProfileRoot and persistent session/saves after clean closure; supply a new canonical UUID AttemptDir on every invocation. The launcher holds an exclusive profile lock and writes a durable active journal before starting children.
The normal entry remains project.godot -> scenes/main.tscn -> scripts/ui/v1_main.gd. Native starts once; validated ready precedes the one direct Godot start. No alternate entry, automatic New/Continue/save or extra RPC client is used.
Use the game's existing localized Save and Exit action/confirmation. Healthy play has no time limit. A first process exit allows 30 seconds for its peer; error/cancellation may request the owned Godot window to close once, without choosing its confirmation for you or killing processes.
GH_PLAYER_NATURAL_CLOSURE means only observed natural zero-code exits, completed drains and identity-checked cleanup. GH_PLAYER_PRESTART_REJECTED means no attempt started; GH_PLAYER_UNCERTAIN_CLOSURE means failure or uncertain closure. These codes never attest save success.
An active journal, occupied run/ready state, malformed owner or unknown namespace blocks reuse. Preserve these leftovers and all saves; do not delete, adopt a process, retry, or auto-repair. Obtain a separate authorized investigation before another launch.
Actual userdata binding and save/new-process Continue acceptance remain UNOBSERVED, even after natural closure. Future qualified binaries and explicit runtime authorization are still required; this source delivery performs no runtime acceptance.
