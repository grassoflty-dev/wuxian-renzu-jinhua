$ErrorActionPreference='Stop'
Set-StrictMode -Version Latest
$script:Checks=0; $script:Failures=0; $script:Tick=0L; $script:Frequency=1000L
$script:Anchor=[DateTimeOffset]'2026-10-26T00:00:00Z'
$script:Files=@{}; $script:Events=[Collections.Generic.List[string]]::new(); $script:GateHeld=$false
$script:NextPid=9000; $script:StopAt=[long]::MaxValue; $script:ReadPoll=0; $script:StopPoll=[int]::MaxValue
$script:FactoryStop=$false; $script:StartStop=$false; $script:DrainPending=$false; $script:StallNative=$false
$script:Receipts=0; $script:Removed=0; $script:Released=0; $script:Context=$null; $script:ActiveRegistry=$null
$script:WindowCloseCalls=0; $script:DrainCompleteAt=[long]::MaxValue
trap { $script:Checks++;$script:Failures++;Write-Output "EARLY_STOP checks=$script:Checks failures=$script:Failures FATAL=$($_.Exception.Message)";throw }
function Check([bool]$Value,[string]$Label) { $script:Checks++; if($script:Checks -le 120 -or $script:Checks % 256 -eq 0){Write-Output "CHECK $script:Checks $Label"}; if(-not $Value){$script:Failures++; Write-Output "FAIL $Label"} }
function Reject([scriptblock]$Action,[string]$Reason,[string]$Label) {
    $caught=$null
    try { $null=& $Action } catch {$caught=$_; if($_.Exception.Message -cne $Reason){Write-Output "UNEXPECTED $Label : $($_.Exception)"}}
    Check ($null -ne $caught -and $caught.Exception.Message -ceq $Reason) $Label
}
function Ignore-Request([string]$Reason,[string]$Label) {
    $caught=$null
    try { Assert-GhStop $script:Context } catch {$caught=$_;Write-Output "UNEXPECTED $Label : $($_.Exception)"}
    Check ($null -eq $caught -and $script:Context.RequestRejected -and $script:Context.RequestOutcome -ceq 'REJECTED_CONTENT' -and $script:Context.RequestReason -ceq $Reason -and -not $script:Context.Cancelled) $Label
}
function Get-FixtureTick {return $script:Tick}
function Start-Sleep([int]$Milliseconds) {$script:Tick+=$Milliseconds}
$root=Join-Path $PSScriptRoot '..'; $asts=@{}; $texts=@{}
foreach($name in @('play-portable.ps1','launch.ps1','process.ps1','ready.ps1','lifetime.ps1','paths.ps1','profile.ps1')) {
    $tokens=$null;$errors=$null
    $asts[$name]=[Management.Automation.Language.Parser]::ParseFile((Join-Path $root $name),[ref]$tokens,[ref]$errors)
    $texts[$name]=$asts[$name].Extent.Text
    Check ($errors.Count -eq 0) "full parser $name"
}
function Definition([string]$File,[string]$Name) {
    $nodes=@($asts[$File].FindAll({param($n) $n -is [Management.Automation.Language.FunctionDefinitionAst] -and $n.Name -ceq $Name},$true))
    if($nodes.Count -ne 1){throw "Missing actual function $Name"};return $nodes[0].Extent.Text
}
function Boundaries([string]$Code) {
    $tokens=$null;$errors=$null;$ast=[Management.Automation.Language.Parser]::ParseInput($Code,[ref]$tokens,[ref]$errors)
    $calls=@($ast.FindAll({param($n)$n -is [Management.Automation.Language.InvokeMemberExpressionAst] -and $n.Expression.Extent.Text -ceq '[IO.File]' -and $n.Member.Extent.Text -cin @('Open','Exists')},$true)|Sort-Object {$_.Extent.StartOffset} -Descending)
    foreach($call in $calls){
        $names=@('Path','Mode','Access','Share');$replacement=if($call.Member.Extent.Text -ceq 'Open'){'(Open-FixtureFile'}else{'(Test-FixtureExists'}
        for($i=0;$i -lt $call.Arguments.Count;$i++){$replacement+=' -'+$names[$i]+' ('+$call.Arguments[$i].Extent.Text+')'}
        $replacement+=')';$Code=$Code.Remove($call.Extent.StartOffset,$call.Extent.Text.Length).Insert($call.Extent.StartOffset,$replacement)
    }
    return $Code.Replace('[Diagnostics.Stopwatch]::GetTimestamp()','(Get-FixtureTick)').Replace('[Diagnostics.Stopwatch]::Frequency','$script:Frequency').Replace('[IO.File]::Exists(', '(Test-FixtureExists ').Replace('[Diagnostics.Process]::GetCurrentProcess()','(Get-FixtureSelf)').Replace('[Diagnostics.Process]::new()','(New-FixtureProcess)').Replace('[Threading.Tasks.Task]::WhenAll(', '(Get-FixtureDrains ').Replace('[Threading.Thread]::Sleep(', '(Start-Sleep ')
}
# Definitions only, with external file/clock/self/process/stream boundaries substituted.
foreach($node in $asts['lifetime.ps1'].FindAll({param($n) $n -is [Management.Automation.Language.FunctionDefinitionAst]},$true)){Invoke-Expression (Boundaries $node.Extent.Text)}
Invoke-Expression (Definition 'paths.ps1' 'Read-GhJsonObject')
foreach($name in @('New-GhChildEnvironment','Read-GhChildImagePath','Complete-GhChildDrain')){Invoke-Expression (Boundaries (Definition 'process.ps1' $name))}
$identity=Boundaries (Definition 'process.ps1' 'Assert-GhChild')
$identity=[regex]::Replace($identity,'(?m)^\s*\$rows = @\(CimCmdlets\\Get-CimInstance[^\r\n]*','        $rows = @(Get-FixtureCim $OwnedRecord)')
$identity=$identity.Replace('[Wuxian.Player.PlayerWindows]::SplitArguments($row.CommandLine)','(Get-FixtureArgv $OwnedRecord)')
if($identity.Contains('CimCmdlets\') -or $identity.Contains('[Wuxian.Player.PlayerWindows]')){throw 'Unsubstituted identity boundary'}
Invoke-Expression $identity
$start=(Boundaries (Definition 'process.ps1' 'Start-GhChild')).Replace('$process.Id','(Get-FixtureId $process)')
Invoke-Expression $start
$ready=(Boundaries (Definition 'ready.ps1' 'Wait-GhReady')).Replace('[Wuxian.Player.PlayerWindows]::OpenReadPinned($path, $false)','(Open-FixtureReady)')
# All subsequent ready-pin operations are unreachable in these missing-file/cancellation cases.
Invoke-Expression $ready
function Get-GhDiagnosticUtcNow {return $script:Anchor.AddMilliseconds($script:Tick)}
function Resolve-GhOrdinaryPath($Path,$Kind,[bool]$Missing=$false) {
    if($Path -match '\.\.' -or $Path -notlike 'C:/fixture*'){throw 'GH_PATH_FIXTURE'}
    if(-not $Missing -and $Kind -eq 'File' -and -not $script:Files.ContainsKey($Path)){throw 'GH_FIXTURE_FILE_MISSING'}
    return $Path
}
function Assert-GhContainedPath($Root,$Path) {if(-not $Path.StartsWith($Root.TrimEnd('/')+'/',[StringComparison]::Ordinal)){throw 'GH_PATH_FIXTURE'}}
function Get-FixtureSelf {
    $p=[pscustomobject]@{Id=123;StartTime=$script:Anchor.UtcDateTime};$p|Add-Member ScriptMethod Dispose {};return $p
}
function Request-Bytes {
    return [Text.Encoding]::UTF8.GetBytes(([ordered]@{schema='wuxian.grey-hive.stop-request/v1';action='stop';profileId=$script:Context.Descriptor.profileId;attemptId=$script:Context.Descriptor.attemptId;controllerPid=123;controllerBirthUtc=$script:Context.Descriptor.controllerBirthUtc}|ConvertTo-Json -Compress))
}
function Test-FixtureExists($Path) {
    if($Path -like '*/stop-request.json'){
        $script:ReadPoll++
        if($script:ReadPoll -ge $script:StopPoll -or $script:Tick -ge $script:StopAt){$script:Files[$Path]=Request-Bytes}
        if($script:Files.ContainsKey($Path)){$script:Events.Add('request-present')}
    }
    return $script:Files.ContainsKey($Path)
}
function Open-FixtureFile($Path,$Mode,$Access,$Share) {
    if ($Path -is [array]) { $Path,$Mode,$Access,$Share = $Path }
    if($Path -like '*/stop.gate' -and $Mode -eq [IO.FileMode]::Open){
        if($script:GateHeld){throw [IO.IOException]::new('sharing violation',-2147024864)}
        if(-not $script:Files.ContainsKey($Path)){throw 'GH_FIXTURE_FILE_MISSING'}
        $script:GateHeld=$true;$script:Events.Add('gate-enter')
        $h=[pscustomobject]@{};$h|Add-Member ScriptMethod Dispose {$script:GateHeld=$false;$script:Events.Add('gate-exit')};return $h
    }
    if($Mode -eq [IO.FileMode]::CreateNew){
        if($script:Files.ContainsKey($Path)){throw 'GH_FIXTURE_COLLISION'}
        $script:Files[$Path]=[byte[]]@();$w=[pscustomobject]@{Path=$Path;Bytes=[byte[]]@()}
        $w|Add-Member ScriptMethod Write {param([byte[]]$bytes);$this.Bytes=$bytes}
        $w|Add-Member ScriptMethod Flush {param([bool]$disk);$script:Files[$this.Path]=$this.Bytes}
        $w|Add-Member ScriptMethod Dispose {};return $w
    }
    if(-not $script:Files.ContainsKey($Path)){throw 'GH_FIXTURE_FILE_MISSING'}
    return [IO.MemoryStream]::new([byte[]]$script:Files[$Path])
}
function Open-FixtureReady {throw [ComponentModel.Win32Exception]::new(2)}
function Get-FixtureCim($Record) {return [pscustomobject]@{ProcessId=$Record.Pid;CreationDate=$Record.Process.StartTime;ExecutablePath=$Record.Image;CommandLine='fixture'}}
function Get-FixtureArgv($Record) {return ,([string[]]@($Record.Image)+$Record.Arguments)}
function Get-FixtureId($Process){if(-not $Process.WasStarted){throw 'Fixture not started'};return $Process.FixtureId}
function New-FixtureReader {
    $base=[pscustomobject]@{};$base|Add-Member ScriptMethod CopyToAsync {param($sink);return [Threading.Tasks.Task]::CompletedTask}
    $reader=[pscustomobject]@{BaseStream=$base};$reader|Add-Member ScriptMethod Dispose {};return $reader
}
function New-FixtureProcess {
    if($script:FactoryStop){$script:Files[$script:Context.Directory+'/stop-request.json']=Request-Bytes}
    $script:NextPid++
    $p=[pscustomobject]@{FixtureId=$script:NextPid;WasStarted=$false;StartInfo=$null;StartTime=$script:Anchor.UtcDateTime;Handle=1;ExitAt=[long]::MaxValue;ExitCode=[int]0;Kills=0;MainModule=[pscustomobject]@{FileName='C:/fixture/native.exe'};StandardOutput=(New-FixtureReader);StandardError=(New-FixtureReader)}
    $p|Add-Member ScriptProperty Id {return $this.FixtureId}
    $p|Add-Member ScriptProperty HasExited {return $this.WasStarted -and $script:Tick -ge $this.ExitAt}
    $p|Add-Member ScriptMethod Start {
        if(@($script:ActiveRegistry|Where-Object{[object]::ReferenceEquals($_.Process,$this) -and $_.StartAttempted}).Count -ne 1){throw 'Missing registration'}
        $script:Events.Add('start');$this.WasStarted=$true
        if($script:StartStop){$script:StopAt=$script:Tick}
        return $true
    }
    $p|Add-Member ScriptMethod Refresh {}
    $p|Add-Member ScriptMethod Kill {$script:Events.Add('kill-'+$this.FixtureId);$this.Kills++;if(-not $script:StallNative){$this.ExitAt=$script:Tick}}
    $p|Add-Member ScriptMethod WaitForExit {param([int]$ms);$script:Events.Add('wait-'+$this.FixtureId);if(-not $this.HasExited){$script:Tick += [Math]::Min([long][Math]::Max(1,$ms),[Math]::Max(0L,$this.ExitAt-$script:Tick))};return $this.HasExited}
    $p|Add-Member ScriptMethod CloseMainWindow {$script:WindowCloseCalls++;throw 'FORBIDDEN WINDOW CLOSE'}
    $p|Add-Member ScriptMethod Dispose {$script:Events.Add('dispose-'+$this.FixtureId)};return $p
}
function Get-FixtureDrains($Tasks) {
    $d=[pscustomobject]@{IsCompletedSuccessfully=$true}
    $d|Add-Member ScriptMethod Wait {param([int]$ms);$script:Events.Add('drain');if($script:Events.Count -gt 200000){throw 'Fixture drain iteration boundary exceeded'};if($script:DrainPending){$script:Tick+=[Math]::Max(1,$ms);if($script:Tick -lt $script:DrainCompleteAt){return $false}};$script:Events.Add('drain-complete');return $true};return $d
}
function Remove-GhReady($Ready,$Registry){$script:Removed++;$Ready.Removed=$true}
function Complete-GhProfile($Profile,$Facts){$script:Receipts++}
function Release-GhProfile($Profile){$script:Released++}
$script:Plan=@{NativeExe='C:/fixture/native.exe';GodotExe='C:/fixture/godot.exe';ProjectDir='C:/fixture/project';ReadyPath='C:/fixture/run/ready.json';RunDir='C:/fixture/run';RoamingDir='r';LocalDir='l';TempDir='t';AttemptsDir='C:/fixture/profile/attempts';AttemptDir='C:/fixture/profile/attempts/11111111-1111-1111-1111-111111111111';AttemptId='11111111-1111-1111-1111-111111111111'}
function Reset-Fixture {
    $script:Tick=0L;$script:Files=@{};$script:Events.Clear();$script:GateHeld=$false;$script:ReadPoll=0;$script:StopPoll=[int]::MaxValue;$script:StopAt=[long]::MaxValue
    $script:FactoryStop=$false;$script:StartStop=$false;$script:DrainPending=$false;$script:StallNative=$false;$script:Receipts=0;$script:Removed=0;$script:Released=0
    $script:DrainCompleteAt=[long]::MaxValue
    $script:Budget=New-GhDiagnosticBudget @{DiagnosticFirstLaunchUtc='2026-10-26T00:00:00.0000000Z';DiagnosticHardDeadlineUtc='2026-10-26T02:00:00.0000000Z'}
    $lease=[pscustomobject]@{ProfileId='22222222-2222-2222-2222-222222222222'}
    $script:Context=New-GhStopContext $script:Plan $lease $script:Budget
    $script:Files['C:/fixture/native.exe']=[byte[]]@();$script:Files['C:/fixture/godot.exe']=[byte[]]@()
    $script:Files['C:/fixture/profile/active-launch.json']=[Text.Encoding]::UTF8.GetBytes('{"schema":"wuxian.grey-hive.active-launch/v1","profileId":"22222222-2222-2222-2222-222222222222","attemptId":"11111111-1111-1111-1111-111111111111"}')
    $script:ActiveRegistry=[Collections.Generic.List[object]]::new()
}
function Publish {return Send-GhStopRequest 'C:/fixture/profile' '22222222-2222-2222-2222-222222222222' '11111111-1111-1111-1111-111111111111' 123 '2026-10-26T00:00:00.0000000Z'}
function Start-Native {return Start-GhChild Native $script:Plan.NativeExe @() 'C:/fixture' $script:Plan $script:ActiveRegistry $script:Budget $script:Context}
function Registry([long]$Exit=[long]::MaxValue) {
    $r=[Collections.Generic.List[object]]::new()
    foreach($kind in @('Native','Godot')){$p=New-FixtureProcess;$p.WasStarted=$true;$p.ExitAt=$Exit;$r.Add([pscustomobject]@{Kind=$kind;Process=$p;Started=$true;StartAttempted=$true;HandleCaptured=$true;Pid=$p.Id;BirthTicks=$p.StartTime.Ticks;Image='C:/fixture/native.exe';Arguments=[string[]]@();CimBirthTicks=$null;CommandLine=$null;StdoutTask=[Threading.Tasks.Task]::CompletedTask;StderrTask=[Threading.Tasks.Task]::CompletedTask;Exited=$false;ExitCode=$null;Drained=$false;Disposed=$false})};return ,$r
}
function Finish($Registry,$Context,$Failure=$null) {
    $lease=[pscustomobject]@{Started=$true;Plan=$script:Plan};$ready=[pscustomobject]@{Path=$script:Plan.ReadyPath;Removed=$false;Stream=$null;Handle=$null}
    return Complete-GhPlayerAttempt $script:Plan $lease $ready $Registry $Failure $script:Budget $Context
}
function Exit-Code([string]$status) {
    $n=@($asts['launch.ps1'].FindAll({param($x)$x -is [Management.Automation.Language.AssignmentStatementAst] -and $x.Left.Extent.Text -ceq '$exitCode'},$true))[0]
    Invoke-Expression $n.Extent.Text;return $exitCode
}
function Both-Before-Wait([string[]]$Events,[int]$A,[int]$B) {
    $aIndex=[array]::IndexOf($Events,'kill-'+$A);$bIndex=[array]::IndexOf($Events,'kill-'+$B)
    $wait=@(for($i=0;$i -lt $Events.Count;$i++){if($Events[$i].StartsWith('wait-') -or $Events[$i] -ceq 'drain'){$i;break}})
    return $aIndex -ge 0 -and $bIndex -ge 0 -and $wait.Count -eq 1 -and $aIndex -lt $wait[0] -and $bIndex -lt $wait[0]
}
Reset-Fixture
Check ((Publish) -ceq 'REQUEST_PUBLISHED') 'publication is request-only'
Reject {Publish} 'GH_FIXTURE_COLLISION' 'request replay never overwrites'
Reject {Start-Native} 'GH_STOP_CANCELLED' 'before first start admission'
Check ($script:Context.Cancelled -and $script:ActiveRegistry.Count -eq 0 -and @($script:Events|Where-Object{$_ -ceq 'start'}).Count -eq 0) 'latched cancellation prevents first registration/start'
$deadline=$script:Context.CleanupDeadline;$script:Tick=5000;Reject {Assert-GhStop $script:Context} 'GH_STOP_CANCELLED' 'latch persists'
Check ($script:Context.CleanupDeadline -eq $deadline -and $script:Budget.FirstLaunchUtc -ceq '2026-10-26T00:00:00.0000000Z' -and $script:Budget.HardDeadlineUtc -ceq '2026-10-26T02:00:00.0000000Z') 'no renewed allowance or anchors'
foreach($key in @('profileId','attemptId','controllerPid','controllerBirthUtc')){
    Reset-Fixture;$v=(Request-Bytes|ForEach-Object{$_})
    $obj=[Text.Encoding]::UTF8.GetString([byte[]]$v)|ConvertFrom-Json -AsHashtable -DateKind String
    $obj[$key]=switch($key){'controllerPid'{456};'controllerBirthUtc'{'2026-10-26T00:00:01.0000000Z'};default{'33333333-3333-3333-3333-333333333333'}}
    $script:Files[$script:Context.Directory+'/stop-request.json']=[Text.Encoding]::UTF8.GetBytes(($obj|ConvertTo-Json -Compress))
    Ignore-Request 'GH_STOP_BINDING' "wrong binding $key"
    Check (-not $script:Context.Cancelled -and $script:Context.CleanupDeadline -eq [long]::MaxValue) "bad $key has no cancellation effects"
}
foreach($case in @(@{text='{}';reason='GH_JSON_KEYS'},@{text='{"schema":1,"schema":2}';reason='GH_JSON_DUPLICATE'},@{text='[]';reason='GH_JSON_OBJECT'})){
    Reset-Fixture;$script:Files[$script:Context.Directory+'/stop-request.json']=[Text.Encoding]::UTF8.GetBytes($case.text)
    Ignore-Request $case.reason 'strict JSON key shape'
}
Reset-Fixture
$script:FactoryStop=$true
Reject {Start-Native} 'GH_STOP_CANCELLED' 'stop arrives during setup before actual Start'
Check ($script:ActiveRegistry.Count -eq 1 -and -not $script:ActiveRegistry[0].StartAttempted -and -not $script:ActiveRegistry[0].Started) 'pre-Start registered object retained without starting'
Reset-Fixture;$native=Start-Native;$script:StopPoll=$script:ReadPoll+3
Reject {Wait-GhReady $script:Plan $native ('a'*64) $native.StartupDeadline $script:Context} 'GH_READY_INVALID' 'readiness wraps cancellation but latch survives'
Check ($script:Context.Cancelled -and $script:ActiveRegistry.Count -eq 1) 'no fabricated readiness or Godot start'
Reset-Fixture;$native=Start-Native;$null=Publish
Reject {Start-GhChild Godot $script:Plan.GodotExe @() $script:Plan.ProjectDir $script:Plan $script:ActiveRegistry $script:Budget $script:Context} 'GH_STOP_CANCELLED' 'between Native and Godot'
Check ($script:ActiveRegistry.Count -eq 1) 'no second child after admitted stop'
Reset-Fixture;$script:StartStop=$true
Reject {Start-Native} 'GH_STOP_CANCELLED' 'request during entered Start observed after registration'
Check ($script:ActiveRegistry.Count -eq 1 -and $script:ActiveRegistry[0].Started) 'already-entered Start retained for cleanup'
Reset-Fixture;$r=Registry;$script:StopAt=100
Reject {Wait-GhPlayerLifetime $r $script:Budget $script:Context} 'GH_STOP_CANCELLED' 'lifetime polling receives stop'
Check $script:Context.Cancelled 'lifetime cancellation latch'
Reset-Fixture;$r=Registry 0;$script:DrainPending=$true;$script:DrainCompleteAt=300L;$script:StopAt=100
Complete-GhChildDrain $r[0] 1000 $script:Context
Check ($script:Context.Cancelled -and $r[0].Drained -and $r[0].Disposed -and $script:Tick -eq 300) 'cancelled pending drain continues to completion before disposal'
$completed=[array]::IndexOf($script:Events.ToArray(),'drain-complete');$disposed=[array]::IndexOf($script:Events.ToArray(),'dispose-'+$r[0].Pid)
Check ($completed -ge 0 -and $disposed -ge 0 -and $completed -lt $disposed) 'completed drain event precedes disposal'
Check ($script:Context.CleanupDeadline -le 1000 -and $script:Budget.HardDeadlineUtc -ceq '2026-10-26T02:00:00.0000000Z') 'drain retains original deadline and Hard'
Complete-GhChildDrain $r[1] 1000 $script:Context
foreach($o in $r){$o|Add-Member NoteProperty NaturalExit $true}
$status=Finish $r $script:Context
Check ($status -ceq 'GH_PLAYER_UNCERTAIN_CLOSURE' -and (Exit-Code $status) -eq 70 -and $script:Receipts -eq 0) 'completed zero-exit drains still cancelled UNKNOWN70'
Reset-Fixture;$r=Registry;$null=Publish;Reject {Assert-GhStop $script:Context} 'GH_STOP_CANCELLED' 'cleanup setup admission'
$script:StallNative=$true
Close-GhOwnedFailure $r $script:Budget $script:Context
Check (Both-Before-Wait $script:Events.ToArray() $r[0].Pid $r[1].Pid) 'both termination requests before first child wait'
Check ($script:Tick -le 30000 -and $r[0].Process.Kills -eq 1 -and $r[1].Process.Kills -eq 1) 'one shared cleanup allowance'
Check (-not (Both-Before-Wait @('kill-1','wait-1','kill-2') 1 2)) 'ordering negative control rejects late second kill'
Check (-not (Both-Before-Wait @('kill-1','wait-1') 1 2)) 'ordering negative control rejects missing second event'
Reset-Fixture;$r=Registry;$r[0].BirthTicks=1;$null=Publish;Reject {Assert-GhStop $script:Context} 'GH_STOP_CANCELLED' 'invalid-peer cleanup admission'
Close-GhOwnedFailure $r $script:Budget $script:Context
Check ($r[0].Process.Kills -eq 0 -and $r[1].Process.Kills -eq 1) 'bad identity cannot prevent valid peer termination'
Reset-Fixture;$r=Registry 0;$null=Publish;Reject {Assert-GhStop $script:Context} 'GH_STOP_CANCELLED' 'sticky finalizer admission'
foreach($o in $r){$o|Add-Member NoteProperty NaturalExit $true;$o.Exited=$true;$o.ExitCode=[int]0;$o.Drained=$true;$o.Disposed=$true}
$status=Finish $r $script:Context $null
Check ($status -ceq 'GH_PLAYER_UNCERTAIN_CLOSURE' -and (Exit-Code $status) -eq 70) 'zero child exits/cleared failure never cancel cancellation'
Check ($script:Receipts -eq 0 -and $script:Removed -eq 0 -and $script:Released -eq 1) 'cancel releases resources without natural-marker retirement'
Check ($script:Files.ContainsKey('C:/fixture/profile/active-launch.json')) 'failed active marker retained'
Reject {Publish} 'GH_STOP_TERMINAL' 'late stop cannot race terminal result'
Reset-Fixture;$r=Registry 0
Wait-GhPlayerLifetime $r $script:Budget $script:Context
$status=Finish $r $script:Context
Check ($status -ceq 'GH_PLAYER_NATURAL_CLOSURE' -and (Exit-Code $status) -eq 0 -and $script:Receipts -eq 1) 'no-stop diagnostic retains natural gates'
Reject {Publish} 'GH_STOP_TERMINAL' 'natural commit excludes later request'
Reset-Fixture;$script:Budget=$null;$r=Registry 7201000
Wait-GhPlayerLifetime $r $null $null
Check ($script:Tick -ge 7201000 -and (Finish $r $null) -ceq 'GH_PLAYER_NATURAL_CLOSURE') 'ordinary null-budget remains healthy-unbounded'
Check ($null -eq (New-GhStopContext $script:Plan $null $null)) 'ordinary default publishes no descriptor'
# Execute the actual dispatcher and actual normal-Play callsite, with only exits and callees spied.
Reset-Fixture
$branch=@($asts['play-portable.ps1'].FindAll({param($n)$n -is [Management.Automation.Language.IfStatementAst] -and $n.Clauses[0].Item1.Extent.Text -ceq '$Operation -eq ''RequestStop'''},$true))
$playCall=@($asts['play-portable.ps1'].FindAll({param($n)$n -is [Management.Automation.Language.CommandAst] -and $n.CommandElements[0].Extent.Text -ceq '$launcher'},$true))
Check ($branch.Count -eq 1 -and $playCall.Count -eq 1) 'unique actual dispatcher and Play callsite'
$savedPublisher=${function:Send-GhStopRequest}
function Send-GhStopRequest {param($ProfileRoot,$ProfileId,$AttemptId,$ControllerPid,$ControllerBirthUtc);$script:Published++;return 'REQUEST_PUBLISHED'}
function Invoke-SpyPlay {param($PwshExe,$NativeExe,$NativeSha256,$GodotExe,$GodotSha256,$ProjectDir,$ProfileRoot,$AttemptDir);$script:Played++}
function Case-Variants([string]$Value){
    for($mask=0;$mask -lt (1 -shl $Value.Length);$mask++){
        $chars=$Value.ToLowerInvariant().ToCharArray()
        for($i=0;$i -lt $chars.Length;$i++){if($mask -band (1 -shl $i)){$chars[$i]=[char]::ToUpperInvariant($chars[$i])}}
        -join $chars
    }
}
if($branch.Count -eq 1 -and $playCall.Count -eq 1){
    $body=$branch[0].Extent.Text.Replace('[Console]::WriteLine($published)','$null=$published').Replace('exit 0','return')
    $locals='$launcher=''Invoke-SpyPlay''; $pwsh=''pwsh''; $native=''native''; $godot=''godot''; $project=''project''; $profile=$ProfileRoot; $attempt=''attempt''; $hashes=@{NativeSha256=''hash'';GodotSha256=''hash''};$diagnosticArguments=@{};'
    Invoke-Expression ('function Invoke-FixtureDispatcher {'+$asts['play-portable.ps1'].ParamBlock.Extent.Text+';'+$locals+$body+"`n"+$playCall[0].Extent.Text+'}')
    foreach($variant in Case-Variants 'RequestStop'){
        $script:Published=0;$script:Played=0;$beforeStarts=@($script:Events|Where-Object{$_ -ceq 'start'}).Count
        Invoke-FixtureDispatcher -Operation $variant -ProfileRoot 'C:/fixture/profile' -ProfileId '22222222-2222-2222-2222-222222222222' -AttemptId '11111111-1111-1111-1111-111111111111' -ControllerPid 123 -ControllerBirthUtc '2026-10-26T00:00:00.0000000Z'
        Check ($script:Published -eq 1 -and $script:Played -eq 0 -and @($script:Events|Where-Object{$_ -ceq 'start'}).Count -eq $beforeStarts) "ValidateSet admitted stop case $variant publishes once, never Play/Start"
    }
    foreach($variant in Case-Variants 'Play'){$script:Published=0;$script:Played=0;Invoke-FixtureDispatcher -Operation $variant -ProfileRoot 'C:/fixture/profile';Check ($script:Published -eq 0 -and $script:Played -eq 1) "Play case $variant unchanged"}
    $script:Published=0;$script:Played=0;Invoke-FixtureDispatcher -ProfileRoot 'C:/fixture/profile';Check ($script:Published -eq 0 -and $script:Played -eq 1) 'default Play unchanged'
}
Set-Item Function:Send-GhStopRequest -Value $savedPublisher
# Additional strict-wire and recovery cases; storage remains entirely in memory.
foreach($case in @(@{key='action';value='run';reason='GH_STOP_ACTION'},@{key='schema';value='other';reason='GH_STOP_ACTION'},@{key='controllerPid';value='123';reason='GH_STOP_PID'},@{key='controllerPid';value=0;reason='GH_STOP_PID'},@{key='profileId';value='BAD';reason='GH_STOP_ID'},@{key='controllerBirthUtc';value=123;reason='GH_STOP_BIRTH'},@{key='controllerBirthUtc';value='bad';reason='GH_DIAGNOSTIC_UTC'})){
    Reset-Fixture;$obj=[Text.Encoding]::UTF8.GetString((Request-Bytes))|ConvertFrom-Json -AsHashtable -DateKind String;$obj[$case.key]=$case.value
    $script:Files[$script:Context.Directory+'/stop-request.json']=[Text.Encoding]::UTF8.GetBytes(($obj|ConvertTo-Json -Compress))
    Ignore-Request $case.reason "strict wire type/value $($case.key)"
    Check (-not $script:Context.Cancelled) 'bad wire leaves cancellation latch false'
}
Reset-Fixture;$obj=[Text.Encoding]::UTF8.GetString((Request-Bytes))|ConvertFrom-Json -AsHashtable -DateKind String;$obj['command']='anything'
$script:Files[$script:Context.Directory+'/stop-request.json']=[Text.Encoding]::UTF8.GetBytes(($obj|ConvertTo-Json -Compress))
Ignore-Request 'GH_JSON_KEYS' 'extra executable authority rejected'
Reset-Fixture;$script:Files[$script:Context.Directory+'/stop-request.json']=[Text.Encoding]::UTF8.GetBytes((' '*1025))
Ignore-Request 'GH_JSON_SIZE' 'request capped at1KiB'
foreach($case in @(@{bytes=[byte[]]@(255);type=[Text.DecoderFallbackException]},@{bytes=[Text.Encoding]::UTF8.GetBytes('{');type=[Text.Json.JsonException]})){
    Reset-Fixture;$script:Files[$script:Context.Directory+'/stop-request.json']=$case.bytes
    Ignore-Request 'GH_STOP_ENCODING' 'strict UTF8/JSON rejected locally, not stop authority'
}
Reset-Fixture
Reject {Send-GhStopRequest 'C:/fixture/../other' $script:Context.Descriptor.profileId $script:Context.Descriptor.attemptId 123 $script:Context.Descriptor.controllerBirthUtc} 'GH_PATH_FIXTURE' 'outside/nonordinary profile rejected'
$active=$script:Files['C:/fixture/profile/active-launch.json'];$script:Files['C:/fixture/profile/active-launch.json']=[Text.Encoding]::UTF8.GetBytes('{"schema":"wuxian.grey-hive.active-launch/v1","profileId":"22222222-2222-2222-2222-222222222222","attemptId":"33333333-3333-3333-3333-333333333333"}')
Reject {Publish} 'GH_STOP_STALE' 'stale active attempt cannot publish'
foreach($key in @('firstLaunchUtc','hardDeadlineUtc')){
    Reset-Fixture;$path=$script:Context.Directory+'/stop-descriptor.json';$obj=[Text.Encoding]::UTF8.GetString($script:Files[$path])|ConvertFrom-Json -AsHashtable -DateKind String
    $obj[$key]=if($key -ceq 'firstLaunchUtc'){'2026-10-27T00:00:00.0000000Z'}else{'2026-10-26T00:00:00.0000000Z'}
    $script:Files[$path]=[Text.Encoding]::UTF8.GetBytes(($obj|ConvertTo-Json -Compress))
    Reject {Publish} 'GH_STOP_INTERVAL' "descriptor anchor validity $key"
    Check (-not $script:Files.ContainsKey($script:Context.Directory+'/stop-request.json')) 'invalid anchor publishes nothing'
}
Reset-Fixture;$null=Publish;Reject {Assert-GhStop $script:Context} 'GH_STOP_CANCELLED' 'negative controls actual admitted latch'
$qualify={param($cancelled,$events) $cancelled -and @($events|Where-Object{$_ -ceq 'request-present'}).Count -gt 0}
Check (& $qualify $script:Context.Cancelled $script:Events.ToArray()) 'qualification requires real latch and request observation'
Check (-not (& $qualify $false $script:Events.ToArray())) 'missing latch invalidates qualification'
Check (-not (& $qualify $true @('gate-enter','gate-exit'))) 'missing admitted-request observation invalidates qualification'
$wrong=$null;try{throw 'unrelated immediate error'}catch{$wrong=$_}
Check ($wrong.Exception.Message -cne 'GH_STOP_CANCELLED') 'unrelated immediate exception cannot qualify intended stop'
$wrapped=[Management.Automation.MethodInvocationException]::new('outer',[Exception]::new('wrong inner'))
Check ($wrapped.GetBaseException().Message -cne 'GH_STOP_CANCELLED') 'wrong wrapped cause cannot qualify stop'
Reset-Fixture;$null=Publish;Reject {Assert-GhStop $script:Context} 'GH_STOP_CANCELLED' 'hard bound cleanup admission'
$cleanup=$script:Context.CleanupDeadline;$script:Tick=7200000;$r=Registry
Close-GhOwnedFailure $r $script:Budget $script:Context
Check ($r[0].Process.Kills -eq 0 -and $r[1].Process.Kills -eq 0 -and $script:Context.CleanupDeadline -le $cleanup -and $script:Tick -eq 7200000) 'hard exhausted cleanup neither kills nor remints waits'
# Execute unchanged natural-marker guards before any pinned-file side effect is reachable.
Invoke-Expression ((Definition 'ready.ps1' 'Remove-GhReady').Replace('function Remove-GhReady','function Test-ActualRemoveGhReady'))
Invoke-Expression ((Definition 'profile.ps1' 'Complete-GhProfile').Replace('function Complete-GhProfile','function Test-ActualCompleteGhProfile'))
Reset-Fixture;$r=Registry;foreach($o in $r){$o|Add-Member NoteProperty NaturalExit $false}
Reject {Test-ActualRemoveGhReady ([pscustomobject]@{Removed=$false}) $r} 'GH_READY_REMOVE' 'actual ready guard rejects cancellation facts before pin I/O'
$lease=[pscustomobject]@{Released=$false;Started=$true;Completed=$false;JournalHandle=[pscustomobject]@{IsClosed=$false};LockStream=[pscustomobject]@{CanWrite=$true}}
Reject {Test-ActualCompleteGhProfile $lease @{nativeNaturalExit=$false;godotNaturalExit=$false;nativeExitCode=[int]0;godotExitCode=[int]0;drainsCompleted=$true;readyRemoved=$false}} 'GH_PROFILE_UNCLEAN' 'actual receipt guard rejects cancelled zero exits'
$open=Boundaries (Definition 'profile.ps1' 'Open-GhProfile')
$open=$open.Replace('[IO.File]::GetAttributes($root)','[IO.FileAttributes]::Directory').Replace('[IO.Directory]::EnumerateFileSystemEntries($root)','(Get-FixtureEntries)')
Invoke-Expression $open
function Get-FixtureEntries {return ,([string[]]@('C:/fixture/profile/profile-owner.json','C:/fixture/profile/profile.lock','C:/fixture/profile/active-launch.json'))}
$script:Files['C:/fixture/profile/profile-owner.json']=[byte[]]@();$script:Files['C:/fixture/profile/profile.lock']=[byte[]]@();$script:Plan.ProfileRoot='C:/fixture/profile'
Reject {Open-GhProfile $script:Plan} 'GH_PROFILE_UNKNOWN_OR_ACTIVE' 'actual namespace guard refuses retained failed marker'
Reset-Fixture;$script:Context.CleanupDeadline=60L;$script:GateHeld=$true
Reject {Enter-GhStopGate $script:Context} 'GH_STOP_GATE_DEADLINE' 'contended gate bounded by current common cleanup deadline'
Check ($script:Tick -eq 60 -and $script:Context.CleanupDeadline -eq 60) 'gate contention never remints deadline'
$script:GateHeld=$false
Reset-Fixture;$r=Registry 0;$script:DrainPending=$true
Reject {Complete-GhChildDrain $r[0] 200 $null} 'GH_CHILD_DRAIN' 'drain exhaustion cannot become clean'
Check (-not $r[0].Drained -and $r[0].Disposed) 'failed drain disposes retained resources without success flag'
Reset-Fixture;$r=Registry;$null=Publish;Reject {Assert-GhStop $script:Context} 'GH_STOP_CANCELLED' 'repeated cleanup admission'
$r[0].BirthTicks=1;$r[1].BirthTicks=1;$deadline=$script:Context.CleanupDeadline
Close-GhOwnedFailure $r $script:Budget $script:Context;$script:Tick+=500
Close-GhOwnedFailure $r $script:Budget $script:Context
Check ($script:Context.CleanupDeadline -le $deadline -and $r[0].Process.Kills -eq 0 -and $r[1].Process.Kills -eq 0) 'repeated invalid-identity cleanup cannot renew allowance or terminate'
$status='untrusted-status'
$whitelist=@($asts['launch.ps1'].FindAll({param($n)$n -is [Management.Automation.Language.IfStatementAst] -and $n.Clauses[0].Item1.Extent.Text.StartsWith('$status -cnotin')},$true))
Check ($whitelist.Count -eq 1) 'unique actual terminal status whitelist'
if($whitelist.Count -eq 1){Invoke-Expression $whitelist[0].Extent.Text;Check ($status -ceq 'GH_PLAYER_UNCERTAIN_CLOSURE' -and (Exit-Code $status) -eq 70) 'actual whitelist and exit mapping reject unknown disposition'}
# Invalid requests traverse the actual lifetime and finalizer without authorizing failure cleanup.
foreach($bad in @('malformed','wrong-binding')){
    Reset-Fixture;$r=Registry 100
    $bytes=if($bad -ceq 'malformed'){[Text.Encoding]::UTF8.GetBytes('{')}else{$obj=[Text.Encoding]::UTF8.GetString((Request-Bytes))|ConvertFrom-Json -AsHashtable -DateKind String;$obj.controllerPid=456;[Text.Encoding]::UTF8.GetBytes(($obj|ConvertTo-Json -Compress))}
    $script:Files[$script:Context.Directory+'/stop-request.json']=$bytes
    $caught=$null;try{Assert-GhStop $script:Context}catch{$caught=$_}
    Check ($null -eq $caught -and -not $script:Context.Cancelled -and -not $script:Budget.Failed -and $script:Receipts -eq 0 -and $script:Removed -eq 0 -and $r[0].Process.Kills -eq 0 -and $r[1].Process.Kills -eq 0) "$bad rejection has no cleanup/profile/ready side effects"
    $reason=$script:Context.RequestReason;$polls=$script:ReadPoll;Assert-GhStop $script:Context
    Check ($script:ReadPoll -eq $polls -and $script:Context.RequestReason -ceq $reason -and $script:Context.RequestOutcome -ceq 'REJECTED_CONTENT') "$bad rejection stable/no reparse authority"
    $failure=$null;try{Wait-GhPlayerLifetime $r $script:Budget $script:Context}catch{$failure=$true;Write-Output "UNEXPECTED enclosing $bad : $($_.Exception)"}
    $status=Finish $r $script:Context $failure
    Check ($null -eq $failure -and $status -ceq 'GH_PLAYER_NATURAL_CLOSURE' -and $script:Receipts -eq 1 -and $r[0].Process.Kills -eq 0 -and $r[1].Process.Kills -eq 0) "$bad healthy progression and natural finalizer remain possible"
}
Reset-Fixture;$script:GateHeld=$true;$script:Context.CleanupDeadline=40
Reject {Assert-GhStop $script:Context} 'GH_STOP_GATE_DEADLINE' 'genuine gate deadline is not a content rejection'
Check (-not $script:Context.RequestRejected -and -not $script:Context.Cancelled) 'gate/controller errors remain distinct'
$script:GateHeld=$false
Reset-Fixture;$r=Registry 0;$script:StopAt=100;$script:DrainPending=$true
Reject {Complete-GhChildDrain $r[0] 200 $script:Context} 'GH_CHILD_DRAIN' 'cancelled drain bounded exhaustion remains failure'
Check ($script:Tick -eq 200 -and $r[0].Disposed -and -not $r[0].Drained -and $script:Context.Cancelled -and $script:Context.CleanupDeadline -le 200) 'pending drain disposes only after bounded exhaustion, not cancellation alone'
# Execute actual launcher callsite ASTs with spy boundaries; preserve context object identity.
$script:ForwardCalls=[Collections.Generic.List[object]]::new();$script:ForwardContext=[object]::new()
$saved=@{};foreach($name in @('New-GhStopContext','Start-GhChild','Wait-GhReady','Wait-GhPlayerLifetime','Complete-GhPlayerAttempt')){$saved[$name]=(Get-Item ('Function:'+$name)).ScriptBlock}
function New-GhStopContext {param($Plan,$ProfileLease,$Budget);return $script:ForwardContext}
function Start-GhChild {param($Kind,$Executable,$Arguments,$WorkingDirectory,$Plan,$OwnedRegistry,$DiagnosticBudget,$StopContext);$script:ForwardCalls.Add([pscustomobject]@{stage='Start'+$Kind;context=$StopContext});return [pscustomobject]@{StartupDeadline=123L}}
function Wait-GhReady {param($Plan,$NativeOwned,$Token,$StartupDeadline,$StopContext);$script:ForwardCalls.Add([pscustomobject]@{stage='Ready';context=$StopContext});return [pscustomobject]@{}}
function Wait-GhPlayerLifetime {param($OwnedRegistry,$DiagnosticBudget,$StopContext);$script:ForwardCalls.Add([pscustomobject]@{stage='Lifetime';context=$StopContext})}
function Complete-GhPlayerAttempt {param($Plan,$ProfileLease,$ReadyLease,$OwnedRegistry,$Failure,$DiagnosticBudget,$StopContext);$script:ForwardCalls.Add([pscustomobject]@{stage='Finalize';context=$StopContext});return 'GH_PLAYER_UNCERTAIN_CLOSURE'}
$callNames=@('New-GhStopContext','Start-GhChild','Wait-GhReady','Wait-GhPlayerLifetime','Complete-GhPlayerAttempt')
$calls=@($asts['launch.ps1'].FindAll({param($n)$n -is [Management.Automation.Language.CommandAst] -and $n.GetCommandName() -cin $callNames},$true)|Sort-Object {$_.Extent.StartOffset})
Check ($calls.Count -eq 6) 'exact actual launcher callsite count'
$statements=foreach($call in $calls){$node=$call.Parent;if($node.Parent -is [Management.Automation.Language.AssignmentStatementAst]){$node=$node.Parent};$node.Extent.Text}
$orchestration='function Invoke-FixtureForwarding {param($plan,$profile,$budget,$owned,$failure);$nativeArguments=@();$godotArguments=@();$token=''fixture'';'+($statements -join "`n")+'}'
Invoke-Expression $orchestration
Invoke-FixtureForwarding $script:Plan ([pscustomobject]@{}) $script:Budget ([Collections.Generic.List[object]]::new()) $null
function Context-Forwarded($Calls,$Expected){
    $stages=@('StartNative','Ready','StartGodot','Lifetime','Finalize')
    if($Calls.Count -ne $stages.Count){return $false}
    for($i=0;$i -lt $stages.Count;$i++){if($Calls[$i].stage -cne $stages[$i] -or -not [object]::ReferenceEquals($Calls[$i].context,$Expected)){return $false}}
    return $true
}
Check (Context-Forwarded $script:ForwardCalls $script:ForwardContext) 'same actual stop context reference reaches both starts/ready/lifetime/finalization'
$dropped=@($script:ForwardCalls|ForEach-Object{[pscustomobject]@{stage=$_.stage;context=$_.context}});$dropped[1].context=$null
Check (-not (Context-Forwarded $dropped $script:ForwardContext)) 'null context forwarding negative control invalidates qualification'
Check (-not (Context-Forwarded @($script:ForwardCalls|Select-Object -Skip 1) $script:ForwardContext)) 'dropped callsite negative control invalidates qualification'
foreach($name in $saved.Keys){Set-Item ('Function:'+$name) -Value $saved[$name]}
$noWindowClose={param([int]$Count) $Count -eq 0}
Check (& $noWindowClose $script:WindowCloseCalls) 'cumulative forbidden window-close invocations remain zero across all cleanup cases'
Check (-not (& $noWindowClose 1)) 'window-close detector negative control catches even a swallowed invocation'
Write-Output "EARLY_STOP checks=$script:Checks failures=$script:Failures"
if($script:Failures){exit 1};exit 0
