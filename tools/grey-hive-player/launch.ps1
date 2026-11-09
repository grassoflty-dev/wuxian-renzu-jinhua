# Source candidate: invoke only after independent acceptance and runtime authorization.
param(
    [Parameter(Mandatory)][string]$PwshExe,
    [Parameter(Mandatory)][string]$NativeExe,
    [Parameter(Mandatory)][string]$NativeSha256,
    [Parameter(Mandatory)][string]$GodotExe,
    [Parameter(Mandatory)][string]$GodotSha256,
    [Parameter(Mandatory)][string]$ProjectDir,
    [Parameter(Mandatory)][string]$ProfileRoot,
    [Parameter(Mandatory)][string]$AttemptDir,
    [int]$GodotWindowWidth = 0,
    [int]$GodotWindowHeight = 0,
    [string]$DiagnosticFirstLaunchUtc,
    [string]$DiagnosticHardDeadlineUtc
)
$ErrorActionPreference = 'Stop'
if (($GodotWindowWidth -ne 0 -or $GodotWindowHeight -ne 0) -and ($GodotWindowWidth -lt 640 -or $GodotWindowWidth -gt 3840 -or $GodotWindowHeight -lt 360 -or $GodotWindowHeight -gt 2160)) { throw 'GH_WINDOW_RESOLUTION' }; $WarningPreference = 'SilentlyContinue'
$VerbosePreference = 'SilentlyContinue'; $DebugPreference = 'SilentlyContinue'
$InformationPreference = 'SilentlyContinue'; $ProgressPreference = 'SilentlyContinue'
$plan = $null; $profile = $null; $ready = $null; $token = $null; $failure = $true
$budget = $null; $stop = $null
$owned = [Collections.Generic.List[object]]::new(); $helpersLoaded = $false
$status = 'GH_PLAYER_PRESTART_REJECTED'
$phase = 'bootstrap'; $primaryError = $null; $finalizerError = $null
$firstGuardCode = 'NOT_OBSERVED'; $firstGuardReason = 'NOT_OBSERVED'
$preCleanupListenerRow = [ordered]@{ State=$null; OwnedPidMatches=$null }
$preCleanupNative = [ordered]@{ HasExited=$null; ExitCode=$null; Seen=$false; State=$null; NativeSampleUnixMs=$null; ReceiveUnixMs=$null; Count=0; Malformed=$false; Age='UNKNOWN'; ReceiveAge='UNKNOWN' }
# Only fixed guard codes and bounded metadata may leave the private launcher.
function ConvertTo-GhDiagnosticError {
    param([System.Management.Automation.ErrorRecord]$Record, [string]$ErrorPhase)
    if ($null -eq $Record) { return $null }
    $type = $Record.Exception.GetType().FullName
    if ($type -cnotmatch '^[A-Za-z.]{1,160}$') { $type = '[redacted]' }
    $id = [string]$Record.FullyQualifiedErrorId
    if ($id -cnotmatch '^(?:GH_[A-Z_]{1,80}|[A-Za-z.]{1,96})$') { $id = '[redacted]' }
    $basename = [IO.Path]::GetFileName($Record.InvocationInfo.ScriptName)
    if ($basename -cnotin @('launch.ps1','paths.ps1','profile.ps1','process.ps1','ready.ps1','lifetime.ps1')) { $basename = '[redacted]' }
    $summary = '[redacted]'
    if ($Record.Exception.Message -cmatch '^GH_[A-Z_]{1,80}$') { $summary = $Record.Exception.Message }
    # Bounded metadata only; failures here must not replace the original diagnostic.
    $deepestType = $null; $deepestHResult = $null; $innerWin32Code = $null
    try {
        $exception = $Record.Exception
        for ($i=0; $i -lt 4 -and $null -ne $exception; $i++) {
            $candidateType = $exception.GetType().FullName
            $deepestType = if ($candidateType -is [string] -and $candidateType -cmatch '^[A-Za-z.]{1,160}$') { $candidateType } else { $null }
            $deepestHResult = [int]$exception.HResult
            if ($i -gt 0 -and $null -eq $innerWin32Code -and $exception -is [ComponentModel.Win32Exception]) { $innerWin32Code = [int]$exception.NativeErrorCode }
            $exception = $exception.InnerException
        }
    } catch { }
    $originBasename = $null; $originLine = $null
    try {
        $stack = $Record.ScriptStackTrace
        if ($stack -is [string] -and $stack.Length -le 4096) {
            $frames = $stack -split "`r?`n", 5
            for ($i=0; $i -lt [Math]::Min(4,$frames.Count); $i++) {
                if ($frames[$i] -match '(?:^|[\\/])(?<name>[A-Za-z0-9_-]+\.ps1): line (?<number>[0-9]{1,7})$') {
                    $name = $Matches['name']; $number = [int]$Matches['number']
                    if ($name -cin @('launch.ps1','paths.ps1','profile.ps1','process.ps1','ready.ps1','lifetime.ps1') -and $number -gt 0 -and $number -le 1000000) {
                        $originBasename=$name; $originLine=$number; break
                    }
                }
            }
        }
    } catch { $originBasename=$null; $originLine=$null }
    return [ordered]@{ phase=$ErrorPhase; exceptionType=$type; errorID=$id; basename=$basename; line=$Record.InvocationInfo.ScriptLineNumber; summary=$summary; deepestExceptionType=$deepestType; deepestHResult=$deepestHResult; innerWin32Code=$innerWin32Code; originBasename=$originBasename; originLine=$originLine }
}
$parameters = @{}; foreach ($key in $PSBoundParameters.Keys) { if ($key -cnotin @('DiagnosticFirstLaunchUtc','DiagnosticHardDeadlineUtc')) { $parameters.Add($key, $PSBoundParameters[$key]) } }
try {
    if (-not $IsWindows -or $PSVersionTable.PSEdition -cne 'Core' -or $PSVersionTable.PSVersion -lt [version]'7.4') { throw 'GH_HOST' }
    # Literal adjacent definitions only. Nothing in the project or PATH is imported.
    if ($PSScriptRoot -notmatch '^[A-Za-z]:[\\/]') { throw 'GH_HELPER_PATH' }
    $bootstrap = [IO.FileInfo]::new([IO.Path]::Combine($PSScriptRoot, 'paths.ps1'))
    if (-not $bootstrap.Exists -or ($bootstrap.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) { throw 'GH_HELPER_PATH' }
    for ($ancestor = $bootstrap.Directory; $null -ne $ancestor; $ancestor = $ancestor.Parent) {
        if (-not $ancestor.Exists -or ($ancestor.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) { throw 'GH_HELPER_PATH' }
    }
    . ($bootstrap.FullName)
    $adjacent = Resolve-GhOrdinaryPath $PSScriptRoot Directory
    foreach ($name in @('profile.ps1','process.ps1','ready.ps1','lifetime.ps1','PlayerWindows.cs')) {
        $null = Resolve-GhOrdinaryPath ($adjacent + '/' + $name) File
    }
    foreach ($name in @('profile.ps1','process.ps1','ready.ps1','lifetime.ps1')) { . ($adjacent + '/' + $name) }
    $helpersLoaded = $true
    $phase = 'diagnostic'
    $diagnosticArguments = Resolve-GhDiagnosticArguments $PSBoundParameters ([Environment]::GetEnvironmentVariable('WUXIAN_GH_DIAGNOSTIC_FIRST_UTC')) ([Environment]::GetEnvironmentVariable('WUXIAN_GH_DIAGNOSTIC_HARD_UTC'))
    $budget = New-GhDiagnosticBudget $diagnosticArguments
    $phase = 'plan'
    $plan = New-GhPlayerPlan $parameters
    $phase = 'interop'
    # Reject an existing type rather than silently reuse an unrelated or older assembly.
    if ($null -ne ('Wuxian.Player.PlayerWindows' -as [type])) { throw 'GH_INTEROP_PRELOADED' }
    $null = Add-Type -Path ($adjacent + '/PlayerWindows.cs') -ErrorAction Stop
    $probe = [Wuxian.Player.PlayerWindows]::OpenReadPinned($plan.PwshExe, $false)
    try { $null = [Wuxian.Player.PlayerWindows]::ReadIdentity($probe) } finally { $probe.Dispose() }
    $phase = 'profile'
    Assert-GhDiagnosticBudget $budget -ForStart
    $profile = Open-GhProfile $plan
    $phase = 'journal'
    Start-GhProfileAttempt $profile $plan
    $stop = New-GhStopContext $plan $profile $budget
    $bytes = [Security.Cryptography.RandomNumberGenerator]::GetBytes(32)
    try { $token = [Convert]::ToHexString($bytes).ToLowerInvariant() }
    finally { [Array]::Clear($bytes, 0, $bytes.Length); $bytes = $null }
    $nativeArguments = @('--session-dir', $plan.SessionDir, '--save-dir', $plan.SaveDir, '--session-token', $token, '--owner-proof-target-pid', [string]$PID)
    $phase = 'native'
    $native = Start-GhChild Native $plan.NativeExe $nativeArguments ([IO.Path]::GetDirectoryName($plan.NativeExe).Replace('\','/')) $plan $owned $budget $stop
    $phase = 'ready'
    $ready = Wait-GhReady $plan $native $token $native.StartupDeadline $stop
    $godotArguments = @('--path', $plan.ProjectDir, '--rendering-method', 'gl_compatibility', '--log-file', $plan.GodotLog)
    if ($GodotWindowWidth -ne 0) { $godotArguments += @('--resolution', ('{0}x{1}' -f $GodotWindowWidth, $GodotWindowHeight)) }
    $godotArguments += @('--', '--v1-ready', $plan.ReadyPath, '--v1-token', $token)
    $phase = 'godot'
    $null = Start-GhChild Godot $plan.GodotExe $godotArguments $plan.ProjectDir $plan $owned $budget $stop
    $phase = 'lifetime'
    Wait-GhPlayerLifetime $owned $budget $stop
    $failure = $null
} catch {
    $primaryError = $_; $failure = $true
    # Snapshot only the first owned-native guard result before cleanup can perform more checks.
    try {
        $firstNative = @($owned | Where-Object { $_.Kind -ceq 'Native' } | Select-Object -First 1)
        if ($firstNative.Count -eq 1) {
            $codes = @('NOT_OBSERVED','UNKNOWN','GH_CHILD_IDENTITY','GH_CHILD_IMAGE','GH_CHILD_IMAGE_UNAVAILABLE','GH_CHILD_DEADLINE','GH_CHILD_CIM','GH_CHILD_CIM_IMAGE','GH_CHILD_ARGV','GH_CHILD_ARGV_IMAGE','GH_CHILD_CIM_CHANGED')
            $reasons = @('NOT_OBSERVED','UNKNOWN','STATE','DEADLINE','EXITED','PID_MISMATCH','BIRTH_MISMATCH')
            $candidateCode = $firstNative[0].FirstGuardCode; $candidateReason = $firstNative[0].FirstGuardReason
            $firstGuardCode = if ($candidateCode -is [string] -and $candidateCode -cin $codes) { $candidateCode } else { 'UNKNOWN' }
            $firstGuardReason = if ($candidateReason -is [string] -and $candidateReason -cin $reasons) { $candidateReason } else { 'UNKNOWN' }
        }
    } catch { $firstGuardCode = 'UNKNOWN'; $firstGuardReason = 'UNKNOWN' }
    # Copy only the already observed row's typed diagnostic values before cleanup.
    try {
        if ($firstNative.Count -eq 1) {
            $property = $firstNative[0].PSObject.Properties['FirstListenerRow']
            if ($null -ne $property -and $null -ne $property.Value) {
                $rowDiagnostic = $property.Value
                if ($rowDiagnostic.State -is [long] -and $rowDiagnostic.State -ge 0 -and $rowDiagnostic.State -le [uint32]::MaxValue) { $preCleanupListenerRow.State = $rowDiagnostic.State }
                if ($rowDiagnostic.OwnedPidMatches -is [bool]) { $preCleanupListenerRow.OwnedPidMatches = $rowDiagnostic.OwnedPidMatches }
            }
        }
    } catch { }
    # Capture this owned process and its bounded stream snapshot before finalization.
    try {
        if ($firstNative.Count -eq 1) {
            $n=$firstNative[0]; $now=[DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds()
            try { if ($null -ne $n.NativeErrorCapture) {
                $snap=$n.NativeErrorCapture.GetStartupSnapshot()
                $preCleanupNative.Seen=[bool]$snap.Seen; $preCleanupNative.Count=[int]$snap.Count; $preCleanupNative.Malformed=[bool]$snap.Malformed
                if ($snap.Seen -and -not $snap.Malformed -and $snap.Count -ge 1 -and $snap.Count -le 96 -and $snap.State -in @(-1,0,1) -and $null -ne $n.BirthTicks) {
                    $birth=[DateTimeOffset]::new([DateTime]::new([long]$n.BirthTicks,[DateTimeKind]::Utc)).ToUnixTimeMilliseconds()
                    $nativeAge=$now-$snap.NativeSampleUnixMs; $receiveAge=$now-$snap.ReceiveUnixMs
                    if ($snap.NativeSampleUnixMs -ge $birth -and $snap.ReceiveUnixMs -ge $snap.NativeSampleUnixMs -and $nativeAge -ge 0 -and $receiveAge -ge 0) {
                        $preCleanupNative.State=[int]$snap.State; $preCleanupNative.NativeSampleUnixMs=[long]$snap.NativeSampleUnixMs; $preCleanupNative.ReceiveUnixMs=[long]$snap.ReceiveUnixMs
                        $preCleanupNative.Age=if($nativeAge -le 5000){'FRESH'}else{'STALE'}; $preCleanupNative.ReceiveAge=if($receiveAge -le 5000){'FRESH'}else{'STALE'}
                    }
                }
            } } catch { }
            try { $exited=$n.Process.HasExited; $preCleanupNative.HasExited=[bool]$exited; if($exited){$preCleanupNative.ExitCode=[int]$n.Process.ExitCode} } catch { }
        }
    } catch { }
}
finally {
    # Exactly one finalization attempt; no second cleanup or fallback gameplay entry.
    if ($helpersLoaded) {
        try { $status = Complete-GhPlayerAttempt $plan $profile $ready $owned $failure $budget $stop }
        catch { $finalizerError = $_; $status = 'GH_PLAYER_UNCERTAIN_CLOSURE' }
    }
    if ($status -cnotin @('GH_PLAYER_NATURAL_CLOSURE','GH_PLAYER_PRESTART_REJECTED','GH_PLAYER_UNCERTAIN_CLOSURE')) { $status = 'GH_PLAYER_UNCERTAIN_CLOSURE' }
    $token = $null; $nativeArguments = $null; $godotArguments = $null
    # Emit from finally so a normal error cannot discard L6's closure disposition.
    [Console]::WriteLine($status)
    # Diagnostics run only after the single cleanup attempt and never expose messages/argv.
    try {
        # These are bounded facts only; never serialize process arguments, ready data or raw streams.
        $nativeRecord = @($owned | Where-Object { $_.Kind -ceq 'Native' } | Select-Object -First 1)
        $nativeCode = 'UNKNOWN'; $stderrComplete = $false; $overallDrained = $false; $nativeOutputSeen = $null; $argsVerified = $false
        if ($nativeRecord.Count -eq 1) {
            $record = $nativeRecord[0]
            $argsVerified = $null -ne $record.CimBirthTicks
            $overallDrained = $record.Drained
            $stderrComplete = $null -ne $record.StderrTask -and $record.StderrTask.IsCompletedSuccessfully
            if ($stderrComplete -and $null -ne $record.NativeErrorCapture) {
                $nativeCode = $record.NativeErrorCapture.SafeCode
                $nativeOutputSeen = $record.NativeErrorCapture.HasOutput
            }
        }
        # Retained record fields only; no Process getter after disposal.
        $postCleanup = [ordered]@{ Exited=$null; ExitCode=$null; Drained=$null; Disposed=$null; NaturalExit=$null; CloseRequested=$null }
        if ($nativeRecord.Count -eq 1) {
            foreach ($name in @('Exited','Drained','Disposed','NaturalExit','CloseRequested')) {
                $property = $record.PSObject.Properties[$name]
                if ($null -ne $property -and $property.Value -is [bool]) { $postCleanup[$name] = $property.Value }
            }
            $property = $record.PSObject.Properties['ExitCode']
            if ($null -ne $property -and $property.Value -is [int]) { $postCleanup.ExitCode = $property.Value }
        }
        $readyNumbersValidated = $false
        if ($null -ne $primaryError -and $primaryError.Exception.Message -cmatch '^GH_READY_INVALID_LISTENER_[A-Z_]+$') { $readyNumbersValidated = $true }
        $diagnostic = [ordered]@{
            phase = $phase; status = $status
            primaryError = (ConvertTo-GhDiagnosticError $primaryError $phase)
            finalizerError = (ConvertTo-GhDiagnosticError $finalizerError 'cleanup')
            nativeStderrCode = $nativeCode; stderrComplete = $stderrComplete; overallDrained = $overallDrained; nativeOutputSeen = $nativeOutputSeen
            argsVerified = $argsVerified; readyNumbersValidated = $readyNumbersValidated
            firstGuardCode = $firstGuardCode; firstGuardReason = $firstGuardReason; listenerRowPreCleanup = $preCleanupListenerRow; nativePreCleanup = $preCleanupNative; nativePostCleanup = $postCleanup
        }
        $json = ConvertTo-Json -InputObject $diagnostic -Depth 4 -Compress
        if ($json.Length -gt 4096) { throw 'GH_DIAGNOSTIC_BOUND' }
        [Console]::Error.WriteLine($json)
    } catch {
        try { [Console]::Error.WriteLine('{"phase":"diagnostic","status":"redacted","primaryError":null,"finalizerError":null}') } catch {}
    }
    $exitCode = switch ($status) {
        'GH_PLAYER_NATURAL_CLOSURE' { 0 }
        'GH_PLAYER_PRESTART_REJECTED' { 64 }
        default { 70 }
    }
    exit $exitCode
}
