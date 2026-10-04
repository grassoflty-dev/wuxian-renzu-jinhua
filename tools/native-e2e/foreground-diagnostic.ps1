[CmdletBinding()]
param(
  [string]$ExecutablePath,
  [string]$ExpectedSha256,
  [string]$EvidenceDirectory,
  [switch]$SelfTestMode
)

$ErrorActionPreference = 'Stop'
$script:startedProcess = $null
$script:record = [ordered]@{
  schemaVersion = 1
  diagnostic = 'NATIVE-FOREGROUND-OBSERVE-01'
  status = 'INITIALIZING'
  startedAtUtc = [DateTime]::UtcNow.ToString('o')
  candidate = $null
  process = $null
  observations = [System.Collections.Generic.List[object]]::new()
  cleanup = $null
  errors = [System.Collections.Generic.List[string]]::new()
}

$script:NativeSource = @'
using System;
using System.Runtime.InteropServices;
using System.Text;
public static class ForegroundObserveNative {
  [StructLayout(LayoutKind.Sequential)] public struct RECT { public int Left; public int Top; public int Right; public int Bottom; }
  [StructLayout(LayoutKind.Sequential)] public struct WindowInfo {
    public IntPtr Hwnd; public uint ProcessId; public string Title; public string ClassName;
    public bool Visible; public bool BoundsAvailable; public int Left; public int Top; public int Width; public int Height;
  }
  [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] public static extern bool IsWindow(IntPtr hwnd);
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr hwnd);
  [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr hwnd, int command);
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr hwnd);
  [DllImport("user32.dll")] private static extern uint GetWindowThreadProcessId(IntPtr hwnd, out uint processId);
  [DllImport("user32.dll", CharSet=CharSet.Unicode, SetLastError=true)] private static extern int GetWindowText(IntPtr hwnd, StringBuilder text, int maxCount);
  [DllImport("user32.dll", CharSet=CharSet.Unicode, SetLastError=true)] private static extern int GetClassName(IntPtr hwnd, StringBuilder className, int maxCount);
  [DllImport("user32.dll", SetLastError=true)] private static extern bool GetWindowRect(IntPtr hwnd, out RECT rect);
  public static WindowInfo ReadWindow(IntPtr hwnd) {
    var value = new WindowInfo(); value.Hwnd = hwnd; value.Title = ""; value.ClassName = "";
    if (hwnd == IntPtr.Zero || !IsWindow(hwnd)) return value;
    uint pid; GetWindowThreadProcessId(hwnd, out pid); value.ProcessId = pid;
    var title = new StringBuilder(2048); GetWindowText(hwnd, title, title.Capacity); value.Title = title.ToString();
    var cls = new StringBuilder(512); GetClassName(hwnd, cls, cls.Capacity); value.ClassName = cls.ToString();
    value.Visible = IsWindowVisible(hwnd);
    RECT r; value.BoundsAvailable = GetWindowRect(hwnd, out r);
    if (value.BoundsAvailable) { value.Left=r.Left; value.Top=r.Top; value.Width=r.Right-r.Left; value.Height=r.Bottom-r.Top; }
    return value;
  }
}
'@

