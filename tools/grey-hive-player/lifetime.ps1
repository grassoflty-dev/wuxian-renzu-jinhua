# Optional diagnostic anchors are supplied, never minted by either player entry.
function Resolve-GhDiagnosticArguments {
    param([Parameter(Mandatory)]$Parameters, [AllowNull()][string]$EnvironmentFirst, [AllowNull()][string]$EnvironmentHard)
    $result = @{}
    foreach ($name in @('DiagnosticFirstLaunchUtc','DiagnosticHardDeadlineUtc')) {
        $environment = if ($name -ceq 'DiagnosticFirstLaunchUtc') { $EnvironmentFirst } else { $EnvironmentHard }
        if ($Parameters.ContainsKey($name)) {
            $value = $Parameters[$name]
            if ($value -isnot [string] -or [string]::IsNullOrWhiteSpace($value)) { throw 'GH_DIAGNOSTIC_ARGUMENT' }
            if (-not [string]::IsNullOrEmpty($environment) -and $value -cne $environment) { throw 'GH_DIAGNOSTIC_CONFLICT' }
            $result[$name] = $value
        } elseif (-not [string]::IsNullOrEmpty($environment)) { $result[$name] = $environment }
    }
    if ($result.Count -ne 0 -and $result.Count -ne 2) { throw 'GH_DIAGNOSTIC_PAIR' }
    return $result
}
function ConvertFrom-GhDiagnosticUtc {
    param([Parameter(Mandatory)][string]$Value)
    $format = "yyyy-MM-dd'T'HH:mm:ss.fffffff'Z'"
    $parsed = [DateTimeOffset]::MinValue
    if ($Value -cnotmatch '^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{7}Z$' -or
        -not [DateTimeOffset]::TryParseExact($Value, $format, [Globalization.CultureInfo]::InvariantCulture, [Globalization.DateTimeStyles]::AssumeUniversal, [ref]$parsed) -or
        $parsed.ToUniversalTime().ToString($format, [Globalization.CultureInfo]::InvariantCulture) -cne $Value) { throw 'GH_DIAGNOSTIC_UTC' }
    return $parsed
}
function Get-GhDiagnosticUtcNow { return [DateTimeOffset]::UtcNow }
function New-GhDiagnosticBudget {
    param([Parameter(Mandatory)]$Arguments)
    if ($Arguments.Count -eq 0) { return $null }
    if ($Arguments.Count -ne 2 -or -not $Arguments.ContainsKey('DiagnosticFirstLaunchUtc') -or -not $Arguments.ContainsKey('DiagnosticHardDeadlineUtc')) { throw 'GH_DIAGNOSTIC_PAIR' }
    $first = ConvertFrom-GhDiagnosticUtc $Arguments.DiagnosticFirstLaunchUtc
    $hard = ConvertFrom-GhDiagnosticUtc $Arguments.DiagnosticHardDeadlineUtc
    $seconds = ($hard - $first).TotalSeconds; $now = Get-GhDiagnosticUtcNow
    if ($seconds -le 600 -or $seconds -gt 7200 -or $first -gt $now -or $hard.AddSeconds(-600) -le $now) { throw 'GH_DIAGNOSTIC_INTERVAL' }
    $budget = [pscustomobject]@{ FirstLaunchUtc=$Arguments.DiagnosticFirstLaunchUtc; HardDeadlineUtc=$Arguments.DiagnosticHardDeadlineUtc
        HardUtc=$hard; OperationalUtc=$hard.AddSeconds(-600); HardTick=[long]::MaxValue; OperationalTick=[long]::MaxValue; Failed=$false }
    Update-GhDiagnosticBudget $budget
    return $budget
}
function Update-GhDiagnosticBudget {
    param([AllowNull()]$Budget)
    if ($null -eq $Budget) { return }
    $now = Get-GhDiagnosticUtcNow; $tick = [Diagnostics.Stopwatch]::GetTimestamp()
    foreach ($name in @('Hard','Operational')) {
        $seconds = [Math]::Max(0.0, [Math]::Min(7200.0, ($Budget.($name+'Utc') - $now).TotalSeconds))
        $candidate = $tick + [long][Math]::Floor($seconds * [Diagnostics.Stopwatch]::Frequency)
        $Budget.($name+'Tick') = [Math]::Min($Budget.($name+'Tick'), $candidate)
    }
}
function Assert-GhDiagnosticBudget {
    param([AllowNull()]$Budget, [switch]$ForStart)
    if ($null -eq $Budget) { return }
    Update-GhDiagnosticBudget $Budget
    if ($Budget.Failed -or [Diagnostics.Stopwatch]::GetTimestamp() -ge $(if ($ForStart) { $Budget.OperationalTick } else { $Budget.HardTick })) {
        $Budget.Failed = $true
        throw 'GH_DIAGNOSTIC_TIMEOUT'
    }
}
function Get-GhDiagnosticDeadline {
    param([AllowNull()]$Budget, [Parameter(Mandatory)][long]$Proposed, [switch]$Operational)
    if ($null -eq $Budget) { return $Proposed }
    Update-GhDiagnosticBudget $Budget
    return [Math]::Min($Proposed, $(if ($Operational) { $Budget.OperationalTick } else { $Budget.HardTick }))
}
# Stop files are attempt-local, immutable/exclusive-created, and carry no execution authority.
function Read-GhStopJson {
    param([string]$Path, [string[]]$Keys)
    $path = Resolve-GhOrdinaryPath $Path File
    $stream = [IO.File]::Open($path, [IO.FileMode]::Open, [IO.FileAccess]::Read, [IO.FileShare]::Read)
    try { return Read-GhJsonObject $stream 1024 $Keys } finally { $stream.Dispose() }
}
function Write-GhStopJson {
    param([string]$Path, $Value)
    $null = Resolve-GhOrdinaryPath $Path File $true
    $bytes = [Text.UTF8Encoding]::new($false,$true).GetBytes(($Value | ConvertTo-Json -Compress))
    if ($bytes.Length -gt 1024) { throw 'GH_STOP_SIZE' }
    $stream = [IO.File]::Open($Path, [IO.FileMode]::CreateNew, [IO.FileAccess]::Write, [IO.FileShare]::None)
    try { $stream.Write($bytes); $stream.Flush($true) } finally { $stream.Dispose() }
}
function Assert-GhStopIdentity {
    param($Values)
    foreach ($key in @('profileId','attemptId')) {
        $id = [guid]::Empty
        if ($Values[$key].ValueKind -ne [Text.Json.JsonValueKind]::String -or
            -not [guid]::TryParseExact($Values[$key].GetString(),'D',[ref]$id) -or $id -eq [guid]::Empty -or $Values[$key].GetString() -cne $id.ToString('D')) { throw 'GH_STOP_ID' }
    }
    $number = 0
    if ($Values['controllerPid'].ValueKind -ne [Text.Json.JsonValueKind]::Number -or -not $Values['controllerPid'].TryGetInt32([ref]$number) -or $number -le 0) { throw 'GH_STOP_PID' }
    if ($Values['controllerBirthUtc'].ValueKind -ne [Text.Json.JsonValueKind]::String) { throw 'GH_STOP_BIRTH' }
    $null = ConvertFrom-GhDiagnosticUtc $Values['controllerBirthUtc'].GetString()
}
function Assert-GhStopRequest {
    param($Values, $Descriptor)
    Assert-GhStopIdentity $Values
    foreach ($key in @('schema','action')) { if ($Values[$key].ValueKind -ne [Text.Json.JsonValueKind]::String) { throw 'GH_STOP_TYPE' } }
    if ($Values['schema'].GetString() -cne 'wuxian.grey-hive.stop-request/v1' -or $Values['action'].GetString() -cne 'stop') { throw 'GH_STOP_ACTION' }
    foreach ($key in @('profileId','attemptId','controllerBirthUtc')) {
        if ($Values[$key].GetString() -cne $Descriptor[$key]) { throw 'GH_STOP_BINDING' }
    }
    if ($Values['controllerPid'].GetInt32() -ne $Descriptor.controllerPid) { throw 'GH_STOP_BINDING' }
}
function New-GhStopContext {
    param($Plan, $ProfileLease, [AllowNull()]$Budget)
    if ($null -eq $Budget) { return $null }
    Assert-GhDiagnosticBudget $Budget -ForStart
    $self = [Diagnostics.Process]::GetCurrentProcess()
    try { $birth = $self.StartTime.ToUniversalTime().ToString("yyyy-MM-dd'T'HH:mm:ss.fffffff'Z'"); $controller = $self.Id } finally { $self.Dispose() }
    $descriptor = [ordered]@{schema='wuxian.grey-hive.stop-descriptor/v1';profileId=$ProfileLease.ProfileId;attemptId=$Plan.AttemptId;controllerPid=$controller;controllerBirthUtc=$birth;firstLaunchUtc=$Budget.FirstLaunchUtc;hardDeadlineUtc=$Budget.HardDeadlineUtc}
    $dir = Resolve-GhOrdinaryPath $Plan.AttemptDir Directory
    Assert-GhContainedPath $Plan.AttemptsDir $dir
    $gate = [IO.File]::Open(($dir+'/stop.gate'),[IO.FileMode]::CreateNew,[IO.FileAccess]::ReadWrite,[IO.FileShare]::None)
    $gate.Dispose()
    Write-GhStopJson ($dir+'/stop-descriptor.json') $descriptor
    $pin = [IO.File]::Open(($dir+'/stop-descriptor.json'),[IO.FileMode]::Open,[IO.FileAccess]::Read,[IO.FileShare]::Read)
    return [pscustomobject]@{Directory=$dir;Descriptor=$descriptor;DescriptorPin=$pin;Budget=$Budget;Cancelled=$false;CleanupDeadline=[long]::MaxValue;Terminal=$false;RequestRejected=$false;RequestOutcome='NONE';RequestReason=$null}
}
function Enter-GhStopGate {
    param($Context)
    while ($true) {
        $deadline = Get-GhDiagnosticDeadline $Context.Budget $Context.Budget.HardTick
        if ($null -ne $Context.PSObject.Properties['CleanupDeadline']) { $deadline = [Math]::Min($deadline,$Context.CleanupDeadline) }
        if ([Diagnostics.Stopwatch]::GetTimestamp() -ge $deadline) { throw 'GH_STOP_GATE_DEADLINE' }
        $path = Resolve-GhOrdinaryPath ($Context.Directory+'/stop.gate') File
        try { return [IO.File]::Open($path,[IO.FileMode]::Open,[IO.FileAccess]::ReadWrite,[IO.FileShare]::None) }
        catch [IO.IOException] { if (($_.Exception.HResult -band 65535) -notin @(32,33)) { throw }; Start-Sleep -Milliseconds 20 }
    }
}
function Set-GhStopCancelled {
    param($Context)
    if (-not $Context.Cancelled) {
        $Context.Cancelled = $true
        $Context.Budget.Failed = $true
        $Context.CleanupDeadline = Get-GhDiagnosticDeadline $Context.Budget ([Math]::Min($Context.CleanupDeadline, [Diagnostics.Stopwatch]::GetTimestamp()+30L*[Diagnostics.Stopwatch]::Frequency))
        $Context.Budget.OperationalTick = [Math]::Min($Context.Budget.OperationalTick,[Diagnostics.Stopwatch]::GetTimestamp())
    }
}
function Assert-GhStop {
    param([AllowNull()]$Context, [switch]$GateHeld, [switch]$NoThrowOnCancel)
    if ($null -eq $Context) { return }
    if ($Context.Cancelled) { if (-not $NoThrowOnCancel) { throw 'GH_STOP_CANCELLED' }; return }
    $gate = $null
    try {
        if (-not $GateHeld) { $gate = Enter-GhStopGate $Context }
        if ($Context.Terminal) { return }
        # The fixed request is create-new/immutable: reject bad content once, not as stop authority.
        if ($Context.RequestRejected) { return }
        $requestPath = $Context.Directory+'/stop-request.json'
        if ([IO.File]::Exists($requestPath)) {
            try {
                $values = Read-GhStopJson $requestPath @('schema','action','profileId','attemptId','controllerPid','controllerBirthUtc')
                Assert-GhStopRequest $values $Context.Descriptor
            } catch {
                $cause = $_.Exception.GetBaseException()
                $reason = $cause.Message
                if ($cause -is [Text.DecoderFallbackException] -or $cause -is [Text.Json.JsonException]) { $reason = 'GH_STOP_ENCODING' }
                if ($reason -cnotin @('GH_JSON_LIMIT','GH_JSON_SIZE','GH_JSON_OBJECT','GH_JSON_DUPLICATE','GH_JSON_KEYS','GH_STOP_ID','GH_STOP_PID','GH_STOP_BIRTH','GH_STOP_TYPE','GH_STOP_ACTION','GH_STOP_BINDING','GH_DIAGNOSTIC_UTC','GH_STOP_ENCODING')) { throw }
                $Context.RequestRejected = $true; $Context.RequestOutcome = 'REJECTED_CONTENT'; $Context.RequestReason = $reason
                return
            }
            $Context.RequestOutcome = 'ACCEPTED_STOP'
            Set-GhStopCancelled $Context
            if (-not $NoThrowOnCancel) { throw 'GH_STOP_CANCELLED' }
        }
    } finally { if ($null -ne $gate) { $gate.Dispose() } }
}
function Send-GhStopRequest {
    param([string]$ProfileRoot,[string]$ProfileId,[string]$AttemptId,[int]$ControllerPid,[string]$ControllerBirthUtc)
    # No process query, child start or termination in the publisher.
    $request = [ordered]@{schema='wuxian.grey-hive.stop-request/v1';action='stop';profileId=$ProfileId;attemptId=$AttemptId;controllerPid=$ControllerPid;controllerBirthUtc=$ControllerBirthUtc}
    # Validate identities before using the attempt identifier in a path.
    $memory = [IO.MemoryStream]::new([Text.Encoding]::UTF8.GetBytes(($request | ConvertTo-Json -Compress)))
    try { $values = Read-GhJsonObject $memory 1024 @('schema','action','profileId','attemptId','controllerPid','controllerBirthUtc'); Assert-GhStopIdentity $values } finally { $memory.Dispose() }
    $root = Resolve-GhOrdinaryPath $ProfileRoot Directory
    $attempts = Resolve-GhOrdinaryPath ($root+'/attempts') Directory
    $dir = Resolve-GhOrdinaryPath ($attempts+'/'+$AttemptId) Directory
    Assert-GhContainedPath $root $attempts; Assert-GhContainedPath $attempts $dir
    $raw = Read-GhStopJson ($dir+'/stop-descriptor.json') @('schema','profileId','attemptId','controllerPid','controllerBirthUtc','firstLaunchUtc','hardDeadlineUtc')
    Assert-GhStopIdentity $raw
    $descriptor = @{}
    foreach ($key in @('schema','profileId','attemptId','controllerBirthUtc','firstLaunchUtc','hardDeadlineUtc')) {
        if ($raw[$key].ValueKind -ne [Text.Json.JsonValueKind]::String) { throw 'GH_STOP_TYPE' }; $descriptor[$key] = $raw[$key].GetString()
    }
    $descriptor.controllerPid = $raw['controllerPid'].GetInt32()
    if ($descriptor.schema -cne 'wuxian.grey-hive.stop-descriptor/v1') { throw 'GH_STOP_DESCRIPTOR' }
    Assert-GhStopRequest $values $descriptor
    $first = ConvertFrom-GhDiagnosticUtc $descriptor.firstLaunchUtc; $hard = ConvertFrom-GhDiagnosticUtc $descriptor.hardDeadlineUtc; $now = Get-GhDiagnosticUtcNow
    if (($hard-$first).TotalSeconds -le 600 -or ($hard-$first).TotalSeconds -gt 7200 -or $first -gt $now -or $hard -le $now) { throw 'GH_STOP_INTERVAL' }
    $budget = [pscustomobject]@{HardUtc=$hard;OperationalUtc=$hard.AddSeconds(-600);HardTick=[long]::MaxValue;OperationalTick=[long]::MaxValue;Failed=$false}
    $context = [pscustomobject]@{Directory=$dir;Budget=$budget}
    $gate = Enter-GhStopGate $context
    try {
        if ([IO.File]::Exists($dir+'/stop-result.json')) { throw 'GH_STOP_TERMINAL' }
        $active = Read-GhStopJson ($root+'/active-launch.json') @('schema','profileId','attemptId')
        foreach ($key in @('profileId','attemptId')) { if ($active[$key].ValueKind -ne [Text.Json.JsonValueKind]::String -or $active[$key].GetString() -cne $descriptor[$key]) { throw 'GH_STOP_STALE' } }
        if ($active['schema'].GetString() -cne 'wuxian.grey-hive.active-launch/v1') { throw 'GH_STOP_STALE' }
        Write-GhStopJson ($dir+'/stop-request.json') $request
    } finally { $gate.Dispose() }
    return 'REQUEST_PUBLISHED'
}
# Default healthy lifetime remains unbounded; diagnostic lifetime shares the supplied hard anchor.
function Wait-GhPlayerLifetime {
    param([Parameter(Mandatory)][Collections.Generic.List[object]]$OwnedRegistry, [AllowNull()]$DiagnosticBudget = $null, [AllowNull()]$StopContext = $null)
    Assert-GhStop $StopContext
    if ($OwnedRegistry.Count -ne 2) { throw 'GH_LIFETIME_REGISTRY' }
    $kinds = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    foreach ($owned in $OwnedRegistry) {
        if ($owned.Kind -cnotin @('Native','Godot') -or -not $kinds.Add($owned.Kind) -or -not $owned.Started -or -not $owned.HandleCaptured -or $owned.Disposed -or $owned.Exited) { throw 'GH_LIFETIME_REGISTRY' }
        if ($null -ne $owned.PSObject.Properties['NaturalExit']) { throw 'GH_LIFETIME_REENTRY' }
        $owned | Add-Member -MemberType NoteProperty -Name NaturalExit -Value ([bool]$false)
    }
    $peerDeadline = 0L; $remainingChildren = 2
    while ($remainingChildren -gt 0) {
        Assert-GhStop $StopContext
        Assert-GhDiagnosticBudget $DiagnosticBudget
        foreach ($owned in $OwnedRegistry) {
            Assert-GhStop $StopContext
            if ($owned.NaturalExit) { continue }
            if ($null -ne $DiagnosticBudget) {
                $peerDeadline = if ($peerDeadline -eq 0) { 0L } else { Get-GhDiagnosticDeadline $DiagnosticBudget $peerDeadline }
                if ($peerDeadline -eq 0 -and [Diagnostics.Stopwatch]::GetTimestamp() -ge $DiagnosticBudget.OperationalTick) { $DiagnosticBudget.Failed = $true; throw 'GH_DIAGNOSTIC_TIMEOUT' }
            }
            if ($peerDeadline -ne 0 -and [Diagnostics.Stopwatch]::GetTimestamp() -ge $peerDeadline) { throw 'GH_LIFETIME_PEER_TIMEOUT' }
            # WaitForExit(int) uses this retained process handle, never a PID lookup.
            $waitMs = 100
            if ($null -ne $DiagnosticBudget) {
                $limit = if ($peerDeadline -eq 0) { $DiagnosticBudget.OperationalTick } else { $peerDeadline }
                $waitMs = [int][Math]::Max(0, [Math]::Min(100, [Math]::Floor(($limit - [Diagnostics.Stopwatch]::GetTimestamp()) * 1000.0 / [Diagnostics.Stopwatch]::Frequency)))
            }
            if ($owned.Process.WaitForExit($waitMs)) {
                Assert-GhStop $StopContext
                $observed = [Diagnostics.Stopwatch]::GetTimestamp()
                if ($null -ne $DiagnosticBudget -and ($observed -ge $DiagnosticBudget.HardTick -or $peerDeadline -eq 0 -and $observed -ge $DiagnosticBudget.OperationalTick)) { $DiagnosticBudget.Failed = $true; throw 'GH_DIAGNOSTIC_TIMEOUT' }
                if ($peerDeadline -ne 0 -and $observed -ge $peerDeadline) { throw 'GH_LIFETIME_PEER_TIMEOUT' }
                if ($peerDeadline -eq 0) { $peerDeadline = Get-GhDiagnosticDeadline $DiagnosticBudget ($observed + 30L * [Diagnostics.Stopwatch]::Frequency) }
                $owned.NaturalExit = [bool]$true; $owned.Exited = $true
                $owned.ExitCode = [int]$owned.Process.ExitCode; $remainingChildren--
                if ($owned.ExitCode -ne 0) { throw 'GH_LIFETIME_CHILD_FAILURE' }
            }
            foreach ($drain in @($owned.StdoutTask, $owned.StderrTask)) {
                if ($null -eq $drain -or $drain.IsFaulted -or $drain.IsCanceled) { throw 'GH_LIFETIME_DRAIN_FAILURE' }
            }
        }
    }
    $drainDeadline = Get-GhDiagnosticDeadline $DiagnosticBudget ([Diagnostics.Stopwatch]::GetTimestamp() + 30L * [Diagnostics.Stopwatch]::Frequency)
    foreach ($owned in $OwnedRegistry) { Complete-GhChildDrain $owned $drainDeadline $StopContext }
}
function Close-GhDiagnosticOwnedFailure {
    param([Parameter(Mandatory)][AllowEmptyCollection()][Collections.Generic.List[object]]$OwnedRegistry, [Parameter(Mandatory)]$Budget, [AllowNull()]$StopContext = $null)
    $Budget.Failed = $true
    $deadline = Get-GhDiagnosticDeadline $Budget ([Diagnostics.Stopwatch]::GetTimestamp() + 30L * [Diagnostics.Stopwatch]::Frequency)
    if ($null -ne $StopContext) {
        $StopContext.CleanupDeadline = [Math]::Min($StopContext.CleanupDeadline,$deadline)
        $deadline = $StopContext.CleanupDeadline
    }
    # Termination requests for every eligible retained child precede any wait/drain.
    foreach ($owned in $OwnedRegistry) {
        try {
            if ($null -eq $owned.PSObject.Properties['NaturalExit']) { $owned | Add-Member -MemberType NoteProperty -Name NaturalExit -Value $false }
            $owned.NaturalExit = $false
            if ($owned.Disposed -or -not $owned.Started) { continue }
            $deadline = Get-GhDiagnosticDeadline $Budget $deadline
            if ([Diagnostics.Stopwatch]::GetTimestamp() -ge $deadline) { continue }
            if (-not $owned.Process.HasExited) {
                Assert-GhChild $owned $deadline
                $deadline = Get-GhDiagnosticDeadline $Budget $deadline
                if ([Diagnostics.Stopwatch]::GetTimestamp() -ge $deadline) { continue }
                if ($null -eq $owned.PSObject.Properties['CloseRequested']) { $owned | Add-Member -MemberType NoteProperty -Name CloseRequested -Value $false }
                $owned.CloseRequested = $true
                $owned.Process.Kill() # Retained, freshly verified process only; never tree/PID lookup or UI.
            }
        } catch { } # One invalid identity cannot prevent the valid peer's termination request.
    }
    foreach ($owned in $OwnedRegistry) {
        try {
            if ($owned.Disposed -or -not $owned.Started) { continue }
            while (-not $owned.Process.HasExited -and [Diagnostics.Stopwatch]::GetTimestamp() -lt $deadline) {
                $deadline = Get-GhDiagnosticDeadline $Budget $deadline
                $remainingMs = [Math]::Max(0, [Math]::Min(100, [Math]::Floor(($deadline - [Diagnostics.Stopwatch]::GetTimestamp()) * 1000.0 / [Diagnostics.Stopwatch]::Frequency)))
                $null = $owned.Process.WaitForExit([int]$remainingMs)
            }
            if ($owned.Process.HasExited) { Complete-GhChildDrain $owned $deadline }
        } catch { } # Failed identity/deadline/drain leaves UNKNOWN closure, never a clean receipt.
    }
}
# Failure never grants cleanup permission, including zero-code exits after a close request.
function Close-GhOwnedFailure {
    param([Parameter(Mandatory)][AllowEmptyCollection()][Collections.Generic.List[object]]$OwnedRegistry, [AllowNull()]$DiagnosticBudget = $null, [AllowNull()]$StopContext = $null)
    if ($null -ne $DiagnosticBudget) { Close-GhDiagnosticOwnedFailure $OwnedRegistry $DiagnosticBudget $StopContext; return }
    $deadline = [Diagnostics.Stopwatch]::GetTimestamp() + 30L * [Diagnostics.Stopwatch]::Frequency
    foreach ($owned in $OwnedRegistry) {
        try {
            if ($null -eq $owned.PSObject.Properties['NaturalExit']) { $owned | Add-Member -MemberType NoteProperty -Name NaturalExit -Value ([bool]$false) }
            if ($null -eq $owned.PSObject.Properties['CloseRequested']) { $owned | Add-Member -MemberType NoteProperty -Name CloseRequested -Value ([bool]$false) }
            if ($owned.Kind -ceq 'Godot' -and $owned.Started -and -not $owned.Disposed -and -not $owned.CloseRequested -and -not $owned.Process.HasExited) {
                $owned.CloseRequested = $true
                Assert-GhChild $owned $deadline
                $null = $owned.Process.CloseMainWindow()
            }
        } catch { }
    }
    do {
        $pending = $false
        foreach ($owned in $OwnedRegistry) {
            try {
                if ($owned.Disposed -or -not $owned.Started) { continue }
                if ($owned.Process.HasExited) {
                    $owned.NaturalExit = [bool]$true; $owned.Exited = $true; $owned.ExitCode = [int]$owned.Process.ExitCode
                    Complete-GhChildDrain $owned $deadline
                } else { $pending = $true }
            } catch { }
        }
        if ($pending -and [Diagnostics.Stopwatch]::GetTimestamp() -lt $deadline) { Start-Sleep -Milliseconds 100 }
    } while ($pending -and [Diagnostics.Stopwatch]::GetTimestamp() -lt $deadline)
}
# Called once from entry finally. Return only one sanitized status, never caught error details.
function Complete-GhPlayerAttempt {
    param([AllowNull()]$Plan, [AllowNull()]$ProfileLease, [AllowNull()]$ReadyLease,
        [Parameter(Mandatory)][AllowEmptyCollection()][Collections.Generic.List[object]]$OwnedRegistry, [AllowNull()]$Failure, [AllowNull()]$DiagnosticBudget = $null, [AllowNull()]$StopContext = $null)
    $status = 'GH_PLAYER_UNCERTAIN_CLOSURE'; $clean = $false; $released = $true
    $prestart = ($null -eq $ProfileLease -or -not $ProfileLease.Started)
    foreach ($owned in $OwnedRegistry) { if ($owned.StartAttempted -or $owned.Started) { $prestart = $false } }
    $terminalGate = $null
    try {
        if ($null -ne $StopContext) { $terminalGate = Enter-GhStopGate $StopContext }
        Assert-GhStop $StopContext -GateHeld
        Assert-GhDiagnosticBudget $DiagnosticBudget
        if ($null -ne $Failure -or $null -eq $Plan -or $null -eq $ProfileLease -or $null -eq $ReadyLease) { throw 'GH_ATTEMPT_INCOMPLETE' }
        if (-not [object]::ReferenceEquals($Plan, $ProfileLease.Plan) -or $ReadyLease.Path -cne $Plan.ReadyPath -or $OwnedRegistry.Count -ne 2) { throw 'GH_ATTEMPT_BINDING' }
        $records = @{}
        foreach ($owned in $OwnedRegistry) {
            $natural = $owned.PSObject.Properties['NaturalExit']
            if ($owned.Kind -cnotin @('Native','Godot') -or $records.ContainsKey($owned.Kind) -or $null -eq $natural -or $natural.Value -isnot [bool] -or -not $natural.Value) { throw 'GH_ATTEMPT_UNCLEAN' }
            if (-not $owned.Exited -or -not $owned.Drained -or -not $owned.Disposed -or $owned.ExitCode -isnot [int] -or $owned.ExitCode -ne 0) { throw 'GH_ATTEMPT_UNCLEAN' }
            $requested = $owned.PSObject.Properties['CloseRequested']
            if ($null -ne $requested -and $requested.Value) { throw 'GH_ATTEMPT_UNCLEAN' }
            $records[$owned.Kind] = $owned
        }
        Remove-GhReady $ReadyLease $OwnedRegistry
        $facts = @{ nativeNaturalExit=$records.Native.NaturalExit; godotNaturalExit=$records.Godot.NaturalExit; nativeExitCode=$records.Native.ExitCode; godotExitCode=$records.Godot.ExitCode; drainsCompleted=[bool]$true; readyRemoved=$ReadyLease.Removed }
        Complete-GhProfile $ProfileLease $facts
        $clean = $true; $status = 'GH_PLAYER_NATURAL_CLOSURE'
    } catch { if ($prestart -and ($null -eq $StopContext -or -not $StopContext.Cancelled)) { $status = 'GH_PLAYER_PRESTART_REJECTED' } }
    finally {
        if ($null -ne $StopContext) {
            try {
                # Commit terminal admission under the same gate as start/request publication.
                if ($null -eq $terminalGate) { $terminalGate = Enter-GhStopGate $StopContext }
                $StopContext.Terminal = $true
                Write-GhStopJson ($StopContext.Directory+'/stop-result.json') ([ordered]@{schema='wuxian.grey-hive.stop-result/v1';attemptId=$StopContext.Descriptor.attemptId;cancelled=[bool]$StopContext.Cancelled;status=$status})
            } catch { $released = $false }
            finally { if ($null -ne $terminalGate) { $terminalGate.Dispose() } }
        }
        if (-not $clean) { try { Close-GhOwnedFailure $OwnedRegistry $DiagnosticBudget $StopContext } catch { $released = $false } }
        # Dispose each resource independently, even if a previous disposal failed. No path deletion.
        foreach ($owned in $OwnedRegistry) {
            if ($owned.Disposed) { continue }
            try { if ($owned.Started) { $owned.Process.StandardOutput.Dispose() } } catch { $released = $false }
            try { if ($owned.Started) { $owned.Process.StandardError.Dispose() } } catch { $released = $false }
            try { $owned.Process.Dispose(); $owned.Disposed = $true } catch { $released = $false }
            foreach ($drain in @($owned.StdoutTask, $owned.StderrTask)) { if ($null -ne $drain -and $drain.IsFaulted) { $null = $drain.Exception } }
        }
        if ($null -ne $ReadyLease) {
            try { if ($null -ne $ReadyLease.Stream) { $ReadyLease.Stream.Dispose() } } catch { $released = $false }
            try { if ($null -ne $ReadyLease.Handle) { $ReadyLease.Handle.Dispose() } } catch { $released = $false }
        }
        try { if ($null -ne $ProfileLease) { Release-GhProfile $ProfileLease } } catch { $released = $false }
        try { if ($null -ne $StopContext -and $null -ne $StopContext.DescriptorPin) { $StopContext.DescriptorPin.Dispose() } } catch { $released = $false }
    }
    if (-not $released) { $status = 'GH_PLAYER_UNCERTAIN_CLOSURE' }
    return $status
}
