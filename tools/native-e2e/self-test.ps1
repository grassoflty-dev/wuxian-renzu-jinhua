param()

$ErrorActionPreference = 'Stop'
$runner = Join-Path $PSScriptRoot 'run.ps1'
$scratch = Join-Path $env:TEMP ('native-evidence-harness-selftest-' + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $scratch -Force | Out-Null

function Assert-Case([string]$Name, [string]$Exe, [string[]]$Sizes, [int]$ExpectedExit, [string]$ExpectedStatus) {
  $outputDir = Join-Path $scratch $Name
  $lines = @(& $runner -ExecutablePath $Exe -TargetSize $Sizes -EvidenceDirectory $outputDir 2>&1)
  $code = $LASTEXITCODE
  if ($code -ne $ExpectedExit) { throw "$Name returned $code; expected $ExpectedExit. $($lines -join ' ')" }
  $evidenceFile = Join-Path $outputDir 'evidence.json'
  if (-not (Test-Path -LiteralPath $evidenceFile -PathType Leaf)) { throw "$Name did not write evidence.json" }
  $evidence = Get-Content -LiteralPath $evidenceFile -Raw | ConvertFrom-Json
  if ($evidence.status -ne $ExpectedStatus) { throw "$Name status $($evidence.status); expected $ExpectedStatus" }
  if ($evidence.repository.head -notmatch '^[0-9a-f]{40}$') { throw "$Name lacks a Git HEAD" }
  if ($evidence.buildIdentity.status -ne 'UNSATISFIED' -or $evidence.buildIdentity.embeddedBundleIdentityVerified) {
    throw "$Name invented a verified build identity"
  }
  if ($evidence.process.startedByHarness -or @($evidence.evidence.screenshots).Count -ne 0) {
    throw "$Name launched a process or claimed a screenshot"
  }
  Write-Output "$Name PASS"
}

try {
  $missing = Join-Path $scratch 'missing.exe'
  Assert-Case 'missing-executable' $missing @('1280x720') 1 'FAIL'

  $fake = Join-Path $scratch 'fake.exe'
  Set-Content -LiteralPath $fake -Value 'This is deliberately not a game executable.' -Encoding ASCII
  Assert-Case 'invalid-dimensions' $fake @('invalid') 2 'BLOCKED'

  foreach ($scriptPath in @($runner, $PSCommandPath)) {
    $parsed = $null
    $parseErrors = $null
    [System.Management.Automation.Language.Parser]::ParseFile($scriptPath, [ref]$parsed, [ref]$parseErrors) | Out-Null
    if ($parseErrors.Count -gt 0) { throw "$scriptPath has PowerShell syntax errors" }
  }
  Write-Output 'PowerShell parser PASS'
  $source = Get-Content -LiteralPath $runner -Raw
  $match = [regex]::Match($source, '(?s)\$nativeCode = @"\r?\n(.*?)\r?\n"@')
  if (-not $match.Success) { throw 'Native interop source was not found' }
  $drawingAssemblies = @('System.Drawing.dll', 'System.Drawing.Common.dll', 'System.Drawing.Primitives.dll', 'System.Private.Windows.GdiPlus.dll', 'System.Private.Windows.Core.dll') | ForEach-Object { Join-Path ([System.AppContext]::BaseDirectory) $_ }
  Add-Type -TypeDefinition $match.Groups[1].Value -ReferencedAssemblies $drawingAssemblies -ErrorAction Stop
  if ([NativeEvidenceWindow]::GetSystemMetrics(0) -le 0) { throw 'Windows display metric unavailable' }
  Write-Output 'Native interop compilation PASS'
} finally {
  $resolvedScratch = [IO.Path]::GetFullPath($scratch)
  $tempPrefix = [IO.Path]::GetFullPath($env:TEMP).TrimEnd([IO.Path]::DirectorySeparatorChar) + [IO.Path]::DirectorySeparatorChar
  if (-not $resolvedScratch.StartsWith($tempPrefix, [StringComparison]::OrdinalIgnoreCase) -or
      -not ([IO.Path]::GetFileName($resolvedScratch)).StartsWith('native-evidence-harness-selftest-', [StringComparison]::Ordinal)) {
    throw 'Self-test scratch path escaped the expected TEMP directory'
  }
  Remove-Item -LiteralPath $resolvedScratch -Recurse -Force
}
