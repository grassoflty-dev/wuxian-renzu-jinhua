<#
  fetch_assets.ps1 - download CC0 3D assets for the Wuxian Horror TPS migration (phase 3 prep).
  Downloads only. No text-file mutation. Safe to re-run (skips files already present & non-empty).
#>
$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'

$Root = Join-Path $PSScriptRoot 'raw'
New-Item -ItemType Directory -Force -Path $Root | Out-Null

# Each entry: Name | Url | DestRelPath
$Manifest = @(
  # ---- KayKit Adventurers Character Pack (CC0) : humanoid, rigged, embedded anims ----
  @{ n='kaykit_knight';    u='https://cdn.jsdelivr.net/gh/KayKit-Game-Assets/KayKit-Character-Pack-Adventures-1.0@main/addons/kaykit_character_pack_adventures/Characters/gltf/Knight.glb';    d='kaykit_adventurers/Knight.glb' },
  @{ n='kaykit_rogue';     u='https://cdn.jsdelivr.net/gh/KayKit-Game-Assets/KayKit-Character-Pack-Adventures-1.0@main/addons/kaykit_character_pack_adventures/Characters/gltf/Rogue.glb';     d='kaykit_adventurers/Rogue.glb' },
  @{ n='kaykit_mage';      u='https://cdn.jsdelivr.net/gh/KayKit-Game-Assets/KayKit-Character-Pack-Adventures-1.0@main/addons/kaykit_character_pack_adventures/Characters/gltf/Mage.glb';      d='kaykit_adventurers/Mage.glb' },
  @{ n='kaykit_barbarian'; u='https://cdn.jsdelivr.net/gh/KayKit-Game-Assets/KayKit-Character-Pack-Adventures-1.0@main/addons/kaykit_character_pack_adventures/Characters/gltf/Barbarian.glb'; d='kaykit_adventurers/Barbarian.glb' },
  @{ n='kaykit_rogue_hooded'; u='https://cdn.jsdelivr.net/gh/KayKit-Game-Assets/KayKit-Character-Pack-Adventures-1.0@main/addons/kaykit_character_pack_adventures/Characters/gltf/Rogue_Hooded.glb'; d='kaykit_adventurers/Rogue_Hooded.glb' },

  # ---- KayKit Skeletons Character Pack (CC0) : undead humanoid, rigged, embedded anims ----
  @{ n='kaykit_skel_warrior'; u='https://cdn.jsdelivr.net/gh/KayKit-Game-Assets/KayKit-Character-Pack-Skeletons-1.0@main/addons/kaykit_character_pack_skeletons/Characters/gltf/Skeleton_Warrior.glb'; d='kaykit_skeletons/Skeleton_Warrior.glb' },
  @{ n='kaykit_skel_minion';  u='https://cdn.jsdelivr.net/gh/KayKit-Game-Assets/KayKit-Character-Pack-Skeletons-1.0@main/addons/kaykit_character_pack_skeletons/Characters/gltf/Skeleton_Minion.glb';  d='kaykit_skeletons/Skeleton_Minion.glb' }
)

$results = @()
foreach ($m in $Manifest) {
  $dest = Join-Path $Root $m.d
  $dir  = Split-Path $dest -Parent
  New-Item -ItemType Directory -Force -Path $dir | Out-Null
  if ((Test-Path $dest) -and ((Get-Item $dest).Length -gt 1024)) {
    $results += [pscustomobject]@{ name=$m.n; size=(Get-Item $dest).Length; status='cached'; dest=$m.d }
    continue
  }
  $ok = $false
  foreach ($attempt in 1..3) {
    try {
      Invoke-WebRequest -Uri $m.u -OutFile $dest -TimeoutSec 180 -UseBasicParsing
      $ok = $true; break
    } catch {
      Write-Host ("  retry {0} for {1}: {2}" -f $attempt, $m.n, $_.Exception.Message)
      Start-Sleep -Seconds 2
    }
  }
  if ($ok) {
    $len = (Get-Item $dest).Length
    $results += [pscustomobject]@{ name=$m.n; size=$len; status='ok'; dest=$m.d }
  } else {
    $results += [pscustomobject]@{ name=$m.n; size=0; status='FAILED'; dest=$m.d }
  }
}
$results | Format-Table -AutoSize
$total = ($results | Measure-Object -Property size -Sum).Sum
Write-Host ("TOTAL BYTES: {0}  ({1:N2} MB)" -f $total, ($total/1MB))