function Get-HwndText([IntPtr]$Value) { '0x{0:X}' -f $Value.ToInt64() }
function Get-WindowRecord([IntPtr]$Hwnd) {
  $w = [ForegroundObserveNative]::ReadWindow($Hwnd)
  [ordered]@{
    hwnd = Get-HwndText $w.Hwnd
    pid = [uint32]$w.ProcessId
    title = [string]$w.Title
    className = [string]$w.ClassName
    visible = [bool]$w.Visible
    boundsAvailable = [bool]$w.BoundsAvailable
    bounds = if ($w.BoundsAvailable) { @([int]$w.Left,[int]$w.Top,[int]$w.Width,[int]$w.Height) } else { $null }
  }
}
function Get-ProcessPath([Diagnostics.Process]$Process) {
  try { [IO.Path]::GetFullPath($Process.MainModule.FileName) } catch { $null }
}
function Get-ProcessSha([Diagnostics.Process]$Process) {
  $path = Get-ProcessPath $Process
  if (-not $path) { return $null }
  (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToUpperInvariant()
}
function Assert-NoReparseAncestors([string]$Path) {
  $cursor = [IO.DirectoryInfo]::new($Path)
  while ($null -ne $cursor) {
    if ($cursor.Exists -and (($cursor.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0)) { throw 'E_REPARSE_POINT_IN_EVIDENCE_PATH' }
    $cursor = $cursor.Parent
  }
}
function Add-Observation([string]$Phase, [int]$Attempt, [Nullable[bool]]$ApiReturn) {
  $script:startedProcess.Refresh()
  $targetHwnd = [IntPtr]$script:startedProcess.MainWindowHandle
  $fg = [ForegroundObserveNative]::GetForegroundWindow()
  $script:record.observations.Add([ordered]@{
    timestampUtc = [DateTime]::UtcNow.ToString('o')
    phase = $Phase
    attempt = $Attempt
    apiReturn = $ApiReturn
    target = Get-WindowRecord $targetHwnd
    foreground = Get-WindowRecord $fg
    targetIsForeground = ($targetHwnd -ne [IntPtr]::Zero -and $fg -eq $targetHwnd)
  })
}
function Write-Evidence {
  if (-not $script:evidenceFile) { return }
  $json = $script:record | ConvertTo-Json -Depth 12
  $bytes = [Text.UTF8Encoding]::new($false).GetBytes($json + "`n")
  $stream = [IO.File]::Open($script:evidenceFile, [IO.FileMode]::CreateNew, [IO.FileAccess]::Write, [IO.FileShare]::Read)
  try { $stream.Write($bytes, 0, $bytes.Length); $stream.Flush($true) } finally { $stream.Dispose() }
}

try {
  Add-Type -TypeDefinition $script:NativeSource -Language CSharp
  if ($SelfTestMode) {
    $dummy = [ForegroundObserveNative]::ReadWindow([IntPtr]::Zero)
    $sample = [ordered]@{ win32Type = [ForegroundObserveNative].FullName; boundsType = [ForegroundObserveNative+RECT].FullName; windowRecord = (Get-WindowRecord ([IntPtr]::Zero)); json = ([ordered]@{ok=$true;count=1} | ConvertTo-Json -Compress) }
    if ($dummy.ProcessId -ne 0 -or $sample.json -ne '{"ok":true,"count":1}') { throw 'SELFTEST_STRUCTURE_OR_JSON_FAILED' }
    Write-Output 'SELFTEST_PASS: Win32 declarations, window record structure, and JSON serialization compiled; no window activation was called.'
    exit 0
  }

  if (-not $ExecutablePath -or -not $ExpectedSha256 -or -not $EvidenceDirectory) { throw 'E_REQUIRED_ARGUMENT_MISSING' }
  $candidate = [IO.Path]::GetFullPath($ExecutablePath)
  $pinnedCandidate = 'E:\wuxian-v1-native-candidates\2cf0a43fabd8f127b59cbf13ed35ff15a0c5e62d-f4e695cd7c514acf8cdc6934a0954a2b-pinned-debug-candidate\wuxian-horror-ch1.exe'
  if (-not $candidate.Equals($pinnedCandidate, [StringComparison]::OrdinalIgnoreCase)) { throw 'E_CANDIDATE_PATH_NOT_PINNED' }
  $tmpRoot = [IO.Path]::GetFullPath([IO.Path]::GetTempPath()).TrimEnd('\') + '\'
  $evidence = [IO.Path]::GetFullPath($EvidenceDirectory)
  if (-not $evidence.StartsWith($tmpRoot, [StringComparison]::OrdinalIgnoreCase)) { throw 'E_EVIDENCE_NOT_UNDER_TEMP' }
  if (-not ([IO.Path]::GetFileName($evidence).StartsWith('native-foreground-observe-01-', [StringComparison]::OrdinalIgnoreCase))) { throw 'E_EVIDENCE_NAME_NOT_UNIQUE_PREFIX' }
  Assert-NoReparseAncestors $evidence
  if (Test-Path -LiteralPath $evidence) { throw 'E_EVIDENCE_ALREADY_EXISTS' }
  if (-not (Test-Path -LiteralPath $candidate -PathType Leaf)) { throw 'E_CANDIDATE_MISSING' }
  $expected = $ExpectedSha256.ToUpperInvariant()
  if ($expected -ne 'C35F1D7EB6180E14DE06F1C768963B94A52376872CE759FC54CFB8555786B785') { throw 'E_EXPECTED_HASH_NOT_PINNED' }
  $item = Get-Item -LiteralPath $candidate
  $sha = (Get-FileHash -LiteralPath $candidate -Algorithm SHA256).Hash.ToUpperInvariant()
  if ($item.Length -ne 260936704 -or $sha -ne $expected) { throw 'E_CANDIDATE_IDENTITY_MISMATCH' }
  $sidecar = Join-Path (Split-Path $candidate -Parent) 'bundle-identity.json'
  $sidecarItem = Get-Item -LiteralPath $sidecar
  $sidecarSha = (Get-FileHash -LiteralPath $sidecar -Algorithm SHA256).Hash.ToUpperInvariant()
  if ($sidecarItem.Length -ne 35096 -or $sidecarSha -ne 'A87E86C463BAB30F8FD191B0EE2F7C84180D1C453BBE3F2B01DCF42C6B53864E') { throw 'E_SIDECAR_IDENTITY_MISMATCH' }
  $identity = Get-Content -LiteralPath (Join-Path (Split-Path $candidate -Parent) 'identity.json') -Raw | ConvertFrom-Json
  if ($identity.exe.sha256.ToUpperInvariant() -ne $expected -or $identity.exe.lengthBytes -ne 260936704 -or $identity.buildIdentity.gitSha -ne '2cf0a43fabd8f127b59cbf13ed35ff15a0c5e62d' -or -not $identity.sourceTreeDirty) { throw 'E_CANDIDATE_IDENTITY_RECORD_MISMATCH' }
  $existing = @(Get-CimInstance Win32_Process -Filter "name='wuxian-horror-ch1.exe'" -ErrorAction Stop)
  if ($existing.Count -ne 0) { throw 'E_SAME_NAME_PROCESS_ALREADY_RUNNING' }
  [IO.Directory]::CreateDirectory($evidence) | Out-Null
  $script:evidenceFile = Join-Path $evidence 'foreground-observation.json'
  $saveDir = Join-Path $evidence 'isolated-save'
  $diagDir = Join-Path $evidence 'diagnostics'
  [IO.Directory]::CreateDirectory($saveDir) | Out-Null
  [IO.Directory]::CreateDirectory($diagDir) | Out-Null
  $script:record.candidate = [ordered]@{ path=$candidate; lengthBytes=$item.Length; sha256=$sha; sidecarLengthBytes=$sidecarItem.Length; sidecarSha256=$sidecarSha; fixedSourceGitSha=$identity.buildIdentity.gitSha; sourceTreeDirty=[bool]$identity.sourceTreeDirty }
  $script:record.status = 'RUNNING'
  $script:record.foregroundBeforeLaunch = Get-WindowRecord ([ForegroundObserveNative]::GetForegroundWindow())
  $psi = [Diagnostics.ProcessStartInfo]::new()
  $psi.FileName = $candidate; $psi.WorkingDirectory = Split-Path $candidate -Parent; $psi.UseShellExecute = $false
  $psi.Environment['WUXIAN_FORMAL_SAVE_DIR'] = $saveDir
  $psi.Environment['WUXIAN_NATIVE_DIAGNOSTICS_DIR'] = $diagDir
  $script:startedProcess = [Diagnostics.Process]::Start($psi)
  if ($null -eq $script:startedProcess) { throw 'E_PROCESS_START_FAILED' }
  $script:startedProcess.Refresh()
  $script:record.process = [ordered]@{ pid=$script:startedProcess.Id; path=(Get-ProcessPath $script:startedProcess); sha256=(Get-ProcessSha $script:startedProcess); launchedAtUtc=[DateTime]::UtcNow.ToString('o'); saveDirectory=$saveDir; diagnosticsDirectory=$diagDir }
  if ($script:record.process.path -ne $candidate -or $script:record.process.sha256 -ne $expected) { throw 'E_STARTED_PROCESS_IDENTITY_MISMATCH' }
  $deadline = [DateTime]::UtcNow.AddSeconds(25)
  do {
    $script:startedProcess.Refresh()
    if ($script:startedProcess.MainWindowHandle -ne [IntPtr]::Zero) { break }
    Start-Sleep -Milliseconds 100
  } while ([DateTime]::UtcNow -lt $deadline -and -not $script:startedProcess.HasExited)
  if ($script:startedProcess.HasExited -or $script:startedProcess.MainWindowHandle -eq [IntPtr]::Zero) { throw 'E_TARGET_WINDOW_NOT_READY' }
  $hwnd = [IntPtr]$script:startedProcess.MainWindowHandle
  Add-Observation 'pre-ShowWindow' 0 $null
  $showResult = [ForegroundObserveNative]::ShowWindow($hwnd, 9)
  Add-Observation 'post-ShowWindow' 0 ([Nullable[bool]]$showResult)
  for ($attempt=1; $attempt -le 8; $attempt++) {
    $setResult = [ForegroundObserveNative]::SetForegroundWindow($hwnd)
    Start-Sleep -Milliseconds 120
    Add-Observation 'post-SetForegroundWindow' $attempt ([Nullable[bool]]$setResult)
  }
  $script:record.status = 'OBSERVED'
} catch {
  $script:record.status = 'ERROR'
  $script:record.errors.Add($_.Exception.Message)
} finally {
  if ($script:startedProcess) {
    try {
      $script:startedProcess.Refresh()
      if (-not $script:startedProcess.HasExited) {
        $pathNow = Get-ProcessPath $script:startedProcess
        $shaNow = Get-ProcessSha $script:startedProcess
        if ($pathNow -eq $script:record.candidate.path -and $shaNow -eq $script:record.candidate.sha256 -and $script:startedProcess.Id -eq $script:record.process.pid) {
          $closeSent = $script:startedProcess.CloseMainWindow()
          $closed = $script:startedProcess.WaitForExit(3000)
          $forced = $false
          if (-not $closed) {
            $script:startedProcess.Refresh()
            if (-not $script:startedProcess.HasExited -and (Get-ProcessPath $script:startedProcess) -eq $script:record.candidate.path -and (Get-ProcessSha $script:startedProcess) -eq $script:record.candidate.sha256) {
              Stop-Process -Id $script:startedProcess.Id -Force -ErrorAction Stop
              $forced = $true
              $script:startedProcess.WaitForExit(3000) | Out-Null
            }
          }
          $script:record.cleanup = [ordered]@{ identityRechecked=$true; closeMainWindowReturn=$closeSent; exited=([bool]$script:startedProcess.HasExited); exitCode=if ($script:startedProcess.HasExited) { $script:startedProcess.ExitCode } else { $null }; forcedStopUsed=$forced }
        } else { $script:record.cleanup = [ordered]@{ identityRechecked=$false; stopped=$false; reason='process identity did not match; left running' } }
      } else { $script:record.cleanup = [ordered]@{ identityRechecked=$true; exited=$true; exitCode=$script:startedProcess.ExitCode; stoppedBy='process already exited' } }
    } catch { $script:record.errors.Add(('CLEANUP: ' + $_.Exception.Message)); $script:record.cleanup = [ordered]@{ identityRechecked=$false; stopped=$false; error=$_.Exception.Message } }
  }
  if ($script:evidenceFile -and -not (Test-Path -LiteralPath $script:evidenceFile)) {
    try { Write-Evidence } catch { $script:record.errors.Add(('EVIDENCE_WRITE: ' + $_.Exception.Message)); throw }
  }
}
if ($script:record.status -eq 'ERROR') { throw ('FOREGROUND_DIAGNOSTIC_FAILED: ' + ($script:record.errors -join '; ')) }
Write-Output ("OBSERVATION_WRITTEN: " + $script:evidenceFile)
