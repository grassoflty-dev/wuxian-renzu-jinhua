[CmdletBinding()]
param(
  [string]$ExecutablePath,
  [string]$ExpectedSha256,
  [string]$EvidenceDirectory,
  [ValidateRange(5, 60)] [int]$StartupTimeoutSeconds = 25,
  [switch]$SelfTestMode
)

$ErrorActionPreference = 'Stop'
$script:ownedProcess = $null
$script:ownedPid = $null
$script:ownedPath = $null
$script:ownedSha256 = $null
$script:evidenceStream = $null
$script:record = $null
$script:failure = $null
$script:exitCode = 2
$script:previousSaveDirectory = $env:WUXIAN_FORMAL_SAVE_DIR
$script:previousDiagnosticsDirectory = $env:WUXIAN_NATIVE_DIAGNOSTICS_DIR

$script:NativeSource = @'
using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using System.Text;
public static class NativeWindowDiagnosticApi {
  public sealed class WindowRow {
    public IntPtr Hwnd; public uint ProcessId; public string Title; public string ClassName;
    public IntPtr Owner; public IntPtr Parent; public IntPtr Root; public IntPtr RootOwner;
    public bool Visible; public bool BoundsAvailable; public int Left; public int Top; public int Width; public int Height;
  }
  [StructLayout(LayoutKind.Sequential)] private struct RECT { public int Left; public int Top; public int Right; public int Bottom; }
  private delegate bool EnumWindowsCallback(IntPtr hwnd, IntPtr state);
  [DllImport("user32.dll")] private static extern bool EnumWindows(EnumWindowsCallback callback, IntPtr lParam);
  [DllImport("user32.dll")] private static extern uint GetWindowThreadProcessId(IntPtr hwnd, out uint processId);
  [DllImport("user32.dll", CharSet=CharSet.Unicode, SetLastError=true)] private static extern int GetWindowText(IntPtr hwnd, StringBuilder text, int maxCount);
  [DllImport("user32.dll", CharSet=CharSet.Unicode, SetLastError=true)] private static extern int GetClassName(IntPtr hwnd, StringBuilder className, int maxCount);
  [DllImport("user32.dll")] private static extern IntPtr GetWindow(IntPtr hwnd, uint command);
  [DllImport("user32.dll")] private static extern IntPtr GetParent(IntPtr hwnd);
  [DllImport("user32.dll")] private static extern IntPtr GetAncestor(IntPtr hwnd, uint flags);
  [DllImport("user32.dll")] private static extern bool IsWindowVisible(IntPtr hwnd);
  [DllImport("user32.dll", SetLastError=true)] private static extern bool GetWindowRect(IntPtr hwnd, out RECT rect);
  [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
  private static string ReadText(IntPtr hwnd) { var b = new StringBuilder(2048); GetWindowText(hwnd, b, b.Capacity); return b.ToString(); }
  private static string ReadClass(IntPtr hwnd) { var b = new StringBuilder(512); GetClassName(hwnd, b, b.Capacity); return b.ToString(); }
  public static WindowRow[] EnumerateTopLevelWindows() {
    var rows = new List<WindowRow>();
    EnumWindows(delegate(IntPtr hwnd, IntPtr state) {
      uint pid; GetWindowThreadProcessId(hwnd, out pid);
      RECT r; bool hasRect = GetWindowRect(hwnd, out r);
      rows.Add(new WindowRow { Hwnd=hwnd, ProcessId=pid, Title=ReadText(hwnd), ClassName=ReadClass(hwnd),
        Owner=GetWindow(hwnd, 4), Parent=GetParent(hwnd), Root=GetAncestor(hwnd, 2), RootOwner=GetAncestor(hwnd, 3),
        Visible=IsWindowVisible(hwnd), BoundsAvailable=hasRect,
        Left=hasRect?r.Left:0, Top=hasRect?r.Top:0, Width=hasRect?r.Right-r.Left:0, Height=hasRect?r.Bottom-r.Top:0 });
      return true;
    }, IntPtr.Zero);
    return rows.ToArray();
  }
}
'@

function Initialize-NativeApi {
  if (-not ('NativeWindowDiagnosticApi' -as [type])) {
    $referenceDirectory = Join-Path ([System.AppContext]::BaseDirectory) 'ref'
    if (-not (Test-Path -LiteralPath $referenceDirectory -PathType Container)) { throw 'E_NATIVE_REFERENCE_ASSEMBLIES_UNAVAILABLE' }
    $assemblies = @(Get-ChildItem -LiteralPath $referenceDirectory -Filter '*.dll' -File | ForEach-Object FullName)
    if ($assemblies.Count -eq 0) { throw 'E_NATIVE_REFERENCE_ASSEMBLIES_EMPTY' }
    Add-Type -TypeDefinition $script:NativeSource -ReferencedAssemblies $assemblies -ErrorAction Stop
  }
}

function Format-Hwnd([IntPtr]$Hwnd) {
  if ($Hwnd -eq [IntPtr]::Zero) { return '0x0' }
  return ('0x{0:X}' -f $Hwnd.ToInt64())
}

function Test-PathInside([string]$Path, [string]$Root) {
  $p = [IO.Path]::GetFullPath($Path).TrimEnd('\','/')
  $r = [IO.Path]::GetFullPath($Root).TrimEnd('\','/')
  return $p.Equals($r,[StringComparison]::OrdinalIgnoreCase) -or $p.StartsWith($r + '\',[StringComparison]::OrdinalIgnoreCase) -or $p.StartsWith($r + '/',[StringComparison]::OrdinalIgnoreCase)
}

function Get-RepositoryRoot { return [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..\..')) }

function Assert-Inputs {
  if ([string]::IsNullOrWhiteSpace($ExecutablePath) -or -not [IO.Path]::IsPathFullyQualified($ExecutablePath)) { throw 'E_EXECUTABLE_PATH_MUST_BE_ABSOLUTE' }
  if ($ExpectedSha256 -notmatch '^[0-9A-Fa-f]{64}$') { throw 'E_EXPECTED_SHA256_INVALID' }
  if ([string]::IsNullOrWhiteSpace($EvidenceDirectory) -or -not [IO.Path]::IsPathFullyQualified($EvidenceDirectory)) { throw 'E_EVIDENCE_PATH_MUST_BE_ABSOLUTE' }
  if (-not (Test-Path -LiteralPath $ExecutablePath -PathType Leaf)) { throw 'E_EXECUTABLE_NOT_FOUND' }
  $exeItem = Get-Item -LiteralPath $ExecutablePath
  if (($exeItem.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) { throw 'E_EXECUTABLE_REPARSE_POINT' }
  if ($exeItem.Length -ne 260936704) { throw 'E_EXECUTABLE_SIZE_MISMATCH' }
  $exe = (Resolve-Path -LiteralPath $ExecutablePath).Path
  $sha = (Get-FileHash -LiteralPath $exe -Algorithm SHA256).Hash.ToUpperInvariant()
  if ($sha -ne $ExpectedSha256.ToUpperInvariant()) { throw 'E_EXECUTABLE_SHA256_MISMATCH' }
  $evidence = [IO.Path]::GetFullPath($EvidenceDirectory)
  if (Test-PathInside $evidence (Get-RepositoryRoot)) { throw 'E_EVIDENCE_DIRECTORY_INSIDE_REPOSITORY' }
  if (Test-Path -LiteralPath $evidence) {
    $item = Get-Item -LiteralPath $evidence
    if (-not $item.PSIsContainer) { throw 'E_EVIDENCE_PATH_NOT_DIRECTORY' }
    if (($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) { throw 'E_EVIDENCE_DIRECTORY_REPARSE_POINT' }
    if (@(Get-ChildItem -LiteralPath $evidence -Force).Count -ne 0) { throw 'E_EVIDENCE_DIRECTORY_NOT_EMPTY' }
  }
  $parentPath = Split-Path -Parent $evidence
  if (-not (Test-Path -LiteralPath $parentPath -PathType Container)) { throw 'E_EVIDENCE_PARENT_NOT_FOUND' }
  $cursor = Get-Item -LiteralPath (Resolve-Path -LiteralPath $parentPath).Path
  while ($null -ne $cursor) {
    if (($cursor.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) { throw 'E_EVIDENCE_PARENT_REPARSE_POINT' }
    $cursor = if ($cursor.Parent) { Get-Item -LiteralPath $cursor.Parent.FullName } else { $null }
  }
  return [pscustomobject]@{ Exe=$exe; Sha=$sha; Size=$exeItem.Length; Evidence=$evidence }
}

function Get-ProcessIdentity([int]$ProcessIdValue) {
  $p = Get-Process -Id $ProcessIdValue -ErrorAction Stop
  $p.Refresh()
  $path = [IO.Path]::GetFullPath($p.MainModule.FileName)
  $sha = (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToUpperInvariant()
  return [pscustomobject]@{ Process=$p; Path=$path; Sha=$sha; MainWindowHandle=(Format-Hwnd ([IntPtr]$p.MainWindowHandle)); MainWindowTitle=[string]$p.MainWindowTitle }
}

function Convert-WindowRow($Row) {
  return [ordered]@{
    hwnd=(Format-Hwnd $Row.Hwnd); processId=[uint32]$Row.ProcessId; title=[string]$Row.Title; className=[string]$Row.ClassName
    ownerHwnd=(Format-Hwnd $Row.Owner); parentHwnd=(Format-Hwnd $Row.Parent); rootHwnd=(Format-Hwnd $Row.Root); rootOwnerHwnd=(Format-Hwnd $Row.RootOwner)
    visible=[bool]$Row.Visible; boundsAvailable=[bool]$Row.BoundsAvailable
    bounds=[ordered]@{ left=[int]$Row.Left; top=[int]$Row.Top; width=[int]$Row.Width; height=[int]$Row.Height }
  }
}

function Get-Snapshot([int]$ProcessIdValue) {
  $timestamp = (Get-Date).ToUniversalTime().ToString('o')
  $identity = Get-ProcessIdentity $ProcessIdValue
  $foreground = [NativeWindowDiagnosticApi]::GetForegroundWindow()
  $all = @([NativeWindowDiagnosticApi]::EnumerateTopLevelWindows())
  $targetRows = @($all | Where-Object { $_.ProcessId -eq [uint32]$ProcessIdValue })
  $titles = @($targetRows | ForEach-Object { [string]$_.Title } | Select-Object -Unique)
  $titleCounts = @()
  foreach ($title in $titles) {
    $globalCount = @($all | Where-Object { [string]$_.Title -ceq $title }).Count
    $processCount = @($targetRows | Where-Object { [string]$_.Title -ceq $title }).Count
    $titleCounts += [ordered]@{ title=$title; targetPidCount=$processCount; globalCount=$globalCount }
  }
  return [ordered]@{
    sampledAtUtc=$timestamp; processId=$ProcessIdValue; mainModulePath=$identity.Path; mainModuleSha256=$identity.Sha
    processMainWindowHandle=$identity.MainWindowHandle; processMainWindowTitle=$identity.MainWindowTitle
    foregroundHwnd=(Format-Hwnd $foreground)
    exactTitleCounts=$titleCounts; targetWindowCount=$targetRows.Count
    windows=@($targetRows | ForEach-Object { Convert-WindowRow $_ })
  }
}

function Stop-OwnedProcess {
  if ($null -eq $script:ownedProcess -or $null -eq $script:ownedPid) { return }
  try { $id = Get-ProcessIdentity $script:ownedPid } catch { $script:record.process.stop='already-exited-or-unreadable'; return }
  if (-not $id.Path.Equals($script:ownedPath,[StringComparison]::OrdinalIgnoreCase)) { $script:record.process.stop='BLOCKED_PID_PATH_NO_LONGER_MATCHES'; return }
  if ($id.Sha -ne $script:ownedSha256) { $script:record.process.stop='BLOCKED_PID_SHA_NO_LONGER_MATCHES'; return }
  Stop-Process -Id $script:ownedPid -Force
  [void]$script:ownedProcess.WaitForExit(5000)
  $script:record.process.stop='stopped-owned-pid-after-path-and-sha-recheck'
  $script:record.process.stoppedAtUtc=(Get-Date).ToUniversalTime().ToString('o')
}

function Save-Evidence {
  if ($null -eq $script:record -or $null -eq $script:evidenceStream) { return }
  $script:record.completedAtUtc=(Get-Date).ToUniversalTime().ToString('o')
  $json=$script:record | ConvertTo-Json -Depth 12
  $bytes=[Text.UTF8Encoding]::new($false).GetBytes($json)
  $script:evidenceStream.SetLength(0); $script:evidenceStream.Position=0
  $script:evidenceStream.Write($bytes,0,$bytes.Length); $script:evidenceStream.Flush($true)
  $script:evidenceStream.Dispose(); $script:evidenceStream=$null
}

if ($SelfTestMode) {
  Initialize-NativeApi
  $windows=@([NativeWindowDiagnosticApi]::EnumerateTopLevelWindows())
  $sample=[ordered]@{ sampledAtUtc=(Get-Date).ToUniversalTime().ToString('o'); processId=0; windows=@(); exactTitleCounts=@() }
  $json=$sample | ConvertTo-Json -Depth 8
  $parsed=ConvertFrom-Json $json
  if ($null -eq $parsed.sampledAtUtc -or $null -eq $parsed.windows -or $null -eq $parsed.exactTitleCounts) { throw 'E_SELFTEST_JSON_SCHEMA' }
  [ordered]@{ mode='self-test-only'; nativeApi='PASS'; visibleAndHiddenTopLevelEnumeration='PASS'; enumeratedTopLevelWindowCount=$windows.Count; jsonRoundTrip='PASS'; realProcessStarted=$false; keysSent=$false; screenshotCreated=$false } | ConvertTo-Json -Depth 4
  exit 0
}

try {
  if ([Environment]::OSVersion.Platform -ne [PlatformID]::Win32NT) { throw 'E_WINDOWS_REQUIRED' }
  $inputs=Assert-Inputs
  Initialize-NativeApi
  if (-not (Test-Path -LiteralPath $inputs.Evidence)) { [void](New-Item -ItemType Directory -Path $inputs.Evidence) }
  if (@(Get-ChildItem -LiteralPath $inputs.Evidence -Force).Count -ne 0) { throw 'E_EVIDENCE_DIRECTORY_NOT_EMPTY' }
  $evidenceFile=Join-Path $inputs.Evidence 'window-diagnostic.json'
  $script:evidenceStream=[IO.File]::Open($evidenceFile,[IO.FileMode]::CreateNew,[IO.FileAccess]::ReadWrite,[IO.FileShare]::None)
  $script:ownedPath=$inputs.Exe; $script:ownedSha256=$inputs.Sha
  $isolation=[ordered]@{ saveDirectory=(Join-Path $inputs.Evidence 'isolated-save'); diagnosticsDirectory=(Join-Path $inputs.Evidence 'diagnostics'); userSaveDirectoryUsed=$false }
  [void](New-Item -ItemType Directory -Path $isolation.saveDirectory,$isolation.diagnosticsDirectory)
  $env:WUXIAN_FORMAL_SAVE_DIR=$isolation.saveDirectory; $env:WUXIAN_NATIVE_DIAGNOSTICS_DIR=$isolation.diagnosticsDirectory
  $script:record=[ordered]@{
    schemaVersion=1; package='NATIVE-WINDOW-HWND-DIAG-01'; status='BLOCKED'; startedAtUtc=(Get-Date).ToUniversalTime().ToString('o'); completedAtUtc=$null
    executable=[ordered]@{ requestedPath=$ExecutablePath; actualPath=$inputs.Exe; expectedSha256=$inputs.Sha; actualSha256=$inputs.Sha; sizeBytes=$inputs.Size; prelaunchSha256Match=$true }
    process=[ordered]@{ pid=$null; startedByHarness=$false; startedAtUtc=$null; stoppedAtUtc=$null; stop=$null }
    isolation=$isolation; sampling=[ordered]@{ requestedSamples=2; spacingMilliseconds=500; samples=@(); error=$null }
    constraints=[ordered]@{ keysSent=$false; ipcUsed=$false; uiaUsed=$false; screenshotCreated=$false; windowActivated=$false }
    claimBoundary='Diagnostic observation only. Does not establish native acceptance, F3 behavior, WebView2 content, or release eligibility.'; errors=@()
  }
  $script:ownedProcess=Start-Process -FilePath $inputs.Exe -WorkingDirectory (Split-Path -Parent $inputs.Exe) -PassThru -WindowStyle Normal
  $script:ownedPid=$script:ownedProcess.Id
  $script:record.process.pid=$script:ownedPid; $script:record.process.startedByHarness=$true; $script:record.process.startedAtUtc=(Get-Date).ToUniversalTime().ToString('o')
  $deadline=(Get-Date).AddSeconds($StartupTimeoutSeconds)
  $found=$false
  while ((Get-Date) -lt $deadline) {
    try { $identity=Get-ProcessIdentity $script:ownedPid; if ($identity.Path.Equals($inputs.Exe,[StringComparison]::OrdinalIgnoreCase) -and $identity.Sha -eq $inputs.Sha) { $found=$true; break } }
    catch { }
    Start-Sleep -Milliseconds 250
  }
  if (-not $found) { throw 'E_TARGET_PROCESS_IDENTITY_TIMEOUT_OR_MISMATCH' }
  for ($i=0;$i -lt 2;$i++) {
    if ($i -gt 0) { Start-Sleep -Milliseconds 500 }
    try { $script:record.sampling.samples += (Get-Snapshot $script:ownedPid) }
    catch { $script:record.sampling.error=$_.Exception.Message; throw }
  }
  $samples=@($script:record.sampling.samples)
  $valid=$samples.Count -eq 2 -and @($samples | Where-Object { $_.mainModulePath.Equals($inputs.Exe,[StringComparison]::OrdinalIgnoreCase) -and $_.mainModuleSha256 -eq $inputs.Sha }).Count -eq 2
  $script:record.status=if ($valid) { 'OBSERVED_DIAGNOSTIC' } else { 'BLOCKED' }
  if ($valid) { $script:exitCode=0 }
} catch {
  $script:failure=$_.Exception.Message
  if ($null -ne $script:record) { $script:record.errors+= $script:failure }
} finally {
  try { Stop-OwnedProcess } catch { if ($null -ne $script:record) { $script:record.errors+=('process cleanup: '+$_.Exception.Message) } }
  try { Save-Evidence } catch { Write-Error ('E_EVIDENCE_WRITE_FAILED: '+$_.Exception.Message); if ($null -ne $script:evidenceStream) { $script:evidenceStream.Dispose(); $script:evidenceStream=$null } }
  $env:WUXIAN_FORMAL_SAVE_DIR=$script:previousSaveDirectory
  $env:WUXIAN_NATIVE_DIAGNOSTICS_DIR=$script:previousDiagnosticsDirectory
}
if ($null -ne $script:record) {
  Write-Output ('Evidence: '+$evidenceFile)
  Write-Output ('Status: '+$script:record.status)
  if ($script:failure) { Write-Output ('Failure: '+$script:failure) }
} else { Write-Error ('Preflight failed; no process was started. '+[string]$script:failure) }
exit $script:exitCode
