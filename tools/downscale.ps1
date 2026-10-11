# downscale.ps1 — 最近邻下采样（真像素画关键步骤）
# 用法: pwsh -File tools/downscale.ps1 <inPng> <outPng> <size>
param([string]$In, [string]$Out, [int]$Size = 48)
Add-Type -AssemblyName System.Drawing
$src = [System.Drawing.Image]::FromFile((Resolve-Path $In))
$dst = New-Object System.Drawing.Bitmap($Size, $Size, [System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
$g = [System.Drawing.Graphics]::FromImage($dst)
$g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::NearestNeighbor
$g.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::Half
$g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::None
$g.DrawImage($src, 0, 0, $Size, $Size)
$g.Dispose()
$dir = Split-Path -Parent $Out
if ($dir -and -not (Test-Path $dir)) { New-Item -ItemType Directory -Force -Path $dir | Out-Null }
$dst.Save((Join-Path (Get-Location) $Out), [System.Drawing.Imaging.ImageFormat]::Png)
$dst.Dispose(); $src.Dispose()
Write-Output ("OK " + $Out + " " + $Size + "x" + $Size)