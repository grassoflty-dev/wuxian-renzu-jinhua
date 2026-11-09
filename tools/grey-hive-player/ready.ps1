# Definitions only. Ready values contain the token and must never be printed or serialized.
function Wait-GhReady {
    param([Parameter(Mandatory)]$Plan, [Parameter(Mandatory)]$NativeOwned, [Parameter(Mandatory)][string]$Token,
        [Parameter(Mandatory)][long]$StartupDeadline, [AllowNull()]$StopContext = $null)
    $handle = $null; $stream = $null; $transferred = $false
    $readyStage = 'INPUT'
    try {
        Assert-GhStop $StopContext
        if ($NativeOwned.Kind -cne 'Native' -or $StartupDeadline -le 0 -or $StartupDeadline -ne $NativeOwned.StartupDeadline -or $Token -cnotmatch '^[0-9a-f]{64}$') { throw 'GH_READY_INPUT' }
        $readyStage = 'CHILD'
        while ($null -eq $handle) {
            Assert-GhStop $StopContext
            Assert-GhChild $NativeOwned $StartupDeadline
            $readyStage = 'PATH'
            $path = Resolve-GhOrdinaryPath $Plan.ReadyPath File $true
            Assert-GhContainedPath $Plan.RunDir $path
            if ([Diagnostics.Stopwatch]::GetTimestamp() -ge $StartupDeadline) { throw 'GH_READY_DEADLINE' }
            $readyStage = 'OPEN'
            try { $handle = [Wuxian.Player.PlayerWindows]::OpenReadPinned($path, $false) }
            catch {
                $cause = $_.Exception.GetBaseException()
                if ($cause -isnot [ComponentModel.Win32Exception] -or $cause.NativeErrorCode -notin @(2, 3, 32)) { throw 'GH_READY_OPEN' }
                # Only a missing file or an unfinished writer is pending; never retry parsed content.
                Assert-GhChild $NativeOwned $StartupDeadline
                Assert-GhStop $StopContext
                $remaining = ($StartupDeadline - [Diagnostics.Stopwatch]::GetTimestamp()) / [double][Diagnostics.Stopwatch]::Frequency
                if ($remaining -le 0) { throw 'GH_READY_DEADLINE' }
                Start-Sleep -Milliseconds ([int][Math]::Min(100, [Math]::Max(1, [Math]::Floor($remaining * 1000))))
            }
        }
        $readyStage = 'IDENTITY'
        $identity = [Wuxian.Player.PlayerWindows]::ReadIdentity($handle)
        Assert-GhStop $StopContext
        if ($identity.Length -lt 1 -or $identity.Length -gt 16384) { throw 'GH_READY_SIZE' }
        # FileStream owns this same read-only pin; no DELETE access until final cleanup.
        $stream = [IO.FileStream]::new($handle, [IO.FileAccess]::Read)
        $readyStage = 'JSON'
        $values = Read-GhJsonObject $stream 16384 @('schema','protocolVersion','host','port','processId','sessionToken','sessionDir','saveDir')
        $readyStage = 'STRING_TYPE'
        foreach ($key in @('schema','host','sessionToken','sessionDir','saveDir')) {
            if ($values[$key].ValueKind -ne [Text.Json.JsonValueKind]::String) { throw 'GH_READY_TYPE' }
        }
        $readyStage = 'STRING_VALUE'
        $expected = @{ schema = 'wuxian.godot-v1.ready/v1'; host = '127.0.0.1'; sessionToken = $Token; sessionDir = $Plan.SessionDir; saveDir = $Plan.SaveDir }
        foreach ($key in $expected.Keys) {
            if (-not [string]::Equals($values[$key].GetString(), $expected[$key], [StringComparison]::Ordinal)) { throw 'GH_READY_VALUE' }
        }
        $readyStage = 'NUMBER_TYPE'
        $numbers = @{}
        foreach ($key in @('protocolVersion','port','processId')) {
            $number = 0
            if ($values[$key].ValueKind -ne [Text.Json.JsonValueKind]::Number -or -not $values[$key].TryGetInt32([ref]$number)) { throw 'GH_READY_TYPE' }
            $numbers[$key] = $number
        }
        $readyStage = 'NUMBER_VALUE'
        if ($numbers.protocolVersion -ne 1 -or $numbers.port -lt 1 -or $numbers.port -gt 65535 -or $numbers.processId -le 0 -or $numbers.processId -ne $NativeOwned.Pid) { throw 'GH_READY_VALUE' }
        $readyStage = 'HASH'
        $stream.Position = 0
        $hash = [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($stream)).ToLowerInvariant()
        $readyStage = 'LISTENER'
        Assert-GhReadyListener $values $NativeOwned $StartupDeadline $StopContext
        $readyStage = 'FINAL'
        Assert-GhStop $StopContext
        if ([Diagnostics.Stopwatch]::GetTimestamp() -ge $StartupDeadline) { throw 'GH_READY_DEADLINE' }
        $lease = [pscustomobject]@{ Path = $path; Handle = $handle; Stream = $stream; Identity = $identity; Sha256 = $hash; Values = $values; NativeOwned = $NativeOwned; Removed = $false }
        $transferred = $true
        return $lease
    } catch {
        # Emit only allowlisted literal codes, never exception details or ready values.
        $listenerCode = $_.Exception.Message
        $listenerCodes = @('GH_READY_LISTENER_CHILD_BEFORE','GH_READY_LISTENER_HOST','GH_READY_LISTENER_PORT','GH_READY_LISTENER_PROCESS','GH_READY_LISTENER_DEADLINE','GH_READY_LISTENER_QUERY','GH_READY_LISTENER_ROW','GH_READY_LISTENER_CHILD_AFTER','GH_READY_LISTENER_PROOF')
        $stages = @('INPUT','CHILD','PATH','OPEN','IDENTITY','JSON','STRING_TYPE','STRING_VALUE','NUMBER_TYPE','NUMBER_VALUE','HASH','LISTENER','FINAL')
        # New listener row codes contain only this closed set of stage/type labels.
        $safeListener = '^GH_READY_LISTENER_(DEADLINE|COUNT_ZERO|COUNT_MANY|ADDR_TYPE|ADDR_VALUE|PORT|STATE_ENUM|STATE_STRING|STATE_NUMERIC|STATE_OTHER|PID|HOST|PROCESS|QUERY|CHILD_BEFORE|CHILD_AFTER)_(REJECT|CIM|WIN32|IO|OTHER)(_NET_(PRESENT|ABSENT|UNKNOWN)_PID_(ZERO|ONE|MANY|UNKNOWN))?$'
        if ($readyStage -ceq 'LISTENER' -and ($listenerCode -cin $listenerCodes -or $listenerCode -cmatch $safeListener)) { throw ('GH_READY_INVALID_' + $listenerCode.Substring(9)) }
        if ($readyStage -cin $stages) { throw ('GH_READY_INVALID_' + $readyStage) }
        throw 'GH_READY_INVALID'
    }
    finally { if (-not $transferred) { if ($null -ne $stream) { $stream.Dispose() }; if ($null -ne $handle) { $handle.Dispose() } } }
}
# Accept only the retained native child's target-specific kernel duplicate of the same listener.
function Assert-GhReadyListener {
    param([Parameter(Mandatory)]$ReadyValues, [Parameter(Mandatory)]$NativeOwned, [Parameter(Mandatory)][long]$Deadline, [AllowNull()]$StopContext = $null)
    try {
        Assert-GhStop $StopContext
        Assert-GhChild $NativeOwned $Deadline
        $port=0; $processId=0
        if ($NativeOwned.Kind -cne 'Native' -or $null -eq $NativeOwned.NativeErrorCapture -or
            $ReadyValues['host'].ValueKind -ne [Text.Json.JsonValueKind]::String -or $ReadyValues['host'].GetString() -cne '127.0.0.1' -or
            $ReadyValues['port'].ValueKind -ne [Text.Json.JsonValueKind]::Number -or -not $ReadyValues['port'].TryGetInt32([ref]$port) -or $port -lt 1 -or $port -gt 65535 -or
            $ReadyValues['processId'].ValueKind -ne [Text.Json.JsonValueKind]::Number -or -not $ReadyValues['processId'].TryGetInt32([ref]$processId) -or $processId -ne $NativeOwned.Pid) { throw 'GH_READY_LISTENER_PROOF' }
        while ($true) {
            Assert-GhStop $StopContext
            Assert-GhChild $NativeOwned $Deadline
            if ([Diagnostics.Stopwatch]::GetTimestamp() -ge $Deadline) { throw 'GH_READY_LISTENER_PROOF' }
            $state=$NativeOwned.NativeErrorCapture.VerifyOwnerProof($port)
            if ($state -eq 1) { break }
            if ($state -ne 0 -or $NativeOwned.NativeErrorCapture.Completion.IsCompleted) { throw 'GH_READY_LISTENER_PROOF' }
            $remaining=($Deadline-[Diagnostics.Stopwatch]::GetTimestamp())/[double][Diagnostics.Stopwatch]::Frequency
            if ($remaining -le 0) { throw 'GH_READY_LISTENER_PROOF' }
            Start-Sleep -Milliseconds ([int][Math]::Max(1,[Math]::Min(25,[Math]::Floor($remaining*1000))))
        }
        Assert-GhChild $NativeOwned $Deadline
        Assert-GhStop $StopContext
        if ([Diagnostics.Stopwatch]::GetTimestamp() -ge $Deadline) { throw 'GH_READY_LISTENER_PROOF' }
        [Console]::Out.WriteLine('GH_READY_KERNEL_SOCKET_VERIFIED')
    } catch { throw 'GH_READY_LISTENER_PROOF' }
}
# L4 has no NaturalExit field: L6 must Add-Member a boolean, initially false, on each owned record.
# Set it true only on an observed natural exit before drain/disposal; failures only dispose the pin.
function Remove-GhReady {
    param([Parameter(Mandatory)]$ReadyLease, [Parameter(Mandatory)][Collections.Generic.List[object]]$OwnedRegistry)
    $handle = $null; $stream = $null
    try {
        if ($ReadyLease.Removed -or $OwnedRegistry.Count -ne 2) { throw 'GH_READY_CLOSURE' }
        $kinds = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
        foreach ($owned in $OwnedRegistry) {
            if ($owned.Kind -cnotin @('Native','Godot') -or -not $kinds.Add($owned.Kind) -or -not $owned.Started -or -not $owned.HandleCaptured -or $null -eq $owned.BirthTicks -or $owned.Pid -le 0) { throw 'GH_READY_CLOSURE' }
            $natural = $owned.PSObject.Properties['NaturalExit']
            if ($null -eq $natural -or $natural.Value -isnot [bool] -or -not $natural.Value -or -not $owned.Exited -or -not $owned.Drained -or -not $owned.Disposed -or $owned.ExitCode -isnot [int] -or $owned.ExitCode -ne 0) { throw 'GH_READY_CLOSURE' }
            if ($owned.Kind -ceq 'Native' -and -not [object]::ReferenceEquals($owned, $ReadyLease.NativeOwned)) { throw 'GH_READY_CLOSURE' }
        }
        if ($null -eq $ReadyLease.Stream -or $null -eq $ReadyLease.Handle -or $ReadyLease.Handle.IsClosed -or $ReadyLease.Handle.IsInvalid) { throw 'GH_READY_PIN' }
        $ReadyLease.Stream.Dispose(); $ReadyLease.Handle.Dispose()
        $path = Resolve-GhOrdinaryPath $ReadyLease.Path File
        $handle = [Wuxian.Player.PlayerWindows]::OpenReadPinned($path, $true)
        $identity = [Wuxian.Player.PlayerWindows]::ReadIdentity($handle)
        foreach ($key in @('VolumeSerialNumber','FileIdLow','FileIdHigh','CreationTimeUtcTicks','Length','Attributes')) {
            if ($identity.$key -ne $ReadyLease.Identity.$key) { throw 'GH_READY_CHANGED' }
        }
        $stream = [IO.FileStream]::new($handle, [IO.FileAccess]::Read)
        $hash = [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($stream)).ToLowerInvariant()
        if (-not [string]::Equals($hash, $ReadyLease.Sha256, [StringComparison]::Ordinal)) { throw 'GH_READY_CHANGED' }
        # DeletePinned consumes this very handle; disposing the stream first would close it.
        [Wuxian.Player.PlayerWindows]::DeletePinned($handle)
        $ReadyLease.Removed = $true
    } catch { throw 'GH_READY_REMOVE' }
    finally { if ($null -ne $stream) { $stream.Dispose() }; if ($null -ne $handle) { $handle.Dispose() } }
}
