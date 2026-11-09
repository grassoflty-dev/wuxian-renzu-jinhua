# SOURCE CANDIDATE: separate build authorization required. Never run by reading this file.
param(
  [Parameter(Mandatory)][string]$RepoRoot,
  [Parameter(Mandatory)][string]$CargoExe,
  [Parameter(Mandatory)][string]$CargoSha256,
  [Parameter(Mandatory)][string]$RustcExe,
  [Parameter(Mandatory)][string]$RustcSha256,
  [Parameter(Mandatory)][string]$GodotExe,
  [Parameter(Mandatory)][string]$GodotSha256,
  [Parameter(Mandatory)][string]$OfflineCargoHome,
  [Parameter(Mandatory)][string]$OutputRoot,
  [Parameter(Mandatory)][string]$TargetTriple
)
$ErrorActionPreference='Stop'
if (-not $IsWindows -or $PSVersionTable.PSEdition -cne 'Core' -or $PSVersionTable.PSVersion -lt [version]'7.4') { throw 'Windows and PowerShell Core 7.4+ required' }
function Resolve-Ordinary([string]$Path, [bool]$Exists) {
  if (-not [IO.Path]::IsPathFullyQualified($Path)) { throw 'Absolute path required' }
  $full=[IO.Path]::GetFullPath($Path)
  if ($Exists -and -not (Test-Path -LiteralPath $full)) { throw "Missing path: $full" }
  $item=if (Test-Path -LiteralPath $full) { Get-Item -Force -LiteralPath $full } else { Get-Item -Force -LiteralPath ([IO.Path]::GetDirectoryName($full)) }
  while ($null -ne $item) {
    if (($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) { throw "Reparse point: $($item.FullName)" }
    $item=if ($item.PSIsContainer) { $item.Parent } else { $item.Directory }
  }
  return $full
}
function Pin-File([string]$Path,[string]$Expected) {
  if ($Expected -cnotmatch '^[a-fA-F0-9]{64}$') { throw 'Invalid SHA-256' }
  $item=Get-Item -LiteralPath (Resolve-Ordinary $Path $true) -Force
  if ($item.PSIsContainer) { throw 'Expected file' }
  $hash=(Get-FileHash -LiteralPath $item.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
  if ($hash -cne $Expected.ToLowerInvariant()) { throw "Pin mismatch: $($item.FullName)" }
  return [ordered]@{path=$item.FullName;bytes=$item.Length;sha256=$hash}
}
function Walk-Input([string]$Base,[string]$Directory,[System.Collections.Generic.List[object]]$Rows,[bool]$ExcludeGenerated) {
  foreach ($item in Get-ChildItem -LiteralPath $Directory -Force) {
    if (($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) { throw "Input reparse point: $($item.FullName)" }
    if ($item.PSIsContainer) {
      if ($ExcludeGenerated -and $item.Name -cin @('target','.godot')) { continue }
      Walk-Input $Base $item.FullName $Rows $ExcludeGenerated
    } else {
      $Rows.Add([ordered]@{path=[IO.Path]::GetRelativePath($Base,$item.FullName).Replace('\','/');bytes=$item.Length;sha256=(Get-FileHash -LiteralPath $item.FullName -Algorithm SHA256).Hash.ToLowerInvariant()})
    }
  }
}
function Snapshot([string]$Base,[string[]]$Trees,[string[]]$Files,[bool]$ExcludeGenerated) {
  $rows=[Collections.Generic.List[object]]::new()
  foreach ($tree in $Trees) {
    $dir=Resolve-Ordinary (Join-Path $Base $tree) $true
    if (-not (Test-Path -LiteralPath $dir -PathType Container)) { throw "Input tree missing: $tree" }
    Walk-Input $Base $dir $rows $ExcludeGenerated
  }
  foreach ($relative in $Files) {
    $path=Join-Path $Base $relative
    if (Test-Path -LiteralPath $path -PathType Leaf) {
      $item=Get-Item -LiteralPath (Resolve-Ordinary $path $true) -Force
      $rows.Add([ordered]@{path=$relative;bytes=$item.Length;sha256=(Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant()})
    } else { $rows.Add([ordered]@{path=$relative;absent=$true}) }
  }
  return @($rows | Sort-Object { $_.path })
}
$repo=Resolve-Ordinary $RepoRoot $true
# Cargo searches ancestors of its working directory. Refuse unrecorded ancestor configs.
$ancestor=(Get-Item -LiteralPath $repo -Force).Parent
while ($null -ne $ancestor) {
  foreach ($name in @('config','config.toml')) {
    if (Test-Path -LiteralPath (Join-Path $ancestor.FullName ".cargo/$name")) { throw "Uninventoried ancestor Cargo config: $($ancestor.FullName)" }
  }
  $ancestor=$ancestor.Parent
}
$workingDir=Resolve-Ordinary (Join-Path $repo 'server-rs') $true
$cargo=Pin-File $CargoExe $CargoSha256
$rustc=Pin-File $RustcExe $RustcSha256
$godot=Pin-File $GodotExe $GodotSha256
$cargoHomePath=Resolve-Ordinary $OfflineCargoHome $true
if (-not (Test-Path -LiteralPath $cargoHomePath -PathType Container)) { throw 'Qualified prepopulated offline Cargo home required' }
if ($TargetTriple -cnotmatch '^[A-Za-z0-9_]+(-[A-Za-z0-9_]+)+$') { throw 'Explicit target triple required' }
$out=Resolve-Ordinary $OutputRoot $false
if (Test-Path -LiteralPath $out) { throw 'Output root already exists' }
foreach ($protected in @($repo,$cargoHomePath)) {
  if ($out.StartsWith($protected.TrimEnd('\')+'\',[StringComparison]::OrdinalIgnoreCase) -or $protected.StartsWith($out.TrimEnd('\')+'\',[StringComparison]::OrdinalIgnoreCase)) { throw 'Output overlaps source or dependency cache' }
}
foreach ($entry in Get-ChildItem Env:) {
  if ($entry.Name -match '^(CARGO_|RUSTFLAGS$|RUSTC|RUSTDOC|RUSTUP_)') { throw "Clear inherited build setting: $($entry.Name)" }
}
$trees=@('server-rs','content/scenes/compiled','governance/assets','content/enemies','apps/godot_playable','tools/grey-hive-player')
$files=@('rust-toolchain','rust-toolchain.toml','.cargo/config','.cargo/config.toml','server-rs/rust-toolchain','server-rs/rust-toolchain.toml','server-rs/.cargo/config','server-rs/.cargo/config.toml')
$before=@(Snapshot $repo $trees $files $true)
# The cache is separately qualified and prepopulated; this script never downloads or prepares it.
$cacheBefore=@(Snapshot $cargoHomePath @('.') @() $false)
New-Item -ItemType Directory -Path $out -ErrorAction Stop | Out-Null
$target=Join-Path $out 'target'
[IO.Directory]::CreateDirectory($target) | Out-Null
$argsList=@('build','--manifest-path',(Join-Path $repo 'server-rs/Cargo.toml'),'--bin','wuxian_v1_authority','--release','--locked','--offline','--target',$TargetTriple,'--target-dir',$target)
$originalLocation=Get-Location
$originalProcessDirectory=[Environment]::CurrentDirectory
try {
  Set-Location -LiteralPath $workingDir
  [Environment]::CurrentDirectory=$workingDir
  $env:CARGO_HOME=$cargoHomePath; $env:RUSTC=$rustc.path
  & $cargo.path @argsList 1> (Join-Path $out 'build.stdout.log') 2> (Join-Path $out 'build.stderr.log')
  if ($LASTEXITCODE -ne 0) { throw "Cargo failed: $LASTEXITCODE" }
} finally {
  Remove-Item Env:CARGO_HOME,Env:RUSTC -ErrorAction SilentlyContinue
  [Environment]::CurrentDirectory=$originalProcessDirectory
  Set-Location -LiteralPath $originalLocation.Path
}
$null=Pin-File $CargoExe $CargoSha256
$null=Pin-File $RustcExe $RustcSha256
$null=Pin-File $GodotExe $GodotSha256
$after=@(Snapshot $repo $trees $files $true)
$cacheAfter=@(Snapshot $cargoHomePath @('.') @() $false)
if ((ConvertTo-Json -InputObject $before -Depth 6 -Compress) -cne (ConvertTo-Json -InputObject $after -Depth 6 -Compress)) { throw 'Inputs changed; no success manifest' }
if ((ConvertTo-Json -InputObject $cacheBefore -Depth 6 -Compress) -cne (ConvertTo-Json -InputObject $cacheAfter -Depth 6 -Compress)) { throw 'Dependency cache changed; no success manifest' }
$binary=Get-Item -LiteralPath (Resolve-Ordinary (Join-Path $target "$TargetTriple/release/wuxian_v1_authority.exe") $true) -Force
$record=[ordered]@{schema='wuxian-current-build-provenance/v1';status='BUILD_SUCCEEDED_INPUTS_STABLE';cargo=$cargo;rustc=$rustc;godot=$godot;cargoHome=$cargoHomePath;dependencyCache=$cacheBefore;target=$TargetTriple;workingDir=$workingDir;configPolicy='repo/server configs inventoried; ancestors above repo rejected; offline Cargo home inventoried';arguments=$argsList;inputFiles=$before;native=@{path=$binary.FullName;bytes=$binary.Length;sha256=(Get-FileHash -LiteralPath $binary.FullName -Algorithm SHA256).Hash.ToLowerInvariant()};frontend='SOURCE_ONLY_UNPARSED';limitations='Linker, toolchain supporting files, OS and environment are not fully pinned; pre/post observations cannot prove no intermediate changes or bit reproducibility. No bridge compilation, gameplay or runtime acceptance claimed.'}
$bytes=[Text.Encoding]::UTF8.GetBytes(($record | ConvertTo-Json -Depth 12))
$file=[IO.File]::Open((Join-Path $out 'build-provenance.json'),[IO.FileMode]::CreateNew,[IO.FileAccess]::Write,[IO.FileShare]::None)
try { $file.Write($bytes,0,$bytes.Length);$file.Flush($true) } finally { $file.Dispose() }
