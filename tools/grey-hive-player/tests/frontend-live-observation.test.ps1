$ErrorActionPreference='Stop'
Set-StrictMode -Version Latest
$script:Checks=0;$script:Failures=0;$script:Creates=0;$script:Disposals=0;$script:Collision=$false;$script:IoFailure=$false;$script:DisposeFailure=$false;$script:Existing='KEEP'
function Check([bool]$Value,[string]$Label){$script:Checks++;if(-not $Value){$script:Failures++;Write-Output "FAIL $Label"}}
trap{$script:Checks++;$script:Failures++;Write-Output "FRONTEND_PRODUCER checks=$script:Checks failures=$script:Failures FATAL=$($_.Exception.Message)";throw}
$path=Join-Path $PSScriptRoot '../process.ps1';$tokens=$null;$errors=$null
$ast=[Management.Automation.Language.Parser]::ParseFile($path,[ref]$tokens,[ref]$errors)
Check ($errors.Count -eq 0) 'full changed production parser'
function Definition([string]$Name){$nodes=@($ast.FindAll({param($n)$n -is [Management.Automation.Language.FunctionDefinitionAst] -and $n.Name -ceq $Name},$true));if($nodes.Count -ne 1){throw "Missing/duplicate function $Name"};return $nodes[0].Extent.Text}
Invoke-Expression (Definition 'New-GhChildEnvironment')
$source=Definition 'New-GhFrontendObservationCarrier'
$tokens=$null;$errors=$null;$producerAst=[Management.Automation.Language.Parser]::ParseInput($source,[ref]$tokens,[ref]$errors)
$opens=@($producerAst.FindAll({param($n)$n -is [Management.Automation.Language.InvokeMemberExpressionAst] -and $n.Expression.Extent.Text -ceq '[IO.File]' -and $n.Member.Extent.Text -ceq 'Open'},$true))
if($opens.Count -ne 1){throw 'Missing/duplicate reservation boundary'}
$replacement='(Open-FixtureReservation';$names=@('Path','Mode','Access','Share')
for($i=0;$i -lt $opens[0].Arguments.Count;$i++){$replacement+=' -'+$names[$i]+' ('+$opens[0].Arguments[$i].Extent.Text+')'};$replacement+=')'
$source=$source.Remove($opens[0].Extent.StartOffset,$opens[0].Extent.Text.Length).Insert($opens[0].Extent.StartOffset,$replacement)
Invoke-Expression $source
function Resolve-GhOrdinaryPath($Path,$Kind,[bool]$Missing=$false){if($Path -notlike 'C:/fixture/*' -or $Path.Contains('..')){throw 'FIXTURE_PATH_REJECT'};return $Path}
function Assert-GhContainedPath($Root,$Path){if(-not $Path.StartsWith($Root+'/',[StringComparison]::Ordinal)){throw 'FIXTURE_CONTAINMENT_REJECT'}}
function Open-FixtureReservation($Path,$Mode,$Access,$Share){
    $script:Creates++
    Check ($Mode -eq [IO.FileMode]::CreateNew -and $Access -eq [IO.FileAccess]::Write -and $Share -eq [IO.FileShare]::Read) 'reservation never truncates/adopts and promises no exclusive handoff'
    Check ($Path -ceq 'C:/fixture/attempt-A/frontend-live-observation.jsonl') 'fixed sibling from immutable Plan binding'
    if($script:Collision){throw [IO.IOException]::new('Fixture exclusive collision')}
    if($script:IoFailure){throw [IO.IOException]::new('Fixture I/O failure')}
    $stream=[pscustomobject]@{}
    $stream|Add-Member ScriptMethod Dispose {$script:Disposals++;if($script:DisposeFailure){throw [IO.IOException]::new('Fixture dispose failure')}}
    return $stream
}
$startTokens=$null;$startErrors=$null;$startAst=[Management.Automation.Language.Parser]::ParseInput((Definition 'Start-GhChild'),[ref]$startTokens,[ref]$startErrors)
$calls=@($startAst.FindAll({param($n)$n -is [Management.Automation.Language.CommandAst] -and $n.GetCommandName() -cin @('New-GhChildEnvironment','New-GhFrontendObservationCarrier')},$true)|Sort-Object {$_.Extent.StartOffset})
Check ($calls.Count -eq 2 -and $calls[0].GetCommandName() -ceq 'New-GhChildEnvironment' -and $calls[1].GetCommandName() -ceq 'New-GhFrontendObservationCarrier') 'actual Start orchestration scrubs before producer'
Invoke-Expression ('function Invoke-ActualEnvironmentCalls {param($Plan,$psi,$Kind);'+(($calls|ForEach-Object {$_.Extent.Text}) -join "`n")+'}')
$plan=@{AttemptDir='C:/fixture/attempt-A';GodotLog='C:/fixture/attempt-A/godot.log';RoamingDir='roaming';LocalDir='local';TempDir='temp'}
$planBefore=$plan.Clone()
$key='WUXIAN_GH_FRONTEND_OBSERVATION_PATH'
foreach($kind in @('Native','Godot')){foreach($optin in @('0','1','', 'true')){
    $script:Creates=0;$script:Disposals=0;$psi=[Diagnostics.ProcessStartInfo]::new()
    $psi.Environment[$key]='C:/untrusted/override';$psi.Environment['WUXIAN_GH_DIAGNOSTIC_FIRST_UTC']='private-anchor';$psi.Environment['WUXIAN_V1_INPUT_TRACE']=$optin
    Invoke-ActualEnvironmentCalls $plan $psi $kind
    $enabled=$kind -ceq 'Godot' -and $optin -ceq '1'
    Check ($psi.Environment.ContainsKey($key) -eq $enabled) "child-only trace admission $kind/$optin"
    Check ($script:Creates -eq $(if($enabled){1}else{0})) 'OFF/Native does no reservation I/O'
    Check (-not $psi.Environment.ContainsKey('WUXIAN_GH_DIAGNOSTIC_FIRST_UTC')) 'diagnostic anchors still scrubbed'
    Check ($psi.Environment['APPDATA'] -ceq 'roaming' -and $psi.Environment['LOCALAPPDATA'] -ceq 'local' -and $psi.Environment['TEMP'] -ceq 'temp') 'existing child redirects unchanged'
    if($enabled){Check ($psi.Environment[$key] -ceq 'C:/fixture/attempt-A/frontend-live-observation.jsonl' -and $script:Disposals -eq 1) 'only successful reservation supplies carrier'}
}}
foreach($failure in @('collision','open','dispose')){
    $script:Collision=$failure -ceq 'collision';$script:IoFailure=$failure -ceq 'open';$script:DisposeFailure=$failure -ceq 'dispose';$script:Creates=0
    $psi=[Diagnostics.ProcessStartInfo]::new();$psi.Environment['WUXIAN_V1_INPUT_TRACE']='1';$caught=$null
    try{Invoke-ActualEnvironmentCalls $plan $psi 'Godot'}catch{$caught=$_}
    Check ($null -eq $caught -and -not $psi.Environment.ContainsKey($key)) "$failure disables diagnostics, never child admission"
    Check ($script:Creates -eq 1 -and $script:Existing -ceq 'KEEP') 'failure has no second destination/adoption/truncation'
}
$script:Collision=$false;$script:IoFailure=$false;$script:DisposeFailure=$false
foreach($badLog in @('C:/fixture/other/godot.log','C:/fixture/attempt-A/other.log','C:/fixture/attempt-A/../godot.log')){
    $script:Creates=0;$psi=[Diagnostics.ProcessStartInfo]::new();$psi.Environment['WUXIAN_V1_INPUT_TRACE']='1';$bad=$plan.Clone();$bad.GodotLog=$badLog
    Invoke-ActualEnvironmentCalls $bad $psi 'Godot'
    Check ($script:Creates -eq 0 -and -not $psi.Environment.ContainsKey($key)) 'wrong Plan log binding performs no reservation/fallback'
}
$registration=@($startAst.FindAll({param($n)$n -is [Management.Automation.Language.InvokeMemberExpressionAst] -and $n.Expression.Extent.Text -ceq '$OwnedRegistry' -and $n.Member.Extent.Text -ceq 'Add'},$true))
$starts=@($startAst.FindAll({param($n)$n -is [Management.Automation.Language.InvokeMemberExpressionAst] -and $n.Expression.Extent.Text -ceq '$process' -and $n.Member.Extent.Text -ceq 'Start'},$true))
Check ($registration.Count -eq 1 -and $starts.Count -eq 1 -and $calls.Count -eq 2 -and $calls[1].Extent.StartOffset -lt $registration[0].Extent.StartOffset -and $registration[0].Extent.StartOffset -lt $starts[0].Extent.StartOffset) 'new producer keeps registration-before-Start order; Start itself never evaluated'
Check ($plan.Keys.Count -eq $planBefore.Keys.Count -and @($planBefore.Keys|Where-Object {-not $plan.ContainsKey($_) -or $plan[$_] -cne $planBefore[$_]}).Count -eq 0) 'producer does not mutate Plan or add destination input'
Write-Output "FRONTEND_PRODUCER checks=$script:Checks failures=$script:Failures"
if($script:Failures){exit 1};exit 0
