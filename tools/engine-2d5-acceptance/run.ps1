param(
  [string]$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
)

$ErrorActionPreference = 'Stop'
$evidence = Join-Path $RepoRoot 'artifacts\acceptance\engine-2d5-core-01'
New-Item -ItemType Directory -Path $evidence -Force | Out-Null

$checks = @(
  @{ Name = 'owner-idle-pause'; Command = { cargo test --manifest-path (Join-Path $RepoRoot 'server-rs\Cargo.toml') --lib owner_advances_without_browser_input_and_pause_holds_time --quiet } },
  @{ Name = 'presentation-events'; Command = { cargo test --manifest-path (Join-Path $RepoRoot 'server-rs\Cargo.toml') --lib presentation_events_have_stable_ids_independent_of_snapshots --quiet } },
  @{ Name = 'process-restart-continue'; Command = { cargo test --manifest-path (Join-Path $RepoRoot 'server-rs\Cargo.toml') --test grey_hive_first_flow power_and_open_gate_survive_process_restart_with_collision_rebuilt --quiet } },
  @{ Name = 'save-slots'; Command = { cargo test --manifest-path (Join-Path $RepoRoot 'server-rs\Cargo.toml') --test freeze_v02_save_v3 --quiet } },
  @{ Name = 'web-contracts'; Command = { npm test --prefix (Join-Path $RepoRoot 'apps\web') } }
)

$results = foreach ($check in $checks) {
  $output = & $check.Command 2>&1
  $code = $LASTEXITCODE
  $logPath = Join-Path $evidence ($check.Name + '.log')
  [System.IO.File]::WriteAllText($logPath, (($output | Out-String).TrimEnd() + "`n"), [System.Text.UTF8Encoding]::new($false))
  [pscustomobject]@{ name = $check.Name; passed = ($code -eq 0); exitCode = $code; log = ('artifacts/acceptance/engine-2d5-core-01/' + $check.Name + '.log') }
}
$results | ConvertTo-Json -Depth 3 | Out-File -LiteralPath (Join-Path $evidence 'verification.json') -Encoding utf8
$results | Format-Table -AutoSize
if ($results.Where({ -not $_.passed }).Count -gt 0) { exit 1 }
