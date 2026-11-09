$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$script:Checks = 0; $script:Failures = 0; $script:Frequency = 1000L
$script:Anchor = [DateTimeOffset]'2026-10-26T00:00:00Z'
$script:Tick = 0L; $script:WallOffset = 0.0; $script:Jump = 0L
$script:EnvFirst = $null; $script:EnvHard = $null; $script:PlanCalls = 0
$script:DrainDelay = 0L; $script:DrainDeadlines = @(); $script:Receipts = 0; $script:WindowCloses = 0
$script:RegisteredBeforeStart = $false; $script:ThrowAfterStart = $false; $script:NextPid = 9000
$script:FactoryAdvance = 0L
function Check([bool]$Value, [string]$Label) {
    $script:Checks++
    if (-not $Value) { $script:Failures++; Write-Output "FAIL $Label" }
}
function Test-RejectionReason([Exception]$Exception, [string]$ExpectedReason, [bool]$Wrapped = $false) {
    if ($null -eq $Exception) { return $false }
    if ($Wrapped) {
        if ($Exception -isnot [System.Management.Automation.MethodInvocationException] -or $null -eq $Exception.InnerException) { return $false }
        $Exception = $Exception.InnerException
        while ($null -ne $Exception.InnerException) { $Exception = $Exception.InnerException }
    }
    return $Exception.Message -ceq $ExpectedReason
}
function Reject([scriptblock]$Action, [string]$Label, [string]$ExpectedReason, [bool]$Wrapped = $false) {
    $rejected = $false
    $script:LastRejection = ''
    try { $null = & $Action } catch {
        $script:LastRejection = $_.Exception.Message
        $rejected = Test-RejectionReason $_.Exception $ExpectedReason $Wrapped
        if (-not $rejected) { Write-Output ("UNEXPECTED_REJECTION {0}: {1}" -f $Label, $_.Exception.ToString()) }
    }
    Check $rejected $Label
    if ($rejected) { Write-Output ("REJECTION {0}: {1}" -f $Label, $script:LastRejection.Substring(0,[Math]::Min(240,$script:LastRejection.Length))) }
}
function Reset-Clock([long]$At = 0) {
    $script:Tick = $At; $script:WallOffset = 0; $script:Jump = 0
    $script:DrainDelay = 0; $script:DrainDeadlines = @(); $script:Receipts = 0; $script:WindowCloses = 0
}
function Get-FixtureTick { return $script:Tick }
function Get-FixtureEnvironment([string]$Name) {
    if ($Name -ceq 'WUXIAN_GH_DIAGNOSTIC_FIRST_UTC') { return $script:EnvFirst }
    if ($Name -ceq 'WUXIAN_GH_DIAGNOSTIC_HARD_UTC') { return $script:EnvHard }
    throw 'Unexpected environment boundary'
}
function Convert-ClockBoundary([string]$Text) {
    return $Text.Replace('[Diagnostics.Stopwatch]::GetTimestamp()', '(Get-FixtureTick)').Replace('[Diagnostics.Stopwatch]::Frequency', '$script:Frequency').Replace('[Threading.Thread]::Sleep(', 'Invoke-FixtureSleep(')
}
function Invoke-FixtureSleep([int]$Milliseconds) { $script:Tick += $Milliseconds }
function Start-Sleep { param([int]$Milliseconds); Invoke-FixtureSleep $Milliseconds }
$root = Join-Path $PSScriptRoot '..'
$asts = @{}; $texts = @{}
foreach ($name in @('play-portable.ps1','launch.ps1','process.ps1','lifetime.ps1')) {
    $tokens = $null; $errors = $null
    $asts[$name] = [Management.Automation.Language.Parser]::ParseFile((Join-Path $root $name), [ref]$tokens, [ref]$errors)
    $texts[$name] = $asts[$name].Extent.Text
    Check ($errors.Count -eq 0) "full production syntax $name"
}
function Definition([string]$File, [string]$Name) {
    $found = @($asts[$File].FindAll({ param($n) $n -is [Management.Automation.Language.FunctionDefinitionAst] -and $n.Name -ceq $Name }, $true))
    if ($found.Count -ne 1) { throw "Missing unique actual helper $Name" }
    return $found[0].Extent.Text
}
# Evaluate definitions only. No entry script, profile, interop, process or CIM is executed.
foreach ($node in $asts['lifetime.ps1'].FindAll({ param($n) $n -is [Management.Automation.Language.FunctionDefinitionAst] }, $true)) {
    Invoke-Expression (Convert-ClockBoundary $node.Extent.Text)
}
function Get-GhDiagnosticUtcNow { return $script:Anchor.AddSeconds($script:Tick / 1000.0 + $script:WallOffset) }
Invoke-Expression (Definition 'process.ps1' 'New-GhChildEnvironment')
Invoke-Expression (Convert-ClockBoundary (Definition 'process.ps1' 'Read-GhChildImagePath'))
$identity = Convert-ClockBoundary (Definition 'process.ps1' 'Assert-GhChild')
$identity = [regex]::Replace($identity, '(?m)^\s*\$rows = @\(CimCmdlets\\Get-CimInstance[^\r\n]*', '        $rows = @(Get-FixtureCim $OwnedRecord)')
$identity = $identity.Replace('[Wuxian.Player.PlayerWindows]::SplitArguments($row.CommandLine)', '(Get-FixtureArgv $OwnedRecord)')
if ($identity.Contains('CimCmdlets\Get-CimInstance') -or $identity.Contains('[Wuxian.Player.PlayerWindows]')) { throw 'Unsubstituted external identity boundary' }
Invoke-Expression $identity
$start = (Convert-ClockBoundary (Definition 'process.ps1' 'Start-GhChild')).Replace('[Diagnostics.Process]::new()', '(New-FixtureProcess)').Replace('$process.Id', '(Get-FixtureId $process)')
if ($start.Contains('[Diagnostics.Process]::new()')) { throw 'Real process factory not substituted' }
Invoke-Expression $start
function Resolve-GhOrdinaryPath($Path, $Kind) { return $Path }
function Get-FixtureCim($Record) {
    return [pscustomobject]@{ ProcessId=$Record.Pid; CreationDate=$Record.Process.StartTime; ExecutablePath=$Record.Image; CommandLine='fixture-command' }
}
function Get-FixtureArgv($Record) { return ,([string[]]@($Record.Image) + $Record.Arguments) }
function Get-FixtureId($Process) {
    # Unlike a .NET getter, PowerShell ScriptProperty suppresses a thrown getter error.
    if (-not $Process.WasStarted) { throw 'Fixture process has not started' }
    return $Process.FixtureId
}
function New-FixtureStream {
    $s = [pscustomobject]@{}
    $s | Add-Member ScriptMethod CopyToAsync { param($sink); return [Threading.Tasks.Task]::CompletedTask }
    $s | Add-Member ScriptMethod Dispose { }
    return $s
}
function New-FixtureProcess {
    $script:Tick += $script:FactoryAdvance
    $script:NextPid++
    $stream = New-FixtureStream
    $reader = [pscustomobject]@{ BaseStream=$stream }
    $reader | Add-Member ScriptMethod Dispose { }
    $p = [pscustomobject]@{ StartInfo=$null; FixtureId=$script:NextPid; WasStarted=$false; Handle=1; StartTime=$script:Anchor.UtcDateTime
        MainModule=[pscustomobject]@{ FileName='C:/fixture/native.exe' }; StandardOutput=$reader; StandardError=$reader
        ExitAt=[long]::MaxValue; ExitCode=0; Kills=0; WasDisposed=$false }
    $p | Add-Member ScriptProperty Id { if (-not $this.WasStarted) { throw 'Not started' }; return $this.FixtureId }
    $p | Add-Member ScriptProperty HasExited { return $this.WasStarted -and $script:Tick -ge $this.ExitAt }
    $p | Add-Member ScriptMethod Start {
        $matches = @($script:ActiveRegistry | Where-Object { [object]::ReferenceEquals($_.Process,$this) -and $_.StartAttempted })
        $script:RegisteredBeforeStart = $matches.Count -eq 1
        $this.WasStarted = $true
        if ($script:ThrowAfterStart) { throw 'Fixture start threw after acquiring process' }
        return $true
    }
    $p | Add-Member ScriptMethod Refresh { }
    $p | Add-Member ScriptMethod WaitForExit {
        param([int]$Milliseconds)
        if ($script:Jump -gt 0) { $script:Tick += $script:Jump; $script:Jump=0 }
        if (-not $this.HasExited) { $script:Tick += [Math]::Min([long][Math]::Max(1,$Milliseconds), [Math]::Max(0L,[long]($this.ExitAt-$script:Tick))) }
        return $this.HasExited
    }
    $p | Add-Member ScriptMethod Kill { $this.Kills++; $this.ExitAt=$script:Tick }
    $p | Add-Member ScriptMethod CloseMainWindow { $script:WindowCloses++; return $true }
    $p | Add-Member ScriptMethod Dispose { $this.WasDisposed=$true }
    return $p
}
# Drain completion is an external boundary; deadline selection remains the actual lifetime code.
function Complete-GhChildDrain($Record, [long]$Deadline) {
    $script:DrainDeadlines += $Deadline
    $script:Tick += $script:DrainDelay
    if ($script:Tick -ge $Deadline) { throw 'Fixture drain deadline' }
    $Record.ExitCode=[int]$Record.Process.ExitCode; $Record.Exited=$true; $Record.Drained=$true; $Record.Disposed=$true
    $Record.Process.Dispose()
}
function Remove-GhReady($Ready,$Registry) { $Ready.Removed=$true }
function Complete-GhProfile($Profile,$Facts) { $script:Receipts++; $Profile.Facts=$Facts }
function Release-GhProfile($Profile) { }
function New-Registry([long]$ExitA = [long]::MaxValue, [long]$ExitB = [long]::MaxValue) {
    $registry=[Collections.Generic.List[object]]::new()
    foreach ($kind in @('Native','Godot')) {
        $p=New-FixtureProcess; $p.WasStarted=$true; $p.ExitAt=if ($kind -ceq 'Native') { $ExitA } else { $ExitB }
        $registry.Add([pscustomobject]@{ Kind=$kind; Process=$p; Started=$true; StartAttempted=$true; HandleCaptured=$true
            Pid=$p.Id; BirthTicks=$p.StartTime.Ticks; Image='C:/fixture/native.exe'; Arguments=[string[]]@(); CimBirthTicks=$null; CommandLine=$null
            StdoutTask=[Threading.Tasks.Task]::CompletedTask; StderrTask=[Threading.Tasks.Task]::CompletedTask
            Exited=$false; ExitCode=$null; Drained=$false; Disposed=$false })
    }
    return ,$registry
}
function Inputs([int]$Seconds=7200) {
    return @{ DiagnosticFirstLaunchUtc=$script:Anchor.ToString("yyyy-MM-dd'T'HH:mm:ss.fffffff'Z'"); DiagnosticHardDeadlineUtc=$script:Anchor.AddSeconds($Seconds).ToString("yyyy-MM-dd'T'HH:mm:ss.fffffff'Z'") }
}
function Exit-ForStatus([string]$status) {
    $node=@($asts['launch.ps1'].FindAll({param($n) $n -is [Management.Automation.Language.AssignmentStatementAst] -and $n.Left.Extent.Text -ceq '$exitCode'},$true))[0]
    Invoke-Expression $node.Extent.Text
    return $exitCode
}
function Finish($Registry,$Budget,$Failure=$null) {
    $plan=@{ReadyPath='fixture-ready'}
    $lease=[pscustomobject]@{ Started=$true; Plan=$plan; Facts=$null }
    $ready=[pscustomobject]@{ Path='fixture-ready'; Removed=$false; Stream=$null; Handle=$null }
    return Complete-GhPlayerAttempt $plan $lease $ready $Registry $Failure $Budget
}
# Actual portable forwarding and actual launch argument partition/budget statements only.
$portableLine=($texts['play-portable.ps1'] -split '\r?\n' | Where-Object { $_.TrimStart().StartsWith('$diagnosticArguments = Resolve-GhDiagnosticArguments') })
$forwardLine=($texts['play-portable.ps1'] -split '\r?\n' | Where-Object { $_.TrimStart().StartsWith('& $launcher -PwshExe') })
$partition=($texts['launch.ps1'] -split '\r?\n' | Where-Object { $_.StartsWith('$parameters = @{}; foreach') })
$launchBudget=($texts['launch.ps1'] -split '\r?\n' | Where-Object { $_.TrimStart().StartsWith('$diagnosticArguments = Resolve-GhDiagnosticArguments') -or $_.TrimStart().StartsWith('$budget = New-GhDiagnosticBudget') }) -join "`n"
$portableLine=$portableLine.Replace('[Environment]::GetEnvironmentVariable(', '(Get-FixtureEnvironment ')
$launchBudget=$launchBudget.Replace('[Environment]::GetEnvironmentVariable(', '(Get-FixtureEnvironment ')
$launcherBody=@'
function Invoke-FixtureLauncher {
param($PwshExe,$NativeExe,$NativeSha256,$GodotExe,$GodotSha256,$ProjectDir,$ProfileRoot,$AttemptDir,[string]$DiagnosticFirstLaunchUtc,[string]$DiagnosticHardDeadlineUtc)
__PARTITION__
__BUDGET__
$script:PlanCalls++
$script:ForwardPlan=$parameters
$script:ForwardBudget=$budget
}
function Invoke-FixturePortable {
param([string]$DiagnosticFirstLaunchUtc,[string]$DiagnosticHardDeadlineUtc)
$launcher='Invoke-FixtureLauncher'; $pwsh='pwsh'; $native='native'; $godot='godot'; $project='project'; $profile='profile'; $attempt='attempt'; $hashes=@{NativeSha256='hash';GodotSha256='hash'}
__RESOLVE__
__FORWARD__
}
'@
Invoke-Expression ($launcherBody.Replace('__PARTITION__',$partition).Replace('__BUDGET__',$launchBudget).Replace('__RESOLVE__',$portableLine).Replace('__FORWARD__',$forwardLine))

