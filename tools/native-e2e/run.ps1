[CmdletBinding(DefaultParameterSetName = "Executable")]
param(
  [Parameter(Mandatory = $true, ParameterSetName = "Executable")]
  [string]$ExecutablePath,

  [Parameter(Mandatory = $true, ParameterSetName = "Process")]
  [int]$ProcessId,

  [string]$EvidenceDirectory,
  [string]$ExpectedWindowTitle = "无限人族进化",
  [string[]]$TargetSize = @("1280x720", "1920x1080", "2560x1440"),
  [switch]$KeepOpen
)

$ErrorActionPreference = "Stop"
$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path
$startedAt = (Get-Date).ToUniversalTime().ToString("o")
$defaultEvidenceRoot = Join-Path $env:TEMP "native-evidence-harness-01"
if ([string]::IsNullOrWhiteSpace($EvidenceDirectory)) {
  $EvidenceDirectory = Join-Path $defaultEvidenceRoot (Get-Date -Format "yyyyMMdd-HHmmss-fff")
}
$repoFullPath = [IO.Path]::GetFullPath($repoRoot).TrimEnd([IO.Path]::DirectorySeparatorChar) + [IO.Path]::DirectorySeparatorChar
$evidenceFullPath = [IO.Path]::GetFullPath($EvidenceDirectory)
if ($evidenceFullPath.StartsWith($repoFullPath, [StringComparison]::OrdinalIgnoreCase)) {
  $evidenceFullPath = Join-Path $defaultEvidenceRoot ("rejected-path-" + [guid]::NewGuid().ToString("N"))
}
New-Item -ItemType Directory -Force -Path $evidenceFullPath | Out-Null
$evidencePath = Join-Path $evidenceFullPath "evidence.json"

$gitHead = ""
try {
  $gitHead = (& git -C $repoRoot rev-parse HEAD 2>$null | Out-String).Trim()
} catch {
  $gitHead = ""
}
$record = [ordered]@{
  schemaVersion = 1
  status = "FAIL"
  startedAtUtc = $startedAt
  completedAtUtc = $null
  repository = [ordered]@{ root = $repoRoot; head = $gitHead }
  executable = [ordered]@{ requestedPath = $ExecutablePath; fullPath = $null; sizeBytes = $null; sha256 = $null }
  process = [ordered]@{ pid = $null; startedByHarness = $false; mainWindowHandle = $null; title = $null; responding = $null }
  display = [ordered]@{ width = $null; height = $null; targetSteps = @() }
  buildIdentity = [ordered]@{
    status = "UNSATISFIED"
    note = "Executable SHA256 and repository HEAD are recorded separately; they do not prove that this binary embeds a bundle built from this HEAD."
    sourceRevisionVerified = $false
    embeddedBundleIdentityVerified = $false
  }
  evidence = [ordered]@{ directory = $evidenceFullPath; json = $evidencePath; screenshots = @() }
  errors = @()
}

function Save-Evidence {
  $record.completedAtUtc = (Get-Date).ToUniversalTime().ToString("o")
  $record | ConvertTo-Json -Depth 10 | Set-Content -LiteralPath $evidencePath -Encoding UTF8
}

function Add-Step([string]$Name, [string]$Status, [string]$Detail, [hashtable]$Data = @{}) {
  $record.display.targetSteps += [ordered]@{
    name = $Name
    status = $Status
    detail = $Detail
    data = $Data
    timeUtc = (Get-Date).ToUniversalTime().ToString("o")
  }
}

