[CmdletBinding()]
param(
	[ValidateSet('PlanOnly','Run')]
	[string]$Mode = 'PlanOnly',
	[string]$ManifestPath,
	[string]$ExpectedManifestSha256
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$script:Task = 'EXEC-HEART-OWNED-OBSERVATION-FIXTURE-20261021-001/r1'
$script:RepoRoot = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../..'))
$script:PrivateBase = 'C:/Users/GWL/.codex/visualizations/2026/10/20/01a159b7-82f9-71c2-8148-c75611de568d/HEART-FLOOR-OBSERVATION-20261021-001/r1'
$script:RequiredPins = [ordered]@{
	'apps/godot_playable/data/scenes/cw_furnace_heart.json' = 'f16f3941767aed71023cd8b9ce75642345a103eabf5c77682e358c96c2e219d5'
	'apps/godot_playable/scripts/world/cw_furnace_heart_floor_material_visual.gd' = '4b6bdb3435f12a4514ed9b94589482b108aee8c14d619878eb760170c49340ba'
	'apps/godot_playable/scripts/bridge/rpc_client.gd' = 'b92452081ea8049a81bca61396e52a8adef894288b8dd30db3c96dca33d0b5ae'
	'apps/godot_playable/scripts/world/renderer.gd' = '25c4b9f7c77f41f620c0f6c030f1d06335f474daf8b0bd650b0fa5c319262041'
	'apps/godot_playable/project.godot' = '9f6c26d26037b0f61bc07f36db40f451f544e7f5542f49ca6d53055f5b5758a3'
	'server-rs/tests/fixtures/earned-cw-furnace-entry-v6.json' = 'd246c77b29c417d94220f7450408655912ea17fefabbecb32fac01c2a15793c2'
}
$script:FixturePath = 'apps/godot_playable/tests/world/heart_floor_owned_observation.gd'
$script:ControllerPath = 'tools/heart-floor-observation/launch.ps1'

function Stop-Run([string]$Code) {
	throw [InvalidOperationException]::new($Code)
}

function Get-BytesSha256([byte[]]$Bytes) {
	$sha = [Security.Cryptography.SHA256]::Create()
	try { return ([Convert]::ToHexString($sha.ComputeHash($Bytes))).ToLowerInvariant() }
	finally { $sha.Dispose() }
}

function Get-FileSha256([string]$Path) {
	if (-not [IO.File]::Exists($Path)) { Stop-Run 'PIN_FILE_MISSING' }
	return (Get-BytesSha256 ([IO.File]::ReadAllBytes($Path)))
}

function Get-FullPath([string]$Path) {
	return [IO.Path]::GetFullPath($Path).TrimEnd([IO.Path]::DirectorySeparatorChar,[IO.Path]::AltDirectorySeparatorChar)
}

function Test-ContainedPath([string]$Root,[string]$Candidate) {
	$base = (Get-FullPath $Root).ToLowerInvariant()
	$path = (Get-FullPath $Candidate).ToLowerInvariant()
	return $path.StartsWith($base + [IO.Path]::DirectorySeparatorChar,[StringComparison]::Ordinal)
}

function Assert-NoReparseAncestors([string]$Path) {
	$current = [IO.DirectoryInfo]::new((Get-FullPath $Path))
	while ($null -ne $current) {
		if (Test-Path -LiteralPath $current.FullName) {
			$item = Get-Item -LiteralPath $current.FullName -Force
			if (($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) { Stop-Run 'REPARSE_ANCESTOR' }
		}
		$current = $current.Parent
	}
}

function Get-GitStatusDigest {
	$rows = @(& git -C $script:RepoRoot status --porcelain=v1 --untracked-files=all)
	if ($LASTEXITCODE -ne 0) { Stop-Run 'GIT_STATUS_READ_FAILED' }
	$text = [string]::Join("`n",[string[]]$rows)
	return (Get-BytesSha256 ([Text.UTF8Encoding]::new($false).GetBytes($text)))
}

function Assert-SourceAndManifest([string]$Path,[string]$ExpectedHash) {
	if ([string]::IsNullOrWhiteSpace($Path) -or -not [IO.Path]::IsPathFullyQualified($Path)) { Stop-Run 'MANIFEST_PATH_REQUIRED' }
	if (Test-ContainedPath $script:RepoRoot $Path) { Stop-Run 'MANIFEST_MUST_BE_EXTERNAL' }
	# ExpectedManifestSha256 is an out-of-band MAIN pin; a manifest label is not permission.
	if ($ExpectedHash -notmatch '^[0-9a-fA-F]{64}$') { Stop-Run 'EXTERNAL_MANIFEST_PIN_REQUIRED' }
	Assert-NoReparseAncestors $Path
	$raw = [IO.File]::ReadAllBytes($Path)
	if ($raw.Length -eq 0 -or $raw.Length -gt 65536 -or (Get-BytesSha256 $raw) -ne $ExpectedHash.ToLowerInvariant()) { Stop-Run 'MANIFEST_HASH_OR_SIZE' }
	$utf8 = [Text.UTF8Encoding]::new($false,$true)
	$manifest = ConvertFrom-Json -InputObject $utf8.GetString($raw) -AsHashtable -ErrorAction Stop
	if ($manifest.schema -ne 'wuxian.heart-owned-observation-run/v1' -or $manifest.task -ne $script:Task) { Stop-Run 'MANIFEST_SCOPE' }
	if ($manifest.runId -notmatch '^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[1-8][0-9a-fA-F]{3}-[89aAbB][0-9a-fA-F]{3}-[0-9a-fA-F]{12}$') { Stop-Run 'RUN_ID_INVALID' }
	if ([string]::IsNullOrWhiteSpace([string]$manifest.resourcePurposeBindingReference) -or [string]::IsNullOrWhiteSpace([string]$manifest.resourcePurposeBindingSha256)) { Stop-Run 'RESOURCE_PURPOSE_BINDING_REQUIRED' }
	if ($manifest.resourcePurposeBindingSha256 -notmatch '^[0-9a-fA-F]{64}$' -or $manifest.resourceAllowlist -isnot [array] -or $manifest.resourceAllowlist.Count -ne 16) { Stop-Run 'RESOURCE_ALLOWLIST_REQUIRED' }
	$resourceSeen = @{}
	foreach ($resource in $manifest.resourceAllowlist) {
		if ($resource -isnot [System.Collections.IDictionary] -or ([string]$resource.path) -notmatch '^res://assets/.+\.png$' -or ([string]$resource.sha256) -notmatch '^[0-9a-fA-F]{64}$' -or $resourceSeen.ContainsKey([string]$resource.path)) { Stop-Run 'RESOURCE_PIN_INVALID' }
	$resourceParts = ([string]$resource.path).Substring(6).Split('/')
		if (([string]$resource.path).Contains('//') -or $resourceParts -contains '..' -or $resourceParts -contains '.') { Stop-Run 'RESOURCE_PATH_INVALID' }
		$relativeAsset = [string]::Join('\',$resourceParts)
		$assetPath = Join-Path (Join-Path $script:RepoRoot 'apps/godot_playable') $relativeAsset
		Assert-NoReparseAncestors $assetPath
		if ((Get-FileSha256 $assetPath) -ne ([string]$resource.sha256).ToLowerInvariant()) { Stop-Run 'RESOURCE_PIN_CHANGED' }
		$resourceSeen[[string]$resource.path] = $true
	}
	if ($manifest.nativeGodotPairs -ne 1 -or $manifest.captureFrameLimit -ne 2 -or $manifest.maxRuntimeSeconds -lt 1 -or $manifest.maxRuntimeSeconds -gt 900 -or $manifest.closureSeconds -ne 300) { Stop-Run 'RUN_LIMITS_INVALID' }
	if ($manifest.fallback -isnot [System.Collections.IDictionary] -or $manifest.fallback.allowTerminateOwned -ne $true -or $manifest.fallback.policy -ne 'EXACT_RETAINED_PROCESS_IDENTITY') { Stop-Run 'OWNED_FALLBACK_NOT_APPROVED' }
	if ($manifest.sourceHead -notmatch '^[0-9a-fA-F]{40}$') { Stop-Run 'SOURCE_HEAD_PIN_REQUIRED' }
	$head = (& git -C $script:RepoRoot rev-parse HEAD).Trim()
	if ($LASTEXITCODE -ne 0 -or $head -ne $manifest.sourceHead) { Stop-Run 'SOURCE_HEAD_MISMATCH' }
	if ($manifest.gitStatusSha256 -notmatch '^[0-9a-fA-F]{64}$' -or (Get-GitStatusDigest) -ne $manifest.gitStatusSha256.ToLowerInvariant()) { Stop-Run 'DIRTY_STATUS_MISMATCH' }
	if ($manifest.sourcePins -isnot [System.Collections.IDictionary] -or $manifest.dirtyInputHashes -isnot [System.Collections.IDictionary]) { Stop-Run 'SOURCE_PIN_MAPS_REQUIRED' }
	foreach ($requiredRuntimePath in @($script:FixturePath,$script:ControllerPath,'apps/godot_playable/scripts/world/scene_catalog.gd')) {
		if (-not $manifest.sourcePins.Contains($requiredRuntimePath)) { Stop-Run 'RUNTIME_SOURCE_PIN_REQUIRED' }
	}
	foreach ($entry in $script:RequiredPins.GetEnumerator()) {
		if (-not $manifest.sourcePins.Contains($entry.Key) -or $manifest.sourcePins[$entry.Key] -ne $entry.Value) { Stop-Run 'REQUIRED_SOURCE_PIN_MISMATCH' }
	}
	foreach ($entry in $manifest.sourcePins.GetEnumerator()) {
		if ([IO.Path]::IsPathFullyQualified([string]$entry.Key) -or ([string]$entry.Key).Split('/') -contains '..' -or ([string]$entry.Key).Contains('\')) { Stop-Run 'SOURCE_PIN_PATH_INVALID' }
		$source = Join-Path $script:RepoRoot ([string]$entry.Key)
		Assert-NoReparseAncestors $source
		if ((Get-FileSha256 $source) -ne ([string]$entry.Value).ToLowerInvariant()) { Stop-Run 'SOURCE_PIN_CHANGED' }
	}
	foreach ($entry in $manifest.dirtyInputHashes.GetEnumerator()) {
		if ([IO.Path]::IsPathFullyQualified([string]$entry.Key) -or ([string]$entry.Key).Split('/') -contains '..' -or ([string]$entry.Key).Contains('\')) { Stop-Run 'DIRTY_PIN_PATH_INVALID' }
		$dirty = Join-Path $script:RepoRoot ([string]$entry.Key)
		Assert-NoReparseAncestors $dirty
		if ((Get-FileSha256 $dirty) -ne ([string]$entry.Value).ToLowerInvariant()) { Stop-Run 'DIRTY_INPUT_CHANGED' }
	}
	foreach ($entry in $script:RequiredPins.GetEnumerator()) {
		if ((Get-FileSha256 (Join-Path $script:RepoRoot $entry.Key)) -ne $entry.Value) { Stop-Run 'FIXED_INPUT_CHANGED' }
	}
	$engine = $manifest.engine
	$native = $manifest.native
	if ($engine -isnot [System.Collections.IDictionary] -or $native -isnot [System.Collections.IDictionary]) { Stop-Run 'EXECUTABLE_PINS_REQUIRED' }
	if ($engine.path -ne 'C:/Users/GWL/WuxianEngineering/tools/godot-4.7.2-20261009/main-resume-extracted-20261009T135429Z_5f3fbf30/Godot_v4.7.2-stable_win64.exe' -or $engine.sha256 -ne 'ab1824f85bfd8e0e4128182c000c4003a3e042245b2967848d089b2a04b22424') { Stop-Run 'GODOT_PIN_MISMATCH' }
	if ($native.path -ne 'C:/Users/GWL/Desktop/wuxian-renzu-jinhua/server-rs/target/release/wuxian_v1_authority.exe' -or $native.sha256 -notmatch '^[0-9a-fA-F]{64}$') { Stop-Run 'NATIVE_PIN_MISMATCH' }
	if ((Get-FileSha256 $engine.path) -ne $engine.sha256.ToLowerInvariant() -or (Get-FileSha256 $native.path) -ne $native.sha256.ToLowerInvariant()) { Stop-Run 'EXECUTABLE_HASH_MISMATCH' }
	$deadline = [DateTimeOffset]::Parse([string]$manifest.deadlineUtc).ToUniversalTime()
	if ($deadline -le [DateTimeOffset]::UtcNow) { Stop-Run 'RUN_DEADLINE_EXPIRED' }
	return $manifest
}

function New-PrivateFile([string]$Path,[byte[]]$Bytes) {
	$stream = [IO.File]::Open($Path,[IO.FileMode]::CreateNew,[IO.FileAccess]::Write,[IO.FileShare]::None)
	try { $stream.Write($Bytes,0,$Bytes.Length); $stream.Flush($true) }
	finally { $stream.Dispose() }
}

function New-RedirectStream([string]$Path) {
	return [IO.File]::Open($Path,[IO.FileMode]::CreateNew,[IO.FileAccess]::Write,[IO.FileShare]::Read)
}

function Start-OwnedProcess([string]$Executable,[string[]]$Arguments,[hashtable]$Environment,[string]$StdoutPath,[string]$StderrPath) {
	$stdout = New-RedirectStream $StdoutPath
	$stderr = New-RedirectStream $StderrPath
	$info = [Diagnostics.ProcessStartInfo]::new()
	$info.FileName = $Executable
	$info.UseShellExecute = $false
	$info.CreateNoWindow = $true
	$info.RedirectStandardOutput = $true
	$info.RedirectStandardError = $true
	foreach ($name in $Environment.Keys) { $info.Environment[$name] = [string]$Environment[$name] }
	foreach ($argument in $Arguments) { $info.ArgumentList.Add([string]$argument) }
	$process = [Diagnostics.Process]::new()
	$process.StartInfo = $info
	if (-not $process.Start()) { $stdout.Dispose(); $stderr.Dispose(); Stop-Run 'OWNED_PROCESS_START_FAILED' }
	$startUtc = $process.StartTime.ToUniversalTime()
	$module = $process.MainModule.FileName
	$stdoutTask = $process.StandardOutput.BaseStream.CopyToAsync($stdout)
	$stderrTask = $process.StandardError.BaseStream.CopyToAsync($stderr)
	return [pscustomobject]@{ Process=$process; StartUtc=$startUtc; Module=$module; Arguments=@($Arguments); Stdout=$stdout; Stderr=$stderr; StdoutTask=$stdoutTask; StderrTask=$stderrTask; StdoutPath=$StdoutPath; StderrPath=$StderrPath; ExitCode=$null; Closed=$false }
}

function Assert-OwnedProcess($Owned) {
	if ($null -eq $Owned -or $null -eq $Owned.Process) { Stop-Run 'OWNED_HANDLE_MISSING' }
	if ($Owned.Process.Id -le 0 -or $Owned.Process.StartTime.ToUniversalTime() -ne $Owned.StartUtc -or $Owned.Process.MainModule.FileName -ne $Owned.Module -or $Owned.Module -ne $Owned.Process.StartInfo.FileName) { Stop-Run 'OWNED_PROCESS_IDENTITY_CHANGED' }
	$current = @($Owned.Process.StartInfo.ArgumentList | ForEach-Object { [string]$_ })
	if ($current.Count -ne $Owned.Arguments.Count) { Stop-Run 'OWNED_ARGUMENT_VECTOR_CHANGED' }
	for ($i=0; $i -lt $current.Count; $i++) { if ($current[$i] -cne $Owned.Arguments[$i]) { Stop-Run 'OWNED_ARGUMENT_VECTOR_CHANGED' } }
}

function Stop-OwnedProcess($Owned,$Manifest) {
	if ($null -eq $Owned -or $Owned.Process.HasExited) { return }
	Assert-OwnedProcess $Owned
	if ($Manifest.fallback.allowTerminateOwned -ne $true -or $Manifest.fallback.policy -ne 'EXACT_RETAINED_PROCESS_IDENTITY') { Stop-Run 'OWNED_FALLBACK_NOT_APPROVED' }
	$null = $Owned.Process.CloseMainWindow()
	if (-not $Owned.Process.WaitForExit(5000)) {
		Assert-OwnedProcess $Owned
		$Owned.Process.Kill()
		if (-not $Owned.Process.WaitForExit(5000)) { Stop-Run 'OWNED_PROCESS_CLOSURE_UNKNOWN' }
	}
	$Owned.ExitCode = $Owned.Process.ExitCode
}

function Complete-OwnedProcess($Owned) {
	if ($null -eq $Owned) { return }
	if (-not $Owned.Process.HasExited) { Stop-Run 'OWNED_PROCESS_STILL_RUNNING' }
	$null = $Owned.StdoutTask.GetAwaiter().GetResult()
	$null = $Owned.StderrTask.GetAwaiter().GetResult()
	$Owned.Stdout.Flush($true)
	$Owned.Stderr.Flush($true)
	$Owned.Stdout.Dispose()
	$Owned.Stderr.Dispose()
	$Owned.ExitCode = $Owned.Process.ExitCode
	$Owned.Closed = $true
}

function Write-JsonPrivate([string]$Path,[object]$Value) {
	$bytes = [Text.UTF8Encoding]::new($false).GetBytes((ConvertTo-Json -InputObject $Value -Depth 8) + "`n")
	New-PrivateFile $Path $bytes
}

function Get-RedactedArguments($Owned,[string]$Token,[string]$RunRoot) {
	$redacted = [Collections.Generic.List[string]]::new()
	foreach ($argument in $Owned.Arguments) {
		if ([string]$argument -ceq $Token) { $redacted.Add('[REDACTED]') }
		else { $redacted.Add(([string]$argument).Replace($RunRoot,'<private>').Replace($script:RepoRoot,'<repo>')) }
	}
	return ,$redacted.ToArray()
}

function Invoke-Run([System.Collections.IDictionary]$Manifest) {
	$runId = ([Guid]::Parse([string]$Manifest.runId)).ToString('D')
	$runRoot = Get-FullPath (Join-Path $script:PrivateBase $runId)
	if (-not (Test-ContainedPath $script:PrivateBase $runRoot) -or (Test-Path -LiteralPath $runRoot)) { Stop-Run 'RUN_NAMESPACE_COLLISION' }
	Assert-NoReparseAncestors $script:PrivateBase
	$session = Join-Path $runRoot 'session'
	$saves = Join-Path $session 'saves'
	$run = Join-Path $session 'run'
	$profileRoot = Join-Path $runRoot 'profile'
	$roaming = Join-Path $profileRoot 'roaming'
	$local = Join-Path $profileRoot 'local'
	$temp = Join-Path $profileRoot 'temp'
	$expectedUser = Join-Path $roaming 'WuxianV1Campaign141'
	$outputs = Join-Path $runRoot 'outputs'
	$directories = @($runRoot,$session,$saves,$run,$profileRoot,$roaming,$local,$temp,$outputs)
	foreach ($directory in $directories) {
		if (-not (Test-ContainedPath $runRoot $directory) -and $directory -ne $runRoot) { Stop-Run 'RUN_PATH_ESCAPE' }
		if (Test-Path -LiteralPath $directory) { Stop-Run 'RUN_PATH_COLLISION' }
		Assert-NoReparseAncestors $directory
	}
	$seedPath = Join-Path $script:RepoRoot 'server-rs/tests/fixtures/earned-cw-furnace-entry-v6.json'
	$seedBytes = [IO.File]::ReadAllBytes($seedPath)
	if ($seedBytes.Length -ne 17801 -or (Get-BytesSha256 $seedBytes) -ne 'd246c77b29c417d94220f7450408655912ea17fefabbecb32fac01c2a15793c2') { Stop-Run 'SEED_IDENTITY_MISMATCH' }
	foreach ($directory in $directories) { $null = [IO.Directory]::CreateDirectory($directory); Assert-NoReparseAncestors $directory }
	$savePath = Join-Path $saves 'formal-save-v6.json'
	New-PrivateFile $savePath $seedBytes
	$now = [DateTimeOffset]::UtcNow
	$deadline = [DateTimeOffset]::Parse([string]$Manifest.deadlineUtc).ToUniversalTime()
	$maxEnd = $now.AddSeconds([Math]::Min([int]$Manifest.maxRuntimeSeconds,900))
	$end = if ($deadline -lt $maxEnd) { $deadline } else { $maxEnd }
	$activeEnd = $end.AddSeconds(-[int]$Manifest.closureSeconds)
	if ($activeEnd -le $now) { Stop-Run 'NO_ACTIVE_RUN_WINDOW' }
	if ([DateTimeOffset]::UtcNow -ge $activeEnd) { Stop-Run 'ACTIVE_RESERVE_REACHED' }
	$tokenBytes = [byte[]]::new(32)
	[Security.Cryptography.RandomNumberGenerator]::Fill($tokenBytes)
	$token = [Convert]::ToHexString($tokenBytes).ToLowerInvariant()
	$engineOut = Join-Path $outputs 'godot.stdout.bin'
	$engineErr = Join-Path $outputs 'godot.stderr.bin'
	$nativeOut = Join-Path $outputs 'native.stdout.bin'
	$nativeErr = Join-Path $outputs 'native.stderr.bin'
	$engineLog = Join-Path $outputs 'godot.engine.log'
	$readyPath = Join-Path $run 'bridge-ready.json'
	$environment = @{ APPDATA=$roaming; LOCALAPPDATA=$local; TEMP=$temp; TMP=$temp }
	$nativeArgs = @('--session-dir',$session,'--save-dir',$saves,'--session-token',$token)
	$godotArgs = @('--path',(Join-Path $script:RepoRoot 'apps/godot_playable'),'--script','res://tests/world/heart_floor_owned_observation.gd','--resolution','1920x1080','--rendering-method','gl_compatibility','--rendering-driver','opengl3','--audio-driver','Dummy','--log-file',$engineLog,'--','--v1-ready',$readyPath,'--v1-token',$token)
	$nativeOwned = $null
	$godotOwned = $null
	$failure = ''
	try {
		$nativeOwned = Start-OwnedProcess $Manifest.native.path $nativeArgs $environment $nativeOut $nativeErr
		$ready = $null
		while ([DateTimeOffset]::UtcNow -lt $activeEnd -and $null -eq $ready) {
			if ($nativeOwned.Process.HasExited) { Stop-Run 'NATIVE_EXIT_BEFORE_READY' }
			if (Test-Path -LiteralPath $readyPath) {
				$readyInfo = Get-Item -LiteralPath $readyPath -Force
				if (($readyInfo.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0 -or $readyInfo.Length -gt 16384) { Stop-Run 'READY_RECORD_UNSAFE' }
				$ready = ConvertFrom-Json -InputObject ([IO.File]::ReadAllText($readyPath)) -AsHashtable -ErrorAction Stop
			}
			else { Start-Sleep -Milliseconds 100 }
		}
		if ($null -eq $ready) { Stop-Run 'READY_RECORD_TIMEOUT' }
		Assert-OwnedProcess $nativeOwned
		if ($ready.schema -ne 'wuxian.godot-v1.ready/v1' -or $ready.protocolVersion -ne 1 -or $ready.host -ne '127.0.0.1' -or $ready.processId -ne $nativeOwned.Process.Id -or $ready.sessionToken -cne $token -or (Get-FullPath $ready.sessionDir) -ne $session -or (Get-FullPath $ready.saveDir) -ne $saves -or $ready.port -lt 1 -or $ready.port -gt 65535) { Stop-Run 'READY_IDENTITY_MISMATCH' }
		$listener = @(Get-NetTCPConnection -State Listen -LocalPort ([int]$ready.port) -ErrorAction Stop | Where-Object { $_.LocalAddress -eq '127.0.0.1' -and $_.OwningProcess -eq $nativeOwned.Process.Id })
		if ($listener.Count -ne 1) { Stop-Run 'LISTENER_OWNERSHIP_MISMATCH' }
		if ($nativeOwned.Process.HasExited) { Stop-Run 'NATIVE_EXIT_BEFORE_CLIENT' }
		if ([DateTimeOffset]::UtcNow -ge $activeEnd) { Stop-Run 'ACTIVE_RESERVE_REACHED' }
		$godotEnvironment = $environment.Clone()
		$godotEnvironment['HEART_OBS_RUN_ROOT'] = $runRoot
		$godotEnvironment['HEART_OBS_OUTPUTS'] = $outputs
		$godotEnvironment['HEART_OBS_EXPECTED_USER_DIR'] = $expectedUser
		$godotEnvironment['HEART_OBS_MANIFEST_SHA256'] = $ExpectedManifestSha256.ToLowerInvariant()
		$godotEnvironment['HEART_OBS_RUN_ID'] = $runId
		$godotOwned = Start-OwnedProcess $Manifest.engine.path $godotArgs $godotEnvironment $engineOut $engineErr
		$remaining = [int][Math]::Max(1,($activeEnd-[DateTimeOffset]::UtcNow).TotalMilliseconds)
		if (-not $godotOwned.Process.WaitForExit($remaining)) { Stop-Run 'GODOT_ACTIVE_WINDOW_TIMEOUT' }
		Complete-OwnedProcess $godotOwned
		if ($godotOwned.ExitCode -ne 0) { Stop-Run 'GODOT_FIXTURE_EXIT_NONZERO' }
		$nativeRemaining = [int][Math]::Max(1,($end-[DateTimeOffset]::UtcNow).TotalMilliseconds)
		if (-not $nativeOwned.Process.WaitForExit($nativeRemaining)) { Stop-Run 'NATIVE_CLOSE_TIMEOUT' }
		Complete-OwnedProcess $nativeOwned
		if ($nativeOwned.ExitCode -ne 0) { Stop-Run 'NATIVE_EXIT_NONZERO' }
		foreach ($name in @('whole-floor.png','entrance-joint.png','context.json','godot.engine.log')) { if (-not [IO.File]::Exists((Join-Path $outputs $name))) { Stop-Run 'EXPECTED_PRIVATE_OUTPUT_MISSING' } }
		$context = ConvertFrom-Json -InputObject ([IO.File]::ReadAllText((Join-Path $outputs 'context.json'))) -AsHashtable -ErrorAction Stop
		if ($context.task -ne $script:Task -or $context.captureCount -ne 2 -or $context.runtimeProfileVerified -ne $true) { Stop-Run 'FIXTURE_CONTEXT_INVALID' }
	}
	catch {
		$failure = ([string]$_.Exception.Message).Replace($token,'[REDACTED]')
		if ($null -ne $godotOwned -and -not $godotOwned.Process.HasExited) { Stop-OwnedProcess $godotOwned $Manifest }
		if ($null -ne $nativeOwned -and -not $nativeOwned.Process.HasExited) { Stop-OwnedProcess $nativeOwned $Manifest }
	}
	finally {
		foreach ($owned in @($godotOwned,$nativeOwned)) {
			if ($null -ne $owned -and $owned.Process.HasExited -and -not $owned.Closed) { Complete-OwnedProcess $owned }
		}
	}
	$report = [ordered]@{
		task=$script:Task; runId=$runId; sourceHead=$Manifest.sourceHead; manifestSha256=$ExpectedManifestSha256.ToLowerInvariant(); outcome=if ($failure) { 'FAIL' } else { 'CAPTURED' }; failureCode=$failure
		godot=if ($null -eq $godotOwned) { $null } else { @{ pid=$godotOwned.Process.Id; startUtc=$godotOwned.StartUtc.ToString('o'); module=$godotOwned.Module; argv=(Get-RedactedArguments $godotOwned $token $runRoot); exitCode=$godotOwned.ExitCode; stdout=(Split-Path -Leaf $engineOut); stderr=(Split-Path -Leaf $engineErr); closed=$godotOwned.Closed } }
		native=if ($null -eq $nativeOwned) { $null } else { @{ pid=$nativeOwned.Process.Id; startUtc=$nativeOwned.StartUtc.ToString('o'); module=$nativeOwned.Module; argv=(Get-RedactedArguments $nativeOwned $token $runRoot); exitCode=$nativeOwned.ExitCode; stdout=(Split-Path -Leaf $nativeOut); stderr=(Split-Path -Leaf $nativeErr); closed=$nativeOwned.Closed } }
		captures=@('whole-floor.png','entrance-joint.png')
	}
	Write-JsonPrivate (Join-Path $outputs 'process.json') $report
	if ($failure) { Stop-Run $failure }
	return $report
}

if ($Mode -eq 'PlanOnly') {
	[pscustomobject]@{ task=$script:Task; mode='PLAN_ONLY'; sourceRoot=$script:RepoRoot; privateRoot=$script:PrivateBase; nativeGodotPairs=1; captureLimit=2; sideEffects=$false } | ConvertTo-Json -Compress
	exit 0
}

if ($PSVersionTable.PSEdition -ne 'Core') { Stop-Run 'POWERSHELL_CORE_REQUIRED' }
$manifestValue = Assert-SourceAndManifest $ManifestPath $ExpectedManifestSha256
$result = Invoke-Run $manifestValue
$result | ConvertTo-Json -Depth 8
exit 0