try { throw 'Unrelated immediate failure' } catch {
    Check (-not (Test-RejectionReason $_.Exception 'GH_LIFETIME_PEER_TIMEOUT')) 'unrelated immediate exception cannot qualify peer timeout'
}
$wrappedWrong = [System.Management.Automation.MethodInvocationException]::new('Start wrapper', [Exception]::new('Unrelated start failure'))
Check (-not (Test-RejectionReason $wrappedWrong 'Fixture start threw after acquiring process' $true)) 'wrong wrapped inner cause cannot qualify partial Start'
Check (-not (Test-RejectionReason ([Exception]::new('Fixture start threw after acquiring process')) 'Fixture start threw after acquiring process' $true)) 'unwrapped matching text cannot qualify wrapped Start'
Reset-Clock
Check ($null -eq (New-GhDiagnosticBudget @{})) 'absent pair keeps default unbounded'
Check ((Resolve-GhDiagnosticArguments @{} $null $null).Count -eq 0) 'absent environment forwards nothing'
$inputs=Inputs
Reject { Resolve-GhDiagnosticArguments @{} $inputs.DiagnosticFirstLaunchUtc $null } 'partial environment pair rejects' 'GH_DIAGNOSTIC_PAIR'
Reject { Resolve-GhDiagnosticArguments @{DiagnosticFirstLaunchUtc=$inputs.DiagnosticFirstLaunchUtc} $null $null } 'partial explicit pair rejects' 'GH_DIAGNOSTIC_PAIR'
Reject { Resolve-GhDiagnosticArguments @{DiagnosticFirstLaunchUtc='';DiagnosticHardDeadlineUtc=$inputs.DiagnosticHardDeadlineUtc} $null $null } 'empty explicit pair rejects' 'GH_DIAGNOSTIC_ARGUMENT'
Reject { Resolve-GhDiagnosticArguments $inputs 'conflict' $inputs.DiagnosticHardDeadlineUtc } 'conflicting environment rejects' 'GH_DIAGNOSTIC_CONFLICT'
foreach ($bad in @('2026-10-26T00:00:00Z','2026-10-26T00:00:00.0000000+00:00','not-a-date')) {
    Reject { New-GhDiagnosticBudget @{DiagnosticFirstLaunchUtc=$bad;DiagnosticHardDeadlineUtc=$inputs.DiagnosticHardDeadlineUtc} } 'noncanonical timestamp rejects' 'GH_DIAGNOSTIC_UTC'
}
Reject { New-GhDiagnosticBudget (Inputs 600) } '600 second interval rejects' 'GH_DIAGNOSTIC_INTERVAL'
Reject { New-GhDiagnosticBudget (Inputs 7201) } 'overbound interval rejects' 'GH_DIAGNOSTIC_INTERVAL'
Reset-Clock -1
Reject { New-GhDiagnosticBudget $inputs } 'future FIRST rejects' 'GH_DIAGNOSTIC_INTERVAL'
Reset-Clock 7200000
Reject { New-GhDiagnosticBudget $inputs } 'expired HARD rejects' 'GH_DIAGNOSTIC_INTERVAL'
Reset-Clock
$script:EnvFirst=$inputs.DiagnosticFirstLaunchUtc; $script:EnvHard=$inputs.DiagnosticHardDeadlineUtc
Invoke-FixturePortable
$a=$script:ForwardBudget
Check ($script:ForwardPlan.Count -eq 8 -and -not $script:ForwardPlan.ContainsKey('DiagnosticFirstLaunchUtc')) 'actual forwarding preserves strict eight-key plan'
Check ($a.FirstLaunchUtc -ceq $inputs.DiagnosticFirstLaunchUtc -and $a.HardDeadlineUtc -ceq $inputs.DiagnosticHardDeadlineUtc) 'actual env forwarding preserves both anchors'
Reset-Clock 1000000
Invoke-FixturePortable
$b=$script:ForwardBudget
Check ($b.HardTick - $script:Tick -lt $a.HardTick) 'B receives reduced remaining time'
Check ($a.HardDeadlineUtc -ceq $b.HardDeadlineUtc -and $a.FirstLaunchUtc -ceq $b.FirstLaunchUtc) 'A B share identical supplied anchors'
$calls=$script:PlanCalls
Reject { Invoke-FixturePortable -DiagnosticFirstLaunchUtc 'conflict' -DiagnosticHardDeadlineUtc $inputs.DiagnosticHardDeadlineUtc } 'actual entry conflict rejects' 'GH_DIAGNOSTIC_CONFLICT'
Check ($script:PlanCalls -eq $calls) 'conflict cannot reach plan/profile/start'
$script:EnvFirst='bad';$script:EnvHard='bad'
Reject { Invoke-FixturePortable } 'actual forwarded malformed pair rejects before plan' 'GH_DIAGNOSTIC_UTC'
Check ($script:PlanCalls -eq $calls) 'malformed cannot reach plan/profile/start'
$script:EnvFirst=$null;$script:EnvHard=$null
Reset-Clock
Invoke-FixturePortable
Check ($null -eq $script:ForwardBudget -and $script:ForwardPlan.Count -eq 8) 'actual default forwarding remains unbounded eight keys'
$budget=New-GhDiagnosticBudget (Inputs)
$originalHard=$budget.HardTick
$script:WallOffset=-500
Update-GhDiagnosticBudget $budget
Check ($budget.HardTick -eq $originalHard) 'backward UTC cannot extend monotonic deadline'
$script:WallOffset=500
Update-GhDiagnosticBudget $budget
Check ($budget.HardTick -lt $originalHard) 'forward UTC shortens monotonic deadline'
Reset-Clock 6598000
$late=New-GhDiagnosticBudget (Inputs)
$script:ActiveRegistry=[Collections.Generic.List[object]]::new()
$plan=@{NativeExe='C:/fixture/native.exe';GodotExe='C:/fixture/godot.exe';ProjectDir='C:/fixture/project';RoamingDir='roaming';LocalDir='local';TempDir='temp'}
$child=Start-GhChild Native $plan.NativeExe @() 'C:/fixture' $plan $script:ActiveRegistry $late
Check $script:RegisteredBeforeStart 'actual child registration precedes fake Start boundary'
Check ($child.StartupDeadline -eq $late.OperationalTick -and $child.StartupDeadline - $script:Tick -eq 2000) 'late B startup90 clamped to two remaining seconds'
Check ($texts['launch.ps1'].Contains('Wait-GhReady $plan $native $token $native.StartupDeadline')) 'unchanged ready receives clamped startup deadline'
$script:ActiveRegistry=[Collections.Generic.List[object]]::new()
$script:FactoryAdvance=2000
Reject { Start-GhChild Native $plan.NativeExe @() 'C:/fixture' $plan $script:ActiveRegistry $late } 'cutoff reached during setup rejects immediately before Start' 'GH_DIAGNOSTIC_TIMEOUT'
$script:FactoryAdvance=0
Check ($script:ActiveRegistry.Count -eq 1 -and -not $script:ActiveRegistry[0].StartAttempted -and -not $script:ActiveRegistry[0].Started) 'registered pre-start failure does not invoke process Start'
$script:Tick=6600000
Reject { Assert-GhDiagnosticBudget $late -ForStart } 'reserve cutoff rejects another child start' 'GH_DIAGNOSTIC_TIMEOUT'
Check $late.Failed 'cutoff failure sticky'
Reset-Clock 6600000
Reject { New-GhDiagnosticBudget (Inputs) } 'late B entry rejected before profile' 'GH_DIAGNOSTIC_INTERVAL'
Reset-Clock
$psi=[Diagnostics.ProcessStartInfo]::new()
$psi.Environment['WUXIAN_GH_DIAGNOSTIC_FIRST_UTC']=$inputs.DiagnosticFirstLaunchUtc
$psi.Environment['WUXIAN_GH_DIAGNOSTIC_HARD_UTC']=$inputs.DiagnosticHardDeadlineUtc
New-GhChildEnvironment $plan $psi
Check (-not $psi.Environment.ContainsKey('WUXIAN_GH_DIAGNOSTIC_FIRST_UTC') -and -not $psi.Environment.ContainsKey('WUXIAN_GH_DIAGNOSTIC_HARD_UTC')) 'existing child scrub removes diagnostic anchors'
$registry=New-Registry 7201000 7201000
$script:Jump=7201000
Wait-GhPlayerLifetime $registry
Check ($script:Tick -gt 7200000 -and (Finish $registry $null) -ceq 'GH_PLAYER_NATURAL_CLOSURE') 'default healthy lifetime remains unbounded beyond7200'
Check ($script:Receipts -eq 1) 'natural default retains closure receipt gate'
Reset-Clock
$budget=New-GhDiagnosticBudget (Inputs 610)
$registry=New-Registry 100 200
Wait-GhPlayerLifetime $registry $budget
Check ((Finish $registry $budget) -ceq 'GH_PLAYER_NATURAL_CLOSURE') 'diagnostic natural closure before cutoff'
Check ((Exit-ForStatus 'GH_PLAYER_NATURAL_CLOSURE') -eq 0) 'actual exit switch natural0'
Reset-Clock
$budget=New-GhDiagnosticBudget (Inputs 610)
$registry=New-Registry
Reject { Wait-GhPlayerLifetime $registry $budget } 'healthy diagnostic stops at operational cutoff' 'GH_DIAGNOSTIC_TIMEOUT'
Check ($script:LastRejection -ceq 'GH_DIAGNOSTIC_TIMEOUT') 'cutoff rejects for diagnostic timeout, not unrelated exception'
Write-Output "CUTOFF_OBSERVED tick=$script:Tick failed=$($budget.Failed) operational=$($budget.OperationalTick)"
Check ($budget.Failed -and $script:Tick -le 10000) 'wait cutoff bounded and sticky'
$status=Finish $registry $budget $true
Check ($status -ceq 'GH_PLAYER_UNCERTAIN_CLOSURE' -and (Exit-ForStatus $status) -eq 70) 'forced mocked exits0 remain uncertain70'
Check ($registry[0].Process.Kills -eq 1 -and $registry[1].Process.Kills -eq 1 -and $script:Receipts -eq 0) 'verified retained processes only; no clean receipt'
Check ($script:WindowCloses -eq 0) 'diagnostic failure never calls CloseMainWindow'
foreach ($record in $registry) { $record.NaturalExit=$true; $record.CloseRequested=$false }
Check ((Finish $registry $budget $null) -ceq 'GH_PLAYER_UNCERTAIN_CLOSURE' -and $script:Receipts -eq 0) 'sticky failed budget independently prevents mocked natural0 receipt upgrade'
Reset-Clock
$budget=New-GhDiagnosticBudget (Inputs 610)
$registry=New-Registry
$registry[0].Pid=1
$registry[1].BirthTicks=1
Close-GhOwnedFailure $registry $budget
Check ($registry[0].Process.Kills -eq 0 -and $registry[1].Process.Kills -eq 0) 'wrong PID/birth identity never terminated'
Check ($budget.Failed -and $script:WindowCloses -eq 0) 'identity rejection stays unknown and avoids UI'
Reset-Clock
$budget=New-GhDiagnosticBudget (Inputs 610)
$registry=New-Registry 9000 60000
Reject { Wait-GhPlayerLifetime $registry $budget } 'peer timeout after first natural exit' 'GH_LIFETIME_PEER_TIMEOUT'
Check ($script:Tick -le 39000) 'peer deadline is first observed exit plus30 not refreshed'
Reset-Clock
$budget=New-GhDiagnosticBudget (Inputs 610)
$registry=New-Registry 100 200
$script:DrainDelay=30000
Reject { Wait-GhPlayerLifetime $registry $budget } 'blocked drain respects shared deadline' 'Fixture drain deadline'
Check ($script:DrainDeadlines.Count -eq 1) 'no fresh allowance after failed drain'
Reset-Clock
$budget=New-GhDiagnosticBudget (Inputs 610)
$registry=New-Registry
$script:Tick=609000
$registry[0].Process.ExitAt=609500;$registry[1].Process.ExitAt=609500
# A natural-exit observation already occurred before cutoff in the caller's wait; clamp helpers directly bind the remaining hard reserve.
Check ((Get-GhDiagnosticDeadline $budget ($script:Tick+30000)) -eq 610000) 'peer/drain/cleanup proposal clamps to common HARD'
Close-GhOwnedFailure $registry $budget
Check (@($script:DrainDeadlines | Where-Object {$_ -gt 610000}).Count -eq 0) 'all failure drains share original hard cap'
Reset-Clock
$budget=New-GhDiagnosticBudget (Inputs 610)
$registry=New-Registry
$script:Tick=610000
Close-GhOwnedFailure $registry $budget
Check ($registry[0].Process.Kills -eq 0 -and $script:Tick -eq 610000) 'hard exhausted cleanup does not wait or kill unverified process'
Check ((Finish $registry $budget $true) -ceq 'GH_PLAYER_UNCERTAIN_CLOSURE') 'hard exhaustion cannot become natural'
Reset-Clock
$budget=New-GhDiagnosticBudget (Inputs 610)
$script:ActiveRegistry=[Collections.Generic.List[object]]::new();$script:ThrowAfterStart=$true
Reject { Start-GhChild Native $plan.NativeExe @() 'C:/fixture' $plan $script:ActiveRegistry $budget } 'partial Start exception retained' 'Fixture start threw after acquiring process' $true
$script:ThrowAfterStart=$false
Check ($script:ActiveRegistry.Count -eq 1 -and $script:ActiveRegistry[0].Started) 'partial start record retains exact process observation'
Close-GhOwnedFailure $script:ActiveRegistry $budget
Check ($script:ActiveRegistry[0].Process.Kills -eq 1 -and $budget.Failed) 'partial start uses verified diagnostic failure cleanup'
$empty=[Collections.Generic.List[object]]::new()
$status=Complete-GhPlayerAttempt $null $null $null $empty $true $null
Check ($status -ceq 'GH_PLAYER_PRESTART_REJECTED' -and (Exit-ForStatus $status) -eq 64) 'actual prestart disposition64 preserved'
Write-Output "DIAGNOSTIC_DEADLINE checks=$script:Checks failures=$script:Failures"
if ($script:Failures) { exit 1 }
exit 0
