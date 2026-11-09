[CmdletBinding()]
param()
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$game = 'C:/Users/GWL/Desktop/wuxian-renzu-jinhua'
$evidence = 'C:/Users/GWL/.codex/visualizations/2026/10/24/GH-C7-MAINTAINED-CONTROLLER-CHECK-20261024-001'
$out = "$evidence/adapter-dependency-check"
$image = 'C:/Users/GWL/WuxianEngineering/tools/godot-4.7.2-20261009/main-resume-extracted-20261009T135429Z_5f3fbf30/Godot_v4.7.2-stable_win64.exe'
$argv = @('--headless','--path',"$game/apps/godot_playable",'--check-only','--script','res://tests/campaign/grey_hive_first_playable.gd','--log-file',"$out/engine.log")
if ($argv.Count -ne 8) { throw 'ARGUMENT_COUNT' }
if (Test-Path -LiteralPath $evidence) { throw 'OUTPUT_COLLISION' }
$ancestor = Split-Path -Path $evidence -Parent
while ($ancestor) {
    if ((Test-Path -LiteralPath $ancestor) -and ((Get-Item -LiteralPath $ancestor -Force).Attributes -band [IO.FileAttributes]::ReparsePoint)) { throw 'OUTPUT_REPARSE' }
    $ancestor = Split-Path -Path $ancestor -Parent
}
if ((Get-Item -LiteralPath $image).Length -ne 180858888 -or (Get-FileHash -LiteralPath $image -Algorithm SHA256).Hash.ToLowerInvariant() -cne 'ab1824f85bfd8e0e4128182c000c4003a3e042245b2967848d089b2a04b22424') { throw 'TOOL_PIN' }
$first = [DateTime]::UtcNow
$hard = $first.AddSeconds(300)
$si = [Diagnostics.ProcessStartInfo]::new()
$si.FileName = $image; $si.WorkingDirectory = "$game/apps/godot_playable"
$si.UseShellExecute = $false; $si.CreateNoWindow = $true; $si.RedirectStandardOutput = $true; $si.RedirectStandardError = $true
foreach ($argument in $argv) { $si.ArgumentList.Add($argument) }
foreach ($key in @($si.Environment.Keys)) { if ($key -like 'WUXIAN_GH_*') { [void]$si.Environment.Remove($key) } }
$si.Environment['APPDATA'] = "$out/profile/roaming"; $si.Environment['LOCALAPPDATA'] = "$out/profile/local"
$si.Environment['TEMP'] = "$out/profile/temp"; $si.Environment['TMP'] = "$out/profile/temp"
$proc = [Diagnostics.Process]::new(); $proc.StartInfo = $si
$cancel = [Threading.CancellationTokenSource]::new()
$state = @{ Error=$null; Errors=[Collections.Generic.List[string]]::new(); EOF=@($false,$false); Observed=@(0L,0L); Written=@(0L,0L); Truncated=@($false,$false); Readers=@($null,$null); Tasks=@($null,$null); Buffers=@([byte[]]::new(8192),[byte[]]::new(8192)); Logs=@($null,$null); Disposed=@($false,$false); TaskSettled=@($false,$false) }
function Add-Failure([string]$code) { if ($null -eq $state.Error) { $state.Error=$code }; if ($state.Errors.Count -lt 16) { $state.Errors.Add($code) } }
function Get-Exited {
    try { return [bool]$proc.HasExited } catch { Add-Failure 'EXIT_QUERY_UNKNOWN'; return $null }
}
function Pump-Pipes {
    for ($i=0; $i -lt 2; $i++) {
        if ($state.EOF[$i] -or $null -eq $state.Tasks[$i] -or -not $state.Tasks[$i].IsCompleted) { continue }
        try {
            $count = $state.Tasks[$i].GetAwaiter().GetResult(); $state.Tasks[$i]=$null
            if ($count -eq 0) { $state.EOF[$i]=$true; continue }
            $state.Observed[$i] += $count
            $keep = [int][Math]::Min($count,[Math]::Max(0,1048576-$state.Written[$i]))
            if ($keep -gt 0) {
                try { $state.Logs[$i].Write($state.Buffers[$i],0,$keep); $state.Written[$i]+=$keep } catch { Add-Failure 'LOG_WRITE_FAILED'; $state.Truncated[$i]=$true }
            }
            if ($state.Observed[$i] -gt 1048576) { $state.Truncated[$i]=$true; Add-Failure 'PIPE_OVERFLOW' }
            $state.Tasks[$i]=$state.Readers[$i].ReadAsync($state.Buffers[$i],0,8192,$cancel.Token)
        } catch { Add-Failure 'PIPE_READ_FAILED'; $state.Tasks[$i]=$null }
    }
}
$started=$false; $forced=$false; $killAttempted=$false; $exited=$null; $numericExit=$null; $exitUTC=$null
$childId=$null; $birthUTC=$null; $actualImage=$null; $handle=$null; $watch=$null; $startUTC=$null
$identity=[ordered]@{ schema='wuxian.check-image-diagnostic/v1'; query='Process.MainModule.FileName'; result='not-reached'; moduleObjectNull=$null; fileNameNull=$null; hasExitedBefore=$null; hasExitedAfter=$null; exceptionType=$null; hresult=$null; elapsedMs=$null; parentArchitecture=[Runtime.InteropServices.RuntimeInformation]::ProcessArchitecture.ToString(); osArchitecture=[Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString() }
try {
    foreach ($name in @('profile/temp','profile/roaming','profile/local')) { [void][IO.Directory]::CreateDirectory("$out/$name") }
    for ($i=0;$i -lt 2;$i++) { $name=@('stdout','stderr')[$i]; $state.Logs[$i]=[IO.File]::Open("$out/$name.log",[IO.FileMode]::CreateNew,[IO.FileAccess]::Write,[IO.FileShare]::Read) }
    $startUTC=[DateTime]::UtcNow.ToString('o'); $watch=[Diagnostics.Stopwatch]::StartNew()
    $started=$proc.Start(); if (-not $started) { throw 'START_FALSE' }
    # Start BOTH pipes before Handle/Id/StartTime/MainModule metadata can fail.
    for ($i=0;$i -lt 2;$i++) {
        try { $state.Readers[$i]=@($proc.StandardOutput.BaseStream,$proc.StandardError.BaseStream)[$i]; $state.Tasks[$i]=$state.Readers[$i].ReadAsync($state.Buffers[$i],0,8192,$cancel.Token) } catch { Add-Failure 'PIPE_START_FAILED' }
    }
    $handle=$proc.Handle; $childId=$proc.Id; $birthUTC=$proc.StartTime.ToUniversalTime().ToString('o')
    $identity.hasExitedBefore=Get-Exited
    try {
        $module=$proc.MainModule; $identity.moduleObjectNull=($null -eq $module)
        if ($null -ne $module) { $actualImage=$module.FileName }
        $identity.fileNameNull=[string]::IsNullOrEmpty($actualImage)
        $identity.result=if ($null -eq $module) { 'null-module' } elseif ([string]::IsNullOrEmpty($actualImage)) { 'null-filename' } elseif ($actualImage -ine $image.Replace('/','\')) { 'path-mismatch' } else { 'matched' }
    } catch { $identity.result='query-exception'; $identity.exceptionType=$_.Exception.GetType().FullName; $identity.hresult=$_.Exception.HResult }
    $identity.hasExitedAfter=Get-Exited; $identity.elapsedMs=$watch.ElapsedMilliseconds
    # The ORIGINAL actual-image requirement stays mandatory. No retry/sleep/hash substitute.
    if ($identity.result -cne 'matched') { Add-Failure 'PROCESS_IMAGE_GATE' }
    while ($null -eq $state.Error) {
        Pump-Pipes; $exited=Get-Exited
        if ($exited -eq $true -and $state.EOF[0] -and $state.EOF[1]) { break }
        if ((Test-Path -LiteralPath "$out/engine.log") -and (Get-Item -LiteralPath "$out/engine.log").Length -gt 1048576) { Add-Failure 'ENGINE_LOG_OVERFLOW' }
        if ($watch.Elapsed.TotalSeconds -ge 90) { Add-Failure 'CHILD_TIMEOUT_90' }
        if ($null -eq $state.Error) { Start-Sleep -Milliseconds 10 }
    }
} catch { Add-Failure 'CONTROLLER_EXCEPTION'; if ($identity.result -eq 'not-reached') { $identity.exceptionType=$_.Exception.GetType().FullName; $identity.hresult=$_.Exception.HResult } }
finally {
    if ($started) {
        # SAME original close allowance: at most30 seconds, and child total at most120.
        $closeEnd=[Math]::Min(120.0,$watch.Elapsed.TotalSeconds+30.0)
        $exited=Get-Exited
        if ($exited -ne $true) {
            $killAttempted=$true
            try { if ($null -eq $handle) { throw 'RETAINED_HANDLE_UNKNOWN' }; $proc.Kill(); $forced=$true } catch { Add-Failure 'DIRECT_CLOSE_FAILED' }
        }
        do {
            Pump-Pipes; $exited=Get-Exited
            if ($exited -eq $true -and $state.EOF[0] -and $state.EOF[1]) { break }
            if ($watch.Elapsed.TotalSeconds -ge $closeEnd) { break }
            Start-Sleep -Milliseconds 10
        } while ($true)
        if ($exited -eq $true) { try { $numericExit=$proc.ExitCode; $exitUTC=$proc.ExitTime.ToUniversalTime().ToString('o') } catch { Add-Failure 'EXIT_FIELDS_UNKNOWN' } }
        else { Add-Failure 'OWNED_PROCESS_CLOSURE_UNKNOWN' }
    }
    # Cancellation/disposal is NOT EOF. Only a completed zero-byte read above establishes EOF.
    try { $cancel.Cancel() } catch { Add-Failure 'READ_CANCEL_FAILED' }
    for ($i=0;$i -lt 2;$i++) {
        try { if ($null -ne $state.Readers[$i]) { $state.Readers[$i].Dispose() }; $state.Disposed[$i]=$true } catch { Add-Failure 'READER_DISPOSE_FAILED' }
        if ($null -eq $state.Tasks[$i]) { $state.TaskSettled[$i]=$true }
        elseif ($state.Tasks[$i].IsCompleted) { try { [void]$state.Tasks[$i].GetAwaiter().GetResult() } catch {}; $state.TaskSettled[$i]=$true }
        try { if ($null -ne $state.Logs[$i]) { $state.Logs[$i].Flush(); $state.Logs[$i].Dispose() } } catch { Add-Failure 'LOG_DISPOSE_FAILED'; $state.Disposed[$i]=$false }
    }
    try { $cancel.Dispose() } catch { Add-Failure 'CANCEL_DISPOSE_FAILED' }
}
$processClosed=(-not $started -or $exited -eq $true)
$processHandleClosed=$false
if ($processClosed) { try { $proc.Dispose(); $processHandleClosed=$true } catch { Add-Failure 'PROCESS_DISPOSE_FAILED' } }
else { $script:UnclosedOwnedCheckProcess=$proc } # current-host reference only; not a claim of retention after host exit.
$allClosed=$processClosed -and $processHandleClosed -and $state.Disposed[0] -and $state.Disposed[1] -and $state.TaskSettled[0] -and $state.TaskSettled[1]
$logs=@(); $failurePattern=$false
foreach ($name in @('stdout.log','stderr.log','engine.log')) {
    $file="$out/$name"
    if (Test-Path -LiteralPath $file) {
        $item=Get-Item -LiteralPath $file
        $logs+=@{path=$file;bytes=$item.Length;sha256=(Get-FileHash -LiteralPath $file -Algorithm SHA256).Hash.ToLowerInvariant()}
        if ($item.Length -gt 1048576) { Add-Failure 'LOG_CAP_EXCEEDED' }
        elseif ((Get-Content -LiteralPath $file -Raw) -match '(?im)SCRIPT ERROR|Parse Error|compile error|Error loading|Failed to load|^ERROR:') { $failurePattern=$true }
    }
}
if (-not (Test-Path -LiteralPath "$out/engine.log" -PathType Leaf)) { Add-Failure 'ENGINE_LOG_MISSING' }
$pass=$started -and $numericExit -eq 0 -and -not $forced -and -not $killAttempted -and $null -eq $state.Error -and -not $failurePattern -and $state.EOF[0] -and $state.EOF[1] -and $allClosed
$reportedImage=if ($null -ne $actualImage -and $actualImage.Length -le 1024) { $actualImage } else { $null }
$receipt=[ordered]@{schema='wuxian.owned-script-check/v1';allocation='ADDITIONAL_C7_CONTROLLER_REPAIR_CHECK_1_OF_1';firstUTC=$first.ToString('o');reserveUTC=$first.AddSeconds(240).ToString('o');hardUTC=$hard.ToString('o');image=$image;actualImage=$reportedImage;actualImageOmitted=($null -ne $actualImage -and $actualImage.Length -gt 1024);identity=$identity;argv=$argv;cwd=$si.WorkingDirectory;pid=$childId;birthUTC=$birthUTC;startUTC=$startUTC;exitUTC=$exitUTC;cleanupFinishedUTC=[DateTime]::UtcNow.ToString('o');numericOSExit=$numericExit;forced=$forced;killAttempted=$killAttempted;processClosed=$processClosed;stdoutEOF=$state.EOF[0];stderrEOF=$state.EOF[1];observedBytes=$state.Observed;writtenBytes=$state.Written;truncated=$state.Truncated;readerDisposed=$state.Disposed;readTasksSettled=$state.TaskSettled;allRetainedChildHandlesClosed=$allClosed;error=$state.Error;errors=$state.Errors;logs=$logs;failurePattern=$failurePattern;preliminaryExecutorPass=$pass;independentAcceptance='PENDING';noInputPlanRun=$true}
$json=$receipt|ConvertTo-Json -Depth 8
$encoded=[Text.UTF8Encoding]::new($false).GetBytes($json+"`n")
if ($encoded.Length -gt 65536) { throw 'RECEIPT_CAP' }
$receiptFile=[IO.File]::Open("$out/receipt.json",[IO.FileMode]::CreateNew,[IO.FileAccess]::Write,[IO.FileShare]::Read)
try { $receiptFile.Write($encoded,0,$encoded.Length) } finally { $receiptFile.Dispose() }
$json
if (-not $pass) { exit 1 }
