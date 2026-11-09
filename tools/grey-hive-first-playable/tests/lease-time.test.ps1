$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

# Extract only the maintained guard; never execute the runner's entry point.
$runnerPath = Join-Path $PSScriptRoot '../run-owned.ps1'
$tokens = $null
$parseErrors = $null
$ast = [System.Management.Automation.Language.Parser]::ParseFile($runnerPath, [ref]$tokens, [ref]$parseErrors)
if ($parseErrors.Count -ne 0) { throw "Runner parse errors: $($parseErrors -join '; ')" }
$guards = @($ast.FindAll({ param($node)
    $node -is [System.Management.Automation.Language.FunctionDefinitionAst] -and
    $node.Name -ceq 'Assert-NotExpired'
}, $true))
if ($guards.Count -ne 1) { throw 'Expected exactly one Assert-NotExpired function' }
$source = $guards[0].Extent.Text
$clockAssignment = '$now = [DateTimeOffset]::UtcNow'
if (($source.Split(@($clockAssignment), [StringSplitOptions]::None)).Count -ne 2) {
    throw 'Expected exactly one production clock assignment'
}
$source = $source.Replace($clockAssignment, '$now = $script:FixtureNow')
Invoke-Expression $source

function Stop-Run { param([int]$Status, [string]$Reason)
    throw "WXSTATUS${Status}:$Reason"
}

$expectedRejection = 'WXSTATUS64:lease or receipt has insufficient remaining bounded time'
$origin = [DateTimeOffset]::Parse('2030-01-01T00:00:00Z')
$fixtures = @(
    @{ Name = 'fullfit'; Now = 0; Hard = 7200; Reserve = 6600; Reject = $false }
    @{ Name = 'shortfit'; Now = 0; Hard = 181; Reserve = 120; Reject = $false }
    @{ Name = '180sec'; Now = 0; Hard = 180; Reserve = 120; Reject = $true }
    @{ Name = '179sec'; Now = 0; Hard = 179; Reserve = 120; Reject = $true }
    @{ Name = 'at-reserve'; Now = 6600; Hard = 7200; Reserve = 6600; Reject = $true }
    @{ Name = 'beforeFIRST'; Now = -1; Hard = 7200; Reserve = 6600; Reject = $true }
    @{ Name = 'futureissued'; Now = 0; Hard = 7200; Reserve = 6600; Issued = 1; Reject = $true }
    @{ Name = 'expired'; Now = 0; Hard = 7200; Reserve = 6600; Expiry = 0; Reject = $true }
    @{ Name = 'expiry-before-hard'; Now = 0; Hard = 7200; Reserve = 6600; Expiry = 7199; Reject = $true }
    @{ Name = 'nonUTC'; Now = 0; Hard = 7200; Reserve = 6600; NonUTC = $true; Reject = $true }
)
$failures = 0
foreach ($fixture in $fixtures) {
    $script:FixtureNow = $origin.AddSeconds($fixture.Now)
    $issuedSeconds = if ($fixture.ContainsKey('Issued')) { $fixture.Issued } else { -60 }
    $expirySeconds = if ($fixture.ContainsKey('Expiry')) { $fixture.Expiry } else { $fixture.Hard }
    $receipt = [pscustomobject]@{
        issuedUtc = $origin.AddSeconds($issuedSeconds).ToString('yyyy-MM-ddTHH:mm:ssZ')
        expiresUtc = $origin.AddSeconds($expirySeconds).ToString('yyyy-MM-ddTHH:mm:ssZ')
    }
    if ($fixture.ContainsKey('NonUTC')) { $receipt.issuedUtc = '2029-12-31T23:59:00+00:00' }
    $manifest = [pscustomobject]@{
        lease = [pscustomobject]@{
            firstUtc = $origin.ToString('yyyy-MM-ddTHH:mm:ssZ')
            reserveUtc = $origin.AddSeconds($fixture.Reserve).ToString('yyyy-MM-ddTHH:mm:ssZ')
        }
        deadlineUtc = $origin.AddSeconds($fixture.Hard).ToString('yyyy-MM-ddTHH:mm:ssZ')
        limits = [pscustomobject]@{ totalSeconds = 6000; closeSeconds = 120; drainSeconds = 60 }
    }
    $actual = $null
    try { Assert-NotExpired $receipt $manifest }
    catch { $actual = $_.Exception.Message }
    $expected = if ($fixture.Reject) { $expectedRejection } else { $null }
    if ($actual -cne $expected) {
        $failures++
        Write-Output "FAIL $($fixture.Name): expected=[$expected] actual=[$actual]"
    } else {
        Write-Output "PASS $($fixture.Name)"
    }
}
Write-Output "$($fixtures.Count - $failures)/$($fixtures.Count) fixtures passed"
if ($failures -gt 0) { exit 1 }
exit 0