$nativeCode = @"
using System;
using System.Runtime.InteropServices;
using System.Text;
using System.Drawing;
public static class NativeEvidenceWindow {
  [StructLayout(LayoutKind.Sequential)]
  public struct RECT { public int Left; public int Top; public int Right; public int Bottom; }
  [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
  [DllImport("user32.dll")] public static extern int GetSystemMetrics(int index);
  [DllImport("user32.dll", SetLastError=true)] public static extern bool GetWindowRect(IntPtr hWnd, out RECT rect);
  [DllImport("user32.dll", SetLastError=true)] public static extern bool SetWindowPos(IntPtr hWnd, IntPtr insertAfter, int x, int y, int width, int height, uint flags);
  [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr hWnd, int command);
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr hWnd);
  [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern int GetWindowText(IntPtr hWnd, StringBuilder text, int maxCount);
  public static string Title(IntPtr hWnd) {
    StringBuilder text = new StringBuilder(1024);
    GetWindowText(hWnd, text, text.Capacity);
    return text.ToString();
  }
  public static int[] Bounds(IntPtr hWnd) {
    RECT r;
    if (!GetWindowRect(hWnd, out r)) return new int[0];
    return new int[] { r.Left, r.Top, r.Right - r.Left, r.Bottom - r.Top };
  }
  public static bool Place(IntPtr hWnd, int x, int y, int width, int height) {
    const uint SWP_NOZORDER = 0x0004;
    const uint SWP_SHOWWINDOW = 0x0040;
    return SetWindowPos(hWnd, IntPtr.Zero, x, y, width, height, SWP_NOZORDER | SWP_SHOWWINDOW);
  }
  public static void CaptureVisibleWindow(string path, int x, int y, int width, int height) {
    using (Bitmap image = new Bitmap(width, height)) {
      using (Graphics graphics = Graphics.FromImage(image)) {
        graphics.CopyFromScreen(x, y, 0, 0, new Size(width, height), CopyPixelOperation.SourceCopy);
      }
      image.Save(path, System.Drawing.Imaging.ImageFormat.Png);
    }
  }
}
"@

$launchedProcess = $false
$targetEntries = @()
foreach ($sizeText in $TargetSize) {
  if ($sizeText -match "^([1-9][0-9]{2,3})x([1-9][0-9]{2,3})$") {
    $width = [int]$Matches[1]
    $height = [int]$Matches[2]
    $targetEntries += [ordered]@{ name = $sizeText; width = $width; height = $height; valid = ($width -le 7680 -and $height -le 4320) }
  } else {
    $targetEntries += [ordered]@{ name = $sizeText; width = $null; height = $null; valid = $false }
  }
}

try {
  if ([string]::IsNullOrWhiteSpace($gitHead)) { throw "E_REPOSITORY_HEAD_UNAVAILABLE" }
  if ($PSCmdlet.ParameterSetName -eq "Executable") {
    if (-not (Test-Path -LiteralPath $ExecutablePath -PathType Leaf)) {
      $record.executable.fullPath = [IO.Path]::GetFullPath($ExecutablePath)
      throw "E_EXECUTABLE_NOT_FOUND"
    }
    $resolvedExe = (Resolve-Path -LiteralPath $ExecutablePath).Path
    $file = Get-Item -LiteralPath $resolvedExe
    $hash = Get-FileHash -LiteralPath $resolvedExe -Algorithm SHA256
    $record.executable.fullPath = $resolvedExe
    $record.executable.sizeBytes = $file.Length
    $record.executable.sha256 = $hash.Hash
  }

  if ($targetEntries.Count -eq 0) { throw "E_NO_TARGET_SIZES" }
  foreach ($target in $targetEntries) {
    if (-not $target.valid) {
      Add-Step $target.name "BLOCKED" "Requested dimensions are invalid or outside the supported capture range; no screenshot was attempted."
    }
  }
  $validTargets = @($targetEntries | Where-Object { $_.valid })
  if ($validTargets.Count -eq 0) {
    $record.status = "BLOCKED"
    throw "E_NO_USABLE_TARGET_SIZES"
  }

  $drawingAssemblies = @('System.Drawing.dll', 'System.Drawing.Common.dll', 'System.Drawing.Primitives.dll', 'System.Private.Windows.GdiPlus.dll', 'System.Private.Windows.Core.dll') | ForEach-Object { Join-Path ([System.AppContext]::BaseDirectory) $_ }
  Add-Type -TypeDefinition $nativeCode -ReferencedAssemblies $drawingAssemblies
  [void][NativeEvidenceWindow]::SetProcessDPIAware()
  $record.display.width = [NativeEvidenceWindow]::GetSystemMetrics(0)
  $record.display.height = [NativeEvidenceWindow]::GetSystemMetrics(1)
  if ($record.display.width -le 0 -or $record.display.height -le 0) { throw "E_DISPLAY_UNAVAILABLE" }

  if ($PSCmdlet.ParameterSetName -eq "Executable") {
    $process = Start-Process -FilePath $record.executable.fullPath -WorkingDirectory (Split-Path -Parent $record.executable.fullPath) -PassThru -WindowStyle Normal
    $launchedProcess = $true
    $record.process.startedByHarness = $true
    $record.process.pid = $process.Id
  } else {
    $process = Get-Process -Id $ProcessId -ErrorAction Stop
    $record.process.pid = $process.Id
    try {
      $resolvedExe = $process.MainModule.FileName
      $file = Get-Item -LiteralPath $resolvedExe -ErrorAction Stop
      $hash = Get-FileHash -LiteralPath $resolvedExe -Algorithm SHA256
      $record.executable.fullPath = $resolvedExe
      $record.executable.sizeBytes = $file.Length
      $record.executable.sha256 = $hash.Hash
    } catch {
      throw "E_PROCESS_EXECUTABLE_IDENTITY_UNAVAILABLE"
    }
  }

  $deadline = (Get-Date).AddSeconds(25)
  $windowHandle = [IntPtr]::Zero
  while ((Get-Date) -lt $deadline) {
    try {
      $process.Refresh()
      $windowHandle = $process.MainWindowHandle
      if ($windowHandle -ne [IntPtr]::Zero) { break }
    } catch {}
    Start-Sleep -Milliseconds 250
  }
  if ($windowHandle -eq [IntPtr]::Zero) {
    Add-Step "main-window" "FAIL" "No visible main-window handle was available for the selected process."
    throw "E_MAIN_WINDOW_UNAVAILABLE"
  }

  $windowTitle = [NativeEvidenceWindow]::Title($windowHandle)
  $record.process.mainWindowHandle = ("0x{0:X}" -f $windowHandle.ToInt64())
  $record.process.title = $windowTitle
  $record.process.responding = $process.Responding
  if ([string]::IsNullOrWhiteSpace($windowTitle) -or -not $windowTitle.Contains($ExpectedWindowTitle)) {
    Add-Step "main-window-title" "FAIL" "Window title did not include the expected application title." @{ expected = $ExpectedWindowTitle; actual = $windowTitle }
    throw "E_WINDOW_TITLE_MISMATCH"
  }
  Add-Step "main-window" "PASS" "Selected process has a visible titled main window." @{ pid = $process.Id; hwnd = $record.process.mainWindowHandle; title = $windowTitle; responding = $record.process.responding }

  [void][NativeEvidenceWindow]::ShowWindow($windowHandle, 9)
  if (-not [NativeEvidenceWindow]::SetForegroundWindow($windowHandle)) {
    Add-Step "foreground" "BLOCKED" "Windows did not grant foreground focus; screenshots were not attempted."
    $record.status = "BLOCKED"
    throw "E_WINDOW_FOREGROUND_UNAVAILABLE"
  }
  foreach ($target in $validTargets) {
    $name = $target.name
    $width = $target.width
    $height = $target.height
    if ($width -gt $record.display.width -or $height -gt $record.display.height) {
      Add-Step $name "BLOCKED" "Requested outer-window size exceeds the primary display bounds; resize and screenshot were not attempted." @{ requestedWidth = $width; requestedHeight = $height; displayWidth = $record.display.width; displayHeight = $record.display.height }
      continue
    }

    $x = [Math]::Max(0, [int](($record.display.width - $width) / 2))
    $y = [Math]::Max(0, [int](($record.display.height - $height) / 2))
    if (-not [NativeEvidenceWindow]::Place($windowHandle, $x, $y, $width, $height)) {
      Add-Step $name "FAIL" "Windows rejected the requested window size." @{ requestedWidth = $width; requestedHeight = $height }
      continue
    }
    Start-Sleep -Milliseconds 700
    $bounds = [NativeEvidenceWindow]::Bounds($windowHandle)
    if ($bounds.Count -ne 4 -or $bounds[0] -ne $x -or $bounds[1] -ne $y -or $bounds[2] -ne $width -or $bounds[3] -ne $height) {
      Add-Step $name "BLOCKED" "The live window did not reach the requested bounds; no screenshot was attempted." @{ requested = @($x, $y, $width, $height); actual = @($bounds) }
      continue
    }
    if ([NativeEvidenceWindow]::GetForegroundWindow() -ne $windowHandle) {
      Add-Step $name "BLOCKED" "The selected native window lost foreground focus; no screenshot was attempted." @{ actual = @($bounds) }
      continue
    }

    $screenshotPath = Join-Path $evidenceFullPath ("window-" + $width + "x" + $height + ".png")
    try {
      [NativeEvidenceWindow]::CaptureVisibleWindow($screenshotPath, $bounds[0], $bounds[1], $bounds[2], $bounds[3])
      $captured = Get-Item -LiteralPath $screenshotPath
      if ($captured.Length -le 0) { throw "E_EMPTY_SCREENSHOT" }
      $captureHash = (Get-FileHash -LiteralPath $screenshotPath -Algorithm SHA256).Hash
      $record.evidence.screenshots += [ordered]@{ width = $width; height = $height; path = $screenshotPath; sizeBytes = $captured.Length; sha256 = $captureHash; source = "CopyFromScreen visible desktop capture" }
      Add-Step $name "PASS" "Visible native window bounds matched and a real desktop screenshot was written." @{ actual = @($bounds); screenshot = $screenshotPath; screenshotBytes = $captured.Length }
    } catch {
      Add-Step $name "BLOCKED" ("Visible desktop capture failed: " + $_.Exception.Message) @{ actual = @($bounds); screenshot = $screenshotPath }
    }
  }

  $statuses = @($record.display.targetSteps | ForEach-Object { $_.status })
  if ($statuses -contains "FAIL") { $record.status = "FAIL" }
  elseif ($statuses -contains "BLOCKED") { $record.status = "BLOCKED" }
  elseif ($statuses.Count -gt 0 -and ($statuses | Where-Object { $_ -ne "PASS" }).Count -eq 0) { $record.status = "PASS" }
  else { $record.status = "FAIL" }
} catch {
  $record.errors += $_.Exception.Message
  if ($record.status -ne "BLOCKED") { $record.status = "FAIL" }
} finally {
  if ($launchedProcess -and -not $KeepOpen -and $null -ne $process) {
    try {
      Stop-Process -Id $process.Id -Force -ErrorAction Stop
      Add-Step "harness-process-cleanup" "PASS" "Stopped only the process started by this invocation." @{ pid = $process.Id }
    } catch {
      Add-Step "harness-process-cleanup" "FAIL" ("Could not stop the process started by this invocation: " + $_.Exception.Message) @{ pid = $process.Id }
      $record.status = "FAIL"
    }
  }
  Save-Evidence
}

Write-Output ("Evidence: " + $evidencePath)
Write-Output ("Status: " + $record.status)
if ($record.status -eq "PASS") { exit 0 }
if ($record.status -eq "BLOCKED") { exit 2 }
exit 1
