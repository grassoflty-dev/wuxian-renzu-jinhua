$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$runner = Join-Path $PSScriptRoot '../process.ps1'
$tokens = $null; $errors = $null
$ast = [System.Management.Automation.Language.Parser]::ParseFile($runner,[ref]$tokens,[ref]$errors)
if ($errors.Count) { throw 'Production syntax failed' }
function Get-FunctionAst([string]$Name) {
    $found = @($ast.FindAll({ param($node) $node -is [System.Management.Automation.Language.FunctionDefinitionAst] -and $node.Name -ceq $Name },$true))
    if ($found.Count -ne 1) { throw "Expected unique $Name" }
    return $found[0]
}
$reader = (Get-FunctionAst 'Read-GhChildImagePath').Extent.Text
$reader = $reader.Replace('[Diagnostics.Stopwatch]::GetTimestamp()', '(Get-FixtureTimestamp)').Replace('[Threading.Thread]::Sleep(', 'Invoke-FixtureSleep(')
Invoke-Expression $reader
function Get-FixtureTimestamp { return $script:FixtureTicks }
function Invoke-FixtureSleep([int]$Milliseconds) { $script:FixtureTicks += [long]($Milliseconds * [Diagnostics.Stopwatch]::Frequency / 1000.0) }
# Keep the actual admission/path/image rejection prefix; no CIM or real process call.
$identity = Get-FunctionAst 'Assert-GhChild'
$try = @($identity.Body.EndBlock.Statements | Where-Object { $_ -is [System.Management.Automation.Language.TryStatementAst] })[0]
$prefix = @()
foreach ($statement in $try.Body.Statements) {
    if ($statement.Extent.Text.StartsWith('$remaining =')) { break }
    $prefix += $statement.Extent.Text.Replace('[Diagnostics.Stopwatch]::GetTimestamp()', '(Get-FixtureTimestamp)')
}
$admission = [scriptblock]::Create('param($OwnedRecord,$Deadline)' + "`n" + ($prefix -join "`n"))
. (Join-Path $PSScriptRoot '../paths.ps1')
$validPath = [Environment]::ProcessPath.Replace('\','/')
$wrongPath = [IO.Path]::GetFullPath($runner).Replace('\','/')
$birth = [datetime]::SpecifyKind([datetime]'2030-01-01T00:00:00', [DateTimeKind]::Utc)
$cases = @('null-then-valid','empty-then-valid','persistent-null','persistent-empty','wrong-image','exit-during-wait','birth-change','pid-change','observation-exception')
foreach ($case in $cases) {
    $script:FixtureTicks = 0L
    $process = [pscustomobject]@{ Id=71; HasExited=$false; StartTime=$birth; RefreshCount=0; Case=$case; ValidPath=$validPath; WrongPath=$wrongPath }
    $process | Add-Member ScriptMethod Refresh {
        if ($this.Case -eq 'observation-exception') { throw [InvalidOperationException]::new('GH_FIXTURE_OBSERVATION') }
        $this.RefreshCount++
        if ($this.RefreshCount -eq 2) {
            if ($this.Case -eq 'exit-during-wait') { $this.HasExited=$true }
            if ($this.Case -eq 'birth-change') { $this.StartTime=$this.StartTime.AddSeconds(1) }
            if ($this.Case -eq 'pid-change') { $this.Id=72 }
        }
    }
    $process | Add-Member ScriptProperty MainModule {
        if ($this.Case -eq 'empty-then-valid' -and $this.RefreshCount -eq 1) { return [pscustomobject]@{FileName=''} }
        if ($this.Case -eq 'persistent-null' -or ($this.RefreshCount -eq 1 -and $this.Case -ne 'wrong-image')) { return $null }
        if ($this.Case -eq 'persistent-empty') { return [pscustomobject]@{FileName=''} }
        return [pscustomobject]@{FileName=$(if ($this.Case -eq 'wrong-image') { $this.WrongPath } else { $this.ValidPath })}
    }
    $owned = [pscustomobject]@{Process=$process;Pid=71;BirthTicks=$birth.Ticks;Disposed=$false;Started=$true;HandleCaptured=$true;Image=$validPath}
    $deadline = [long]([Diagnostics.Stopwatch]::Frequency / 10)
    $expected = switch ($case) {
        { $_ -in 'persistent-null','persistent-empty' } { 'GH_CHILD_IMAGE_UNAVAILABLE' }
        'wrong-image' { 'GH_CHILD_IMAGE' }
        { $_ -in 'exit-during-wait','birth-change','pid-change' } { 'GH_CHILD_IDENTITY' }
        'observation-exception' { 'Exception calling "Refresh" with "0" argument(s): "GH_FIXTURE_OBSERVATION"' }
        default { $null }
    }
    $actual = $null
    try { & $admission $owned $deadline } catch { $actual=$_.Exception.Message }
    if ($actual -cne $expected) { throw "FAIL ${case}: expected=[$expected] actual=[$actual]" }
    if ($case -in 'persistent-null','persistent-empty' -and $script:FixtureTicks -lt $deadline) { throw 'Premature deadline rejection' }
    if ($case -eq 'wrong-image' -and $process.RefreshCount -ne 1) { throw 'Wrong image was retried' }
    Write-Output "PASS $case"
}
Write-Output '9/9 pure image-path cases passed; no child/profile calls'
