[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'
$runner = Join-Path $PSScriptRoot 'identity-run.ps1'
$scriptPath = $PSCommandPath
$scratch = Join-Path $env:TEMP ('native-identity-harness-selftest-' + [guid]::NewGuid().ToString('N'))
[void](New-Item -ItemType Directory -Path $scratch -ErrorAction Stop)

function Assert-Parseable([string]$Path) {
  $tokens = $null
  $errors = $null
  [void][System.Management.Automation.Language.Parser]::ParseFile($Path, [ref]$tokens, [ref]$errors)
  if ($errors.Count -gt 0) { throw "E_SELFTEST_PARSE:${Path}:$($errors[0].Message)" }
}

function Assert-RunnerRejected([string]$Name, [string]$Exe, [string]$Hash, [string]$Evidence, [string]$ExpectedMarker) {
  $lines = @(& pwsh -NoProfile -File $runner -ExecutablePath $Exe -ExpectedSha256 $Hash -EvidenceDirectory $Evidence 2>&1)
  $code = $LASTEXITCODE
  $output = $lines -join "`n"
  if ($code -eq 0) { throw "E_SELFTEST_CASE_UNEXPECTED_SUCCESS:${Name}" }
  if ($output -notmatch [regex]::Escape($ExpectedMarker)) { throw "E_SELFTEST_EXPECTED_REJECTION_MISSING:${Name}:$output" }
  [pscustomobject]@{ name = $Name; status = 'PASS_FAIL_CLOSED'; exitCode = $code; expectedMarker = $ExpectedMarker; sentF3 = $false }
}

Assert-Parseable $runner
Assert-Parseable $scriptPath

$simulationLines = @(& pwsh -NoProfile -File $runner -SelfTestMode 2>&1)
if ($LASTEXITCODE -ne 0) { throw "E_SELFTEST_PREKEY_SIMULATION_EXIT:$($simulationLines -join ' ')" }
$simulation = ($simulationLines -join "`n") | ConvertFrom-Json
if ($simulation.mode -ne 'simulation-only' -or $simulation.realF3Sent -ne $false) { throw 'E_SELFTEST_SIMULATION_MARKER_INVALID' }
if ($simulation.nativeApi.compile -ne 'PASS' -or $simulation.nativeApi.visibleTopLevelWindowEnumeration -ne 'PASS') { throw 'E_SELFTEST_NATIVE_API_PREFLIGHT_FAILED' }
$positiveCases = @('main-window-only','main-plus-two-observed-auxiliary-windows')
$negativeCases = @(
  'wrong-executable-sha','wrong-process-id','wrong-mainmodule-path','wrong-mainmodule-sha','wrong-main-window-hwnd',
  'main-window-without-positive-bounds','main-window-owned-by-different-pid','second-tauri-same-title-window',
  'unknown-empty-title-small-window','tao-window-with-changed-bounds','wrong-main-window-title','foreground-changed'
)
foreach ($name in $positiveCases) {
  $case = @($simulation.results | Where-Object name -eq $name)
  if ($case.Count -ne 1 -or $case[0].f3Calls -ne 1 -or $case[0].gate -ne 'OPEN_SIMULATION_ONLY') { throw "E_SELFTEST_POSITIVE_GATE_CASE:$name" }
}
foreach ($name in $negativeCases) {
  $case = @($simulation.results | Where-Object name -eq $name)
  if ($case.Count -ne 1 -or $case[0].f3Calls -ne 0 -or $case[0].gate -eq 'OPEN_SIMULATION_ONLY') { throw "E_SELFTEST_NEGATIVE_GATE_CASE:$name" }
}
$auxRejects = @($simulation.results | Where-Object { $_.name -in @('second-tauri-same-title-window','unknown-empty-title-small-window','tao-window-with-changed-bounds') -and $_.gate -eq 'E_PREKEY_DISALLOWED_EXTRA_WINDOW' })
if ($auxRejects.Count -ne 3) { throw 'E_SELFTEST_AUXILIARY_REJECTION_CODE' }
$mainHandleReject = $simulation.results | Where-Object { $_.name -eq 'wrong-main-window-hwnd' }
if ($mainHandleReject.gate -ne 'E_PREKEY_MAIN_HWND_MISMATCH') { throw 'E_SELFTEST_MAIN_HWND_ERROR_CODE' }
$boundsReject = $simulation.results | Where-Object { $_.name -eq 'main-window-without-positive-bounds' }
if ($boundsReject.gate -ne 'E_PREKEY_MAIN_WINDOW_METADATA_MISMATCH') { throw 'E_SELFTEST_MAIN_WINDOW_METADATA_ERROR_CODE' }
$noFocus = $simulation.results | Where-Object name -eq 'foreground-changed'
if ($null -eq $noFocus -or $noFocus.f3Calls -ne 0 -or $noFocus.gate -ne 'E_PREKEY_FOREGROUND_FOCUS_MISMATCH') { throw 'E_SELFTEST_PREKEY_GATE_DID_NOT_FAIL_CLOSED' }

$fakeExe = Join-Path $scratch 'deliberately-not-an-executable.exe'
[IO.File]::WriteAllBytes($fakeExe, [Text.Encoding]::ASCII.GetBytes('not an executable; used only for preflight hashing'))
$validHash = (Get-FileHash -LiteralPath $fakeExe -Algorithm SHA256).Hash
$missingEvidencePath = Join-Path $scratch 'missing-sha-evidence'
$missingHash = Assert-RunnerRejected 'missing-expected-sha' $fakeExe '' $missingEvidencePath 'E_EXPECTED_SHA256_INVALID'
if (Test-Path -LiteralPath $missingEvidencePath) { throw 'E_SELFTEST_MISSING_SHA_CREATED_EVIDENCE' }
$wrongEvidencePath = Join-Path $scratch 'wrong-sha-evidence'
$wrongHash = Assert-RunnerRejected 'wrong-expected-sha' $fakeExe ('0' * 64) $wrongEvidencePath 'E_EXECUTABLE_SHA256_MISMATCH'
if (Test-Path -LiteralPath $wrongEvidencePath) { throw 'E_SELFTEST_WRONG_SHA_CREATED_EVIDENCE' }
$repoEvidence = Assert-RunnerRejected 'repository-evidence-path' $fakeExe $validHash $PSScriptRoot 'E_EVIDENCE_DIRECTORY_INSIDE_REPOSITORY'
$nonemptyEvidencePath = Join-Path $scratch 'already-populated-evidence'
[void](New-Item -ItemType Directory -Path $nonemptyEvidencePath)
[IO.File]::WriteAllText((Join-Path $nonemptyEvidencePath 'preserve.txt'), 'self-test sentinel')
$nonemptyEvidence = Assert-RunnerRejected 'nonempty-evidence-path' $fakeExe $validHash $nonemptyEvidencePath 'E_EVIDENCE_DIRECTORY_NOT_EMPTY'
if ([IO.File]::ReadAllText((Join-Path $nonemptyEvidencePath 'preserve.txt')) -ne 'self-test sentinel') { throw 'E_SELFTEST_EXISTING_EVIDENCE_WAS_CHANGED' }

@(
  [pscustomobject]@{ name = 'powershell-parser'; status = 'PASS' }
  [pscustomobject]@{ name = 'native-source-add-type-and-window-enumeration'; status = $simulation.nativeApi.compile; visibleWindowCount = $simulation.nativeApi.visibleWindowCount; realF3Sent = $simulation.realF3Sent }
  $simulation.results
  $missingHash
  $wrongHash
  $repoEvidence
  $nonemptyEvidence
) | ConvertTo-Json -Depth 5
Write-Output ('Self-test scratch preserved at: ' + $scratch)
Write-Output 'No game executable was started and no real F3 key was sent.'
