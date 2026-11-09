# Definitions only. Owned records and their arguments/command lines must never be serialized.
function New-GhChildEnvironment {
    param([Parameter(Mandatory)]$Plan, [Parameter(Mandatory)][Diagnostics.ProcessStartInfo]$ProcessStartInfo)
    foreach ($key in @($ProcessStartInfo.Environment.Keys)) {
        if ($key.StartsWith('WUXIAN_GH_', [StringComparison]::OrdinalIgnoreCase)) { $null = $ProcessStartInfo.Environment.Remove($key) }
    }
    $ProcessStartInfo.Environment['APPDATA'] = $Plan.RoamingDir
    $ProcessStartInfo.Environment['LOCALAPPDATA'] = $Plan.LocalDir
    $ProcessStartInfo.Environment['TEMP'] = $Plan.TempDir
    $ProcessStartInfo.Environment['TMP'] = $Plan.TempDir
}
function New-GhFrontendObservationCarrier {
    param([Parameter(Mandatory)]$Plan, [Parameter(Mandatory)][Diagnostics.ProcessStartInfo]$ProcessStartInfo,
        [Parameter(Mandatory)][ValidateSet('Native','Godot')][string]$Kind)
    $carrier = 'WUXIAN_GH_FRONTEND_OBSERVATION_PATH'
    $null = $ProcessStartInfo.Environment.Remove($carrier)
    if ($Kind -cne 'Godot' -or $ProcessStartInfo.Environment['WUXIAN_V1_INPUT_TRACE'] -cne '1') { return }
    $reservation = $null
    try {
        $dir = Resolve-GhOrdinaryPath $Plan.AttemptDir Directory
        $log = Resolve-GhOrdinaryPath $Plan.GodotLog File $true
        if (-not [string]::Equals($log,$dir.TrimEnd('/')+'/godot.log',[StringComparison]::OrdinalIgnoreCase)) { return }
        $path = Resolve-GhOrdinaryPath ($dir.TrimEnd('/')+'/frontend-live-observation.jsonl') File $true
        Assert-GhContainedPath $dir $path
        $reservation = [IO.File]::Open($path,[IO.FileMode]::CreateNew,[IO.FileAccess]::Write,[IO.FileShare]::Read)
        $reservation.Dispose(); $reservation = $null
        $ProcessStartInfo.Environment[$carrier] = $path
    } catch { $null = $ProcessStartInfo.Environment.Remove($carrier) } # Diagnostics never fail child admission.
    finally { if ($null -ne $reservation) { try { $reservation.Dispose() } catch { } } }
}
# Environment/argument setup: 35-line ceiling; registered start: 50-line ceiling.
function Start-GhChild {
    param([Parameter(Mandatory)][ValidateSet('Native','Godot')][string]$Kind, [Parameter(Mandatory)][string]$Executable,
        [Parameter(Mandatory)][AllowEmptyCollection()][string[]]$Arguments, [Parameter(Mandatory)][string]$WorkingDirectory,
        [Parameter(Mandatory)]$Plan, [Parameter(Mandatory)][AllowEmptyCollection()][Collections.Generic.List[object]]$OwnedRegistry, [AllowNull()]$DiagnosticBudget = $null, [AllowNull()]$StopContext = $null)
    try {
        Assert-GhStop $StopContext
        Assert-GhDiagnosticBudget $DiagnosticBudget -ForStart
        foreach ($owned in $OwnedRegistry) { if ($owned.Kind -eq $Kind) { throw 'GH_CHILD_DUPLICATE' } }
        $image = Resolve-GhOrdinaryPath $Executable File; $directory = Resolve-GhOrdinaryPath $WorkingDirectory Directory
        $expectedDirectory = if ($Kind -eq 'Native') { [IO.Path]::GetDirectoryName($Plan.NativeExe).Replace('\','/') } else { $Plan.ProjectDir }
        if (-not [string]::Equals($image, $Plan[$Kind + 'Exe'], [StringComparison]::OrdinalIgnoreCase) -or -not [string]::Equals($directory, $expectedDirectory, [StringComparison]::OrdinalIgnoreCase)) { throw 'GH_CHILD_PATH' }
        $psi = [Diagnostics.ProcessStartInfo]::new(); $psi.FileName = $image; $psi.WorkingDirectory = $directory; $psi.UseShellExecute = $false
        $psi.RedirectStandardOutput = $true; $psi.RedirectStandardError = $true; $psi.CreateNoWindow = $Kind -eq 'Native'
        $psi.WindowStyle = if ($Kind -eq 'Native') { [Diagnostics.ProcessWindowStyle]::Hidden } else { [Diagnostics.ProcessWindowStyle]::Normal }
        foreach ($argument in $Arguments) { if ($null -eq $argument -or $argument.Contains([char]0)) { throw 'GH_CHILD_ARGUMENT' }; $psi.ArgumentList.Add($argument) }
        New-GhChildEnvironment $Plan $psi
        New-GhFrontendObservationCarrier $Plan $psi $Kind
        # Register the exact process object before any Start call, including all failure paths.
        $process = [Diagnostics.Process]::new(); $process.StartInfo = $psi
        $record = [pscustomobject]@{
            Kind = $Kind; Process = $process; StartAttempted = $false; Started = $false; HandleCaptured = $false
            FirstGuardCode = 'NOT_OBSERVED'; FirstGuardReason = 'NOT_OBSERVED'
            Pid = $null; BirthTicks = $null; Image = $image; Arguments = [string[]]$Arguments.Clone()
            CimBirthTicks = $null; CommandLine = $null; StartupDeadline = [long]0
            StdoutTask = $null; StderrTask = $null; NativeErrorCapture = $null; Exited = $false; ExitCode = $null; Drained = $false; Disposed = $false
        }
        $OwnedRegistry.Add($record)
        $startGate = $null
        try {
            if ($null -ne $StopContext) { $startGate = Enter-GhStopGate $StopContext }
            Assert-GhStop $StopContext -GateHeld
            $record.StartupDeadline = Get-GhDiagnosticDeadline $DiagnosticBudget ([Diagnostics.Stopwatch]::GetTimestamp() + 90L * [Diagnostics.Stopwatch]::Frequency) -Operational
            Assert-GhDiagnosticBudget $DiagnosticBudget -ForStart
            $record.StartupDeadline = Get-GhDiagnosticDeadline $DiagnosticBudget $record.StartupDeadline -Operational
            $record.StartAttempted = $true
            if (-not $process.Start()) { throw 'GH_CHILD_START' }
        } finally {
            # Start can throw after acquiring a process: retain every observation independently.
            try { $record.Pid = $process.Id; $record.Started = $true } catch {}
            if ($record.Started) {
                try { $null = $process.Handle; $record.HandleCaptured = $true; $record.BirthTicks = $process.StartTime.ToUniversalTime().Ticks } catch {}
                try { $record.StdoutTask = $process.StandardOutput.BaseStream.CopyToAsync([IO.Stream]::Null) } catch {}
                try {
                    if ($Kind -ceq 'Native') {
                        $record.NativeErrorCapture = [Wuxian.Player.PlayerWindows+NativeErrorCapture]::new($process.StandardError.BaseStream)
                        $record.StderrTask = $record.NativeErrorCapture.Completion
                    } else { $record.StderrTask = $process.StandardError.BaseStream.CopyToAsync([IO.Stream]::Null) }
                } catch {}
            }
            if ($null -ne $startGate) { $startGate.Dispose() }
        }
        Assert-GhStop $StopContext
        if (-not $record.Started) { throw 'GH_CHILD_NOT_STARTED' }
        if (-not $record.HandleCaptured -or $null -eq $record.BirthTicks) { throw 'GH_CHILD_HANDLE' }
        if ($null -eq $record.StdoutTask) { throw 'GH_CHILD_STDOUT_DRAIN' }
        if ($null -eq $record.StderrTask) { throw 'GH_CHILD_STDERR_DRAIN' }
        Assert-GhChild $record $record.StartupDeadline
        return $record
    } catch { throw }
}
function Read-GhChildImagePath {
    param([Parameter(Mandatory)]$OwnedRecord, [Parameter(Mandatory)][long]$Deadline)
    $process = $OwnedRecord.Process
    while ($true) {
        $process.Refresh()
        if ($process.HasExited -or $process.Id -ne $OwnedRecord.Pid -or $process.StartTime.ToUniversalTime().Ticks -ne $OwnedRecord.BirthTicks) { throw 'GH_CHILD_IDENTITY' }
        if ([Diagnostics.Stopwatch]::GetTimestamp() -ge $Deadline) { throw 'GH_CHILD_IMAGE_UNAVAILABLE' }
        $module = $process.MainModule
        $path = if ($null -eq $module) { $null } else { $module.FileName }
        if (-not [string]::IsNullOrEmpty($path)) {
            if ([Diagnostics.Stopwatch]::GetTimestamp() -ge $Deadline) { throw 'GH_CHILD_IDENTITY' }
            return $path
        }
        $remainingMs = ($Deadline - [Diagnostics.Stopwatch]::GetTimestamp()) * 1000.0 / [Diagnostics.Stopwatch]::Frequency
        if ($remainingMs -le 0) { throw 'GH_CHILD_IMAGE_UNAVAILABLE' }
        [Threading.Thread]::Sleep([int][Math]::Min(20, [Math]::Ceiling($remainingMs)))
    }
}
# Deadline is an absolute Int64 Stopwatch timestamp. CIM observations never establish ownership.
function Set-GhFirstChildGuard {
    param($Record, [string]$Code, [string]$Reason)
    try { if ($Record.FirstGuardCode -ceq 'NOT_OBSERVED') { $Record.FirstGuardCode = $Code; $Record.FirstGuardReason = $Reason } } catch { }
}
function Assert-GhChild {
    param([Parameter(Mandatory)]$OwnedRecord, [Parameter(Mandatory)][long]$Deadline)
    try {
        if ($OwnedRecord.Disposed -or -not $OwnedRecord.Started -or -not $OwnedRecord.HandleCaptured -or $null -eq $OwnedRecord.BirthTicks) { Set-GhFirstChildGuard $OwnedRecord 'GH_CHILD_IDENTITY' 'STATE'; throw 'GH_CHILD_IDENTITY' }
        $process = $OwnedRecord.Process
        if ([Diagnostics.Stopwatch]::GetTimestamp() -ge $Deadline) { Set-GhFirstChildGuard $OwnedRecord 'GH_CHILD_IDENTITY' 'DEADLINE'; throw 'GH_CHILD_IDENTITY' }
        if ($process.HasExited) { Set-GhFirstChildGuard $OwnedRecord 'GH_CHILD_IDENTITY' 'EXITED'; throw 'GH_CHILD_IDENTITY' }
        if ($process.Id -ne $OwnedRecord.Pid) { Set-GhFirstChildGuard $OwnedRecord 'GH_CHILD_IDENTITY' 'PID_MISMATCH'; throw 'GH_CHILD_IDENTITY' }
        if ($process.StartTime.ToUniversalTime().Ticks -ne $OwnedRecord.BirthTicks) { Set-GhFirstChildGuard $OwnedRecord 'GH_CHILD_IDENTITY' 'BIRTH_MISMATCH'; throw 'GH_CHILD_IDENTITY' }
        $image = Resolve-GhOrdinaryPath (Read-GhChildImagePath $OwnedRecord $Deadline) File
        if (-not [string]::Equals($image, $OwnedRecord.Image, [StringComparison]::OrdinalIgnoreCase)) { throw 'GH_CHILD_IMAGE' }
        $remaining = ($Deadline - [Diagnostics.Stopwatch]::GetTimestamp()) / [double][Diagnostics.Stopwatch]::Frequency
        if ($remaining -lt 1) { throw 'GH_CHILD_DEADLINE' }
        $timeout = [int][Math]::Min(5, [Math]::Floor($remaining))
        $rows = @(CimCmdlets\Get-CimInstance -Namespace 'root/cimv2' -ClassName 'Win32_Process' -Filter "ProcessId = $($OwnedRecord.Pid)" -OperationTimeoutSec $timeout -ErrorAction Stop)
        if ([Diagnostics.Stopwatch]::GetTimestamp() -ge $Deadline -or $rows.Count -ne 1) { throw 'GH_CHILD_CIM' }
        $row = $rows[0]
        if ($row.ProcessId -ne $OwnedRecord.Pid -or $row.CreationDate -isnot [datetime] -or $row.ExecutablePath -isnot [string] -or $row.CommandLine -isnot [string] -or [string]::IsNullOrWhiteSpace($row.CommandLine)) { throw 'GH_CHILD_CIM' }
        $cimImage = Resolve-GhOrdinaryPath $row.ExecutablePath File; $cimBirth = $row.CreationDate.ToUniversalTime().Ticks
        if (-not [string]::Equals($cimImage, $OwnedRecord.Image, [StringComparison]::OrdinalIgnoreCase)) { throw 'GH_CHILD_CIM_IMAGE' }
        $decoded = [Wuxian.Player.PlayerWindows]::SplitArguments($row.CommandLine)
        if ($decoded.Length -ne ($OwnedRecord.Arguments.Length + 1)) { throw 'GH_CHILD_ARGV' }
        $argvImage = Resolve-GhOrdinaryPath $decoded[0] File
        if (-not [string]::Equals($argvImage, $OwnedRecord.Image, [StringComparison]::OrdinalIgnoreCase)) { throw 'GH_CHILD_ARGV_IMAGE' }
        for ($i = 0; $i -lt $OwnedRecord.Arguments.Length; $i++) {
            if (-not [string]::Equals($decoded[$i + 1], $OwnedRecord.Arguments[$i], [StringComparison]::Ordinal)) { throw 'GH_CHILD_ARGV' }
        }
        if ($null -ne $OwnedRecord.CimBirthTicks -and ($cimBirth -ne $OwnedRecord.CimBirthTicks -or -not [string]::Equals($row.CommandLine, $OwnedRecord.CommandLine, [StringComparison]::Ordinal))) { throw 'GH_CHILD_CIM_CHANGED' }
        # Bracket CIM with the retained handle's liveness; a recycled PID is never adopted.
        if ($process.HasExited) { Set-GhFirstChildGuard $OwnedRecord 'GH_CHILD_IDENTITY' 'EXITED'; throw 'GH_CHILD_IDENTITY' }
        if ($process.Id -ne $OwnedRecord.Pid) { Set-GhFirstChildGuard $OwnedRecord 'GH_CHILD_IDENTITY' 'PID_MISMATCH'; throw 'GH_CHILD_IDENTITY' }
        if ($process.StartTime.ToUniversalTime().Ticks -ne $OwnedRecord.BirthTicks) { Set-GhFirstChildGuard $OwnedRecord 'GH_CHILD_IDENTITY' 'BIRTH_MISMATCH'; throw 'GH_CHILD_IDENTITY' }
        if ([Diagnostics.Stopwatch]::GetTimestamp() -ge $Deadline) { Set-GhFirstChildGuard $OwnedRecord 'GH_CHILD_IDENTITY' 'DEADLINE'; throw 'GH_CHILD_IDENTITY' }
        if ($null -eq $OwnedRecord.CimBirthTicks) { $OwnedRecord.CimBirthTicks = $cimBirth; $OwnedRecord.CommandLine = $row.CommandLine }
    } catch {
        try {
            $known = @('GH_CHILD_IDENTITY','GH_CHILD_IMAGE','GH_CHILD_IMAGE_UNAVAILABLE','GH_CHILD_DEADLINE','GH_CHILD_CIM','GH_CHILD_CIM_IMAGE','GH_CHILD_ARGV','GH_CHILD_ARGV_IMAGE','GH_CHILD_CIM_CHANGED')
            $code = if ($_.Exception.Message -cin $known) { [string]$_.Exception.Message } else { 'UNKNOWN' }
            Set-GhFirstChildGuard $OwnedRecord $code 'UNKNOWN'
        } catch { }
        throw
    }
}
# ExitCode/Exited survive disposal. A failed drain is never promoted to successful closure.
function Complete-GhChildDrain {
    param([Parameter(Mandatory)]$OwnedRecord, [Parameter(Mandatory)][long]$Deadline, [AllowNull()]$StopContext = $null)
    if ($OwnedRecord.Disposed) { if ($OwnedRecord.Exited -and $OwnedRecord.Drained) { return }; throw 'GH_CHILD_DRAIN' }
    try {
        if (-not $OwnedRecord.Started -or -not $OwnedRecord.Process.HasExited) { throw 'GH_CHILD_DRAIN' }
        $OwnedRecord.ExitCode = [int]$OwnedRecord.Process.ExitCode; $OwnedRecord.Exited = $true
        try {
            if ($null -eq $OwnedRecord.StdoutTask -or $null -eq $OwnedRecord.StderrTask) { throw 'GH_CHILD_DRAIN' }
            $remaining = ($Deadline - [Diagnostics.Stopwatch]::GetTimestamp()) / [double][Diagnostics.Stopwatch]::Frequency
            if ($remaining -le 0) { throw 'GH_CHILD_DRAIN' }
            $drains = [Threading.Tasks.Task]::WhenAll([Threading.Tasks.Task[]]@($OwnedRecord.StdoutTask, $OwnedRecord.StderrTask))
            do {
                Assert-GhStop $StopContext -NoThrowOnCancel
                if ($null -ne $StopContext -and $StopContext.Cancelled) {
                    $StopContext.CleanupDeadline = [Math]::Min($StopContext.CleanupDeadline,$Deadline)
                    $Deadline = Get-GhDiagnosticDeadline $StopContext.Budget $StopContext.CleanupDeadline
                }
                $remaining = ($Deadline - [Diagnostics.Stopwatch]::GetTimestamp()) / [double][Diagnostics.Stopwatch]::Frequency
                if ($remaining -le 0) { throw 'GH_CHILD_DRAIN' }
                $milliseconds = [int][Math]::Min($(if ($null -eq $StopContext) { [int]::MaxValue } else { 100 }), [Math]::Floor($remaining * 1000))
                $done = $drains.Wait($milliseconds)
            } while (-not $done)
            Assert-GhStop $StopContext -NoThrowOnCancel
            if ($null -ne $StopContext -and $StopContext.Cancelled) {
                $StopContext.CleanupDeadline = [Math]::Min($StopContext.CleanupDeadline,$Deadline)
                $Deadline = Get-GhDiagnosticDeadline $StopContext.Budget $StopContext.CleanupDeadline
            }
            if ([Diagnostics.Stopwatch]::GetTimestamp() -ge $Deadline -or -not $drains.IsCompletedSuccessfully) { throw 'GH_CHILD_DRAIN' }
            $OwnedRecord.Drained = $true
        } finally {
            try { $OwnedRecord.Process.StandardOutput.Dispose() } catch {}
            try { $OwnedRecord.Process.StandardError.Dispose() } catch {}
            $OwnedRecord.Process.Dispose(); $OwnedRecord.Disposed = $true
        }
    } catch { throw 'GH_CHILD_DRAIN' }
}
