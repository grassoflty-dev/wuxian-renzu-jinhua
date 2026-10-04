[CmdletBinding()]
param(
  [string]$ExecutablePath,
  [string]$ExpectedSha256,
  [string]$EvidenceDirectory,
  [ValidateRange(5, 60)] [int]$StartupTimeoutSeconds = 25,
  [ValidateRange(1, 30)] [int]$ObservationTimeoutSeconds = 10,
  [switch]$SelfTestMode
)

$ErrorActionPreference = 'Stop'
$script:record = $null
$script:evidenceRoot = $null
$script:ownedProcess = $null
$script:ownedPid = $null
$script:ownedPath = $null
$script:ownedSha256 = $null
$script:targetHwnd = [IntPtr]::Zero
$script:exitCode = 2
$script:lastFailure = $null
$script:evidenceStream = $null
$script:previousSaveDirectory = $env:WUXIAN_FORMAL_SAVE_DIR
$script:previousDiagnosticsDirectory = $env:WUXIAN_NATIVE_DIAGNOSTICS_DIR

function Get-RepositoryRoot {
  return [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..\..'))
}

function Test-PathInside([string]$Path, [string]$Root) {
  $fullPath = [IO.Path]::GetFullPath($Path).TrimEnd([IO.Path]::DirectorySeparatorChar, [IO.Path]::AltDirectorySeparatorChar)
  $fullRoot = [IO.Path]::GetFullPath($Root).TrimEnd([IO.Path]::DirectorySeparatorChar, [IO.Path]::AltDirectorySeparatorChar)
  return $fullPath.Equals($fullRoot, [StringComparison]::OrdinalIgnoreCase) -or
    $fullPath.StartsWith($fullRoot + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase) -or
    $fullPath.StartsWith($fullRoot + [IO.Path]::AltDirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)
}

function Resolve-NewEvidencePath([string]$Path) {
  if ([string]::IsNullOrWhiteSpace($Path) -or -not [IO.Path]::IsPathFullyQualified($Path)) { throw 'E_EVIDENCE_PATH_MUST_BE_ABSOLUTE' }
  $full = [IO.Path]::GetFullPath($Path)
  $repoRoot = Get-RepositoryRoot
  if (Test-PathInside $full $repoRoot) { throw 'E_EVIDENCE_DIRECTORY_INSIDE_REPOSITORY' }
  if (Test-Path -LiteralPath $full) {
    if (-not (Test-Path -LiteralPath $full -PathType Container)) { throw 'E_EVIDENCE_PATH_NOT_DIRECTORY' }
    $targetItem = Get-Item -LiteralPath $full
    if (($targetItem.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) { throw 'E_EVIDENCE_DIRECTORY_REPARSE_POINT' }
    if (@(Get-ChildItem -LiteralPath $full -Force).Count -gt 0) { throw 'E_EVIDENCE_DIRECTORY_NOT_EMPTY' }
  }
  $parentPath = Split-Path -Parent $full
  if (-not (Test-Path -LiteralPath $parentPath -PathType Container)) { throw 'E_EVIDENCE_PARENT_NOT_FOUND' }
  $parent = (Resolve-Path -LiteralPath $parentPath).Path
  $cursor = Get-Item -LiteralPath $parent
  while ($null -ne $cursor) {
    if (($cursor.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) { throw 'E_EVIDENCE_PARENT_REPARSE_POINT' }
    $cursor = if ($cursor.Parent) { Get-Item -LiteralPath $cursor.Parent.FullName } else { $null }
  }
  $resolvedTarget = Join-Path $parent ([IO.Path]::GetFileName($full))
  if (Test-PathInside $resolvedTarget $repoRoot) { throw 'E_EVIDENCE_DIRECTORY_INSIDE_REPOSITORY' }
  return $resolvedTarget
}

function Assert-IdentityInputs {
  if ([string]::IsNullOrWhiteSpace($ExecutablePath) -or -not [IO.Path]::IsPathFullyQualified($ExecutablePath)) { throw 'E_EXECUTABLE_PATH_MUST_BE_ABSOLUTE' }
  if ([string]::IsNullOrWhiteSpace($ExpectedSha256) -or $ExpectedSha256 -notmatch '^[0-9A-Fa-f]{64}$') { throw 'E_EXPECTED_SHA256_INVALID' }
  if ([string]::IsNullOrWhiteSpace($EvidenceDirectory)) { throw 'E_EVIDENCE_DIRECTORY_REQUIRED' }
  if (-not [IO.Path]::IsPathFullyQualified($EvidenceDirectory)) { throw 'E_EVIDENCE_PATH_MUST_BE_ABSOLUTE' }
  if (-not (Test-Path -LiteralPath $ExecutablePath -PathType Leaf)) { throw 'E_EXECUTABLE_NOT_FOUND' }
  $exeItem = Get-Item -LiteralPath $ExecutablePath
  if (($exeItem.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) { throw 'E_EXECUTABLE_REPARSE_POINT' }
  $resolvedExe = (Resolve-Path -LiteralPath $ExecutablePath).Path
  $actualSha = (Get-FileHash -LiteralPath $resolvedExe -Algorithm SHA256).Hash.ToUpperInvariant()
  if ($actualSha -ne $ExpectedSha256.ToUpperInvariant()) { throw 'E_EXECUTABLE_SHA256_MISMATCH' }
  $evidence = Resolve-NewEvidencePath $EvidenceDirectory
  return [pscustomobject]@{ Executable = $resolvedExe; Sha256 = $actualSha; Evidence = $evidence; SizeBytes = $exeItem.Length }
}

function Get-PreKeyFailure($Observation) {
  if (-not $Observation.ExeShaMatches) { return 'E_PREKEY_EXE_SHA_MISMATCH' }
  if (-not $Observation.PidMatches) { return 'E_PREKEY_PID_MISMATCH' }
  if (-not $Observation.ModulePathMatches) { return 'E_PREKEY_MAINMODULE_PATH_MISMATCH' }
  if (-not $Observation.ModuleShaMatches) { return 'E_PREKEY_MAINMODULE_SHA_MISMATCH' }
  if (-not $Observation.MainHwndMatches) { return 'E_PREKEY_MAIN_HWND_MISMATCH' }
  if (-not $Observation.MainWindowMetadataMatches) { return 'E_PREKEY_MAIN_WINDOW_METADATA_MISMATCH' }
  if (-not $Observation.AuxiliaryWindowsAllowed) { return 'E_PREKEY_DISALLOWED_EXTRA_WINDOW' }
  if (-not $Observation.MainWindowTitleMatches) { return 'E_PREKEY_WINDOW_TITLE_MISMATCH' }
  if (-not $Observation.ExpectedWindowTitleMatches) { return 'E_PREKEY_WINDOW_TITLE_MISMATCH' }
  if ($Observation.SameTitleWindowCount -ne 1) { return 'E_PREKEY_WINDOW_TITLE_NOT_UNIQUE' }
  if ([string]::IsNullOrWhiteSpace([string]$Observation.WindowTitle)) { return 'E_PREKEY_WINDOW_TITLE_EMPTY' }
  if (-not $Observation.ForegroundMatches) { return 'E_PREKEY_FOREGROUND_FOCUS_MISMATCH' }
  return $null
}

function Invoke-F3WithPreKeyGate($Observation, [scriptblock]$SendKey) {
  $failure = Get-PreKeyFailure $Observation
  if ($null -ne $failure) { return [pscustomobject]@{ Sent = $false; Failure = $failure } }
  & $SendKey
  return [pscustomobject]@{ Sent = $true; Failure = $null }
}

function Invoke-PreKeySimulationTests {
  $mainHwnd = [IntPtr]0x100
  $mainRow = New-SimulatedWindowRow -Hwnd $mainHwnd -ProcessIdValue 100 -Title '无限人族进化' -ClassName 'Tauri Window' -Visible $true -Left 10 -Top 20 -Width 1294 -Height 758
  $taoRow = New-SimulatedWindowRow -Hwnd ([IntPtr]0x200) -ProcessIdValue 100 -Title '' -ClassName 'Tao Thread Event Target' -Visible $true -Left 0 -Top 0 -Width 14 -Height 14
  $pseudoRow = New-SimulatedWindowRow -Hwnd ([IntPtr]0x300) -ProcessIdValue 100 -Title '' -ClassName 'PseudoConsoleWindow' -Visible $true -Left 0 -Top 0 -Width 0 -Height 0 -Owner ([IntPtr]0x900) -Parent ([IntPtr]0x900) -RootOwner ([IntPtr]0x900) -OwnerProcessId 900 -ParentProcessId 900 -RootOwnerProcessId 900
  $base = [ordered]@{
    ExeShaMatches = $true; PidMatches = $true; ModulePathMatches = $true; ModuleShaMatches = $true
    ProcessId = 100; MainWindowHandle = $mainHwnd; TargetHwnd = $mainHwnd; ProcessWindowTitle = '无限人族进化'
    Rows = @($mainRow); ForegroundMatches = $true
  }
  $cases = @(
    @{ Name = 'main-window-only'; Changes = @{}; Expected = $true },
    @{ Name = 'main-plus-two-observed-auxiliary-windows'; Changes = @{ Rows = @($mainRow,$taoRow,$pseudoRow) }; Expected = $true },
    @{ Name = 'wrong-executable-sha'; Changes = @{ ExeShaMatches = $false }; Expected = $false },
    @{ Name = 'wrong-process-id'; Changes = @{ PidMatches = $false }; Expected = $false },
    @{ Name = 'wrong-mainmodule-path'; Changes = @{ ModulePathMatches = $false }; Expected = $false },
    @{ Name = 'wrong-mainmodule-sha'; Changes = @{ ModuleShaMatches = $false }; Expected = $false },
    @{ Name = 'wrong-main-window-hwnd'; Changes = @{ MainWindowHandle = [IntPtr]0x999 }; Expected = $false },
    @{ Name = 'main-window-without-positive-bounds'; Changes = @{ Rows = @((New-SimulatedWindowRow -Hwnd $mainHwnd -ProcessIdValue 100 -Title '无限人族进化' -ClassName 'Tauri Window' -Visible $true -Left 10 -Top 20 -Width 0 -Height 0)) }; Expected = $false },
    @{ Name = 'main-window-owned-by-different-pid'; Changes = @{ Rows = @((New-SimulatedWindowRow -Hwnd $mainHwnd -ProcessIdValue 101 -Title '无限人族进化' -ClassName 'Tauri Window' -Visible $true -Left 10 -Top 20 -Width 1294 -Height 758)) }; Expected = $false },
    @{ Name = 'second-tauri-same-title-window'; Changes = @{ Rows = @($mainRow,(New-SimulatedWindowRow -Hwnd ([IntPtr]0x400) -ProcessIdValue 100 -Title '无限人族进化' -ClassName 'Tauri Window' -Visible $true -Left 0 -Top 0 -Width 640 -Height 480)) }; Expected = $false },
    @{ Name = 'unknown-empty-title-small-window'; Changes = @{ Rows = @($mainRow,(New-SimulatedWindowRow -Hwnd ([IntPtr]0x500) -ProcessIdValue 100 -Title '' -ClassName 'UnknownHelperWindow' -Visible $true -Left 0 -Top 0 -Width 14 -Height 14)) }; Expected = $false },
    @{ Name = 'tao-window-with-changed-bounds'; Changes = @{ Rows = @($mainRow,(New-SimulatedWindowRow -Hwnd ([IntPtr]0x600) -ProcessIdValue 100 -Title '' -ClassName 'Tao Thread Event Target' -Visible $true -Left 0 -Top 0 -Width 15 -Height 14)) }; Expected = $false },
    @{ Name = 'wrong-main-window-title'; Changes = @{ ProcessWindowTitle = 'Other Title' }; Expected = $false },
    @{ Name = 'foreground-changed'; Changes = @{ ForegroundMatches = $false }; Expected = $false }
  )
  foreach ($case in $cases) {
    $state = [ordered]@{}; foreach ($key in $base.Keys) { $state[$key] = $base[$key] }
    foreach ($key in $case.Changes.Keys) { $state[$key] = $case.Changes[$key] }
    $assessment = Get-WindowGateAssessment -ProcessIdValue $state.ProcessId -MainHandle $state.MainWindowHandle -TargetHandle $state.TargetHwnd -ProcessWindowTitle $state.ProcessWindowTitle -Rows $state.Rows
    $sameTitleCount = @($state.Rows | Where-Object { $_.Title -ceq '无限人族进化' }).Count
    $observation = [pscustomobject]@{
      ExeShaMatches=$state.ExeShaMatches; PidMatches=$state.PidMatches; ModulePathMatches=$state.ModulePathMatches; ModuleShaMatches=$state.ModuleShaMatches
      MainHwndMatches=$assessment.MainHwndMatches; MainWindowMetadataMatches=$assessment.MainWindowMetadataMatches; AuxiliaryWindowsAllowed=$assessment.AuxiliaryWindowsAllowed
      MainWindowTitleMatches=$assessment.MainWindowTitleMatches; ExpectedWindowTitleMatches=$assessment.ExpectedWindowTitleMatches
      VisibleWindowCount=$assessment.VisibleWindowCount; SameTitleWindowCount=$sameTitleCount; WindowTitle=$assessment.WindowTitle
      ForegroundMatches=$state.ForegroundMatches
    }
    $before = $script:simulatedKeyCount
    $result = Invoke-F3WithPreKeyGate $observation { $script:simulatedKeyCount++ }
    $sent = $script:simulatedKeyCount - $before
    $expectedSent = [bool]$case.Expected
    if ($result.Sent -ne $expectedSent -or $sent -ne [int]$expectedSent) { throw "E_SELFTEST_F3_GATE:$($case.Name)" }
    [pscustomobject]@{ name = $case.Name; gate = if ($result.Sent) { 'OPEN_SIMULATION_ONLY' } else { $result.Failure }; f3Calls = $sent; passed = $true }
  }
}

$script:NativeSource = @'
using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using System.Text;
using System.Drawing;
public static class NativeIdentityEvidenceNative {
  public sealed class WindowRow {
    public IntPtr Hwnd; public uint ProcessId; public string Title; public string ClassName;
    public IntPtr Owner; public IntPtr Parent; public IntPtr Root; public IntPtr RootOwner;
    public uint OwnerProcessId; public uint ParentProcessId; public uint RootOwnerProcessId;
    public bool Visible; public bool BoundsAvailable; public int Left; public int Top; public int Width; public int Height;
  }
  private delegate bool EnumWindowsCallback(IntPtr hwnd, IntPtr lParam);
  [DllImport("user32.dll")] private static extern bool EnumWindows(EnumWindowsCallback callback, IntPtr lParam);
  [DllImport("user32.dll")] public static extern bool IsWindow(IntPtr hwnd);
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr hwnd);
  [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr hwnd, int command);
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr hwnd);
  [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr hwnd, out uint processId);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] private static extern int GetWindowText(IntPtr hwnd, StringBuilder text, int maxCount);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] private static extern int GetClassName(IntPtr hwnd, StringBuilder className, int maxCount);
  [DllImport("user32.dll")] private static extern IntPtr GetWindow(IntPtr hwnd, uint command);
  [DllImport("user32.dll")] private static extern IntPtr GetParent(IntPtr hwnd);
  [DllImport("user32.dll")] private static extern IntPtr GetAncestor(IntPtr hwnd, uint flags);
  [DllImport("user32.dll", SetLastError=true)] public static extern bool GetWindowRect(IntPtr hwnd, out RECT rect);
  [DllImport("user32.dll")] public static extern void keybd_event(byte key, byte scan, uint flags, UIntPtr extraInfo);
  [DllImport("kernel32.dll", CharSet=CharSet.Unicode, SetLastError=true)] private static extern bool CreateDirectoryW(string path, IntPtr securityAttributes);
  [StructLayout(LayoutKind.Sequential)] public struct RECT { public int Left; public int Top; public int Right; public int Bottom; }
  public static string Title(IntPtr hwnd) { var b = new StringBuilder(1024); GetWindowText(hwnd, b, b.Capacity); return b.ToString(); }
  public static string ClassName(IntPtr hwnd) { var b = new StringBuilder(512); GetClassName(hwnd, b, b.Capacity); return b.ToString(); }
  private static uint ProcessIdFor(IntPtr hwnd) { uint pid; if (hwnd == IntPtr.Zero) return 0; GetWindowThreadProcessId(hwnd, out pid); return pid; }
  public static WindowRow[] VisibleTopLevelWindows() {
    var rows = new List<WindowRow>();
    EnumWindows(delegate(IntPtr hwnd, IntPtr state) {
      if (!IsWindowVisible(hwnd)) return true;
      uint pid; GetWindowThreadProcessId(hwnd, out pid);
      IntPtr owner = GetWindow(hwnd, 4), parent = GetParent(hwnd), root = GetAncestor(hwnd, 2), rootOwner = GetAncestor(hwnd, 3);
      RECT rect; bool hasRect = GetWindowRect(hwnd, out rect);
      rows.Add(new WindowRow { Hwnd=hwnd, ProcessId=pid, Title=Title(hwnd), ClassName=ClassName(hwnd), Owner=owner, Parent=parent, Root=root, RootOwner=rootOwner,
        OwnerProcessId=ProcessIdFor(owner), ParentProcessId=ProcessIdFor(parent), RootOwnerProcessId=ProcessIdFor(rootOwner), Visible=true, BoundsAvailable=hasRect,
        Left=hasRect?rect.Left:0, Top=hasRect?rect.Top:0, Width=hasRect?rect.Right-rect.Left:0, Height=hasRect?rect.Bottom-rect.Top:0 });
      return true;
    }, IntPtr.Zero);
    return rows.ToArray();
  }
  public static int[] Bounds(IntPtr hwnd) {
    RECT r; if (!GetWindowRect(hwnd, out r)) return new int[0];
    return new int[] { r.Left, r.Top, r.Right-r.Left, r.Bottom-r.Top };
  }
  public static bool SendF3IfForeground(IntPtr target) {
    if (GetForegroundWindow() != target) return false;
    keybd_event(0x72, 0, 0, UIntPtr.Zero);
    keybd_event(0x72, 0, 2, UIntPtr.Zero);
    return true;
  }
  public static bool CreateNewDirectory(string path) {
    if (CreateDirectoryW(path, IntPtr.Zero)) return true;
    int error = Marshal.GetLastWin32Error();
    if (error == 183) return false;
    throw new InvalidOperationException("Could not create exclusive evidence directory. Win32 error " + error.ToString());
  }
  public static void CaptureWindow(string path, int x, int y, int width, int height) {
    if (width <= 0 || height <= 0) throw new ArgumentOutOfRangeException("windowBounds");
    using (var image = new Bitmap(width, height)) using (var graphics = Graphics.FromImage(image)) {
      graphics.CopyFromScreen(x, y, 0, 0, new Size(width, height), CopyPixelOperation.SourceCopy);
      using (var stream = new System.IO.FileStream(path, System.IO.FileMode.CreateNew, System.IO.FileAccess.Write, System.IO.FileShare.None)) {
        image.Save(stream, System.Drawing.Imaging.ImageFormat.Png);
      }
    }
  }
}
'@

function Initialize-NativeIdentityEvidence {
  if (-not ('NativeIdentityEvidenceNative' -as [type])) {
    # Add-Type replaces its default references when -ReferencedAssemblies is
    # specified. Use PowerShell's matching .NET reference assemblies plus the
    # runtime's Windows System.Drawing implementation so core/generic and
    # forwarded drawing types resolve together.
    $referenceDirectory = Join-Path ([System.AppContext]::BaseDirectory) 'ref'
    $drawingAssemblies = @('System.Drawing.Common.dll','System.Private.Windows.GdiPlus.dll','System.Private.Windows.Core.dll') | ForEach-Object {
      $path = Join-Path ([System.AppContext]::BaseDirectory) $_
      if (-not (Test-Path -LiteralPath $path -PathType Leaf)) { throw "E_NATIVE_DRAWING_ASSEMBLY_UNAVAILABLE:$_" }
      $path
    }
    if (-not (Test-Path -LiteralPath $referenceDirectory -PathType Container)) { throw 'E_NATIVE_REFERENCE_ASSEMBLIES_UNAVAILABLE' }
    $assemblies = @(Get-ChildItem -LiteralPath $referenceDirectory -Filter '*.dll' -File | ForEach-Object FullName)
    if ($assemblies.Count -eq 0) { throw 'E_NATIVE_REFERENCE_ASSEMBLIES_EMPTY' }
    $assemblies += $drawingAssemblies
    Add-Type -TypeDefinition $script:NativeSource -ReferencedAssemblies $assemblies -ErrorAction Stop
  }
  Add-Type -AssemblyName UIAutomationClient -ErrorAction Stop
  Add-Type -AssemblyName UIAutomationTypes -ErrorAction Stop
}

function New-SimulatedWindowRow {
  param(
    [IntPtr]$Hwnd,
    [uint32]$ProcessIdValue,
    [string]$Title,
    [string]$ClassName,
    [bool]$Visible,
    [int]$Left,
    [int]$Top,
    [int]$Width,
    [int]$Height,
    [IntPtr]$Owner = [IntPtr]::Zero,
    [IntPtr]$Parent = [IntPtr]::Zero,
    [IntPtr]$Root = [IntPtr]::Zero,
    [IntPtr]$RootOwner = [IntPtr]::Zero,
    [uint32]$OwnerProcessId = 0,
    [uint32]$ParentProcessId = 0,
    [uint32]$RootOwnerProcessId = 0,
    [bool]$BoundsAvailable = $true
  )
  if ($Root -eq [IntPtr]::Zero) { $Root = $Hwnd }
  if ($RootOwner -eq [IntPtr]::Zero) { $RootOwner = $Hwnd }
  return [pscustomobject]@{
    Hwnd=$Hwnd; ProcessId=$ProcessIdValue; Title=$Title; ClassName=$ClassName; Owner=$Owner; Parent=$Parent; Root=$Root; RootOwner=$RootOwner
    OwnerProcessId=$OwnerProcessId; ParentProcessId=$ParentProcessId; RootOwnerProcessId=$RootOwnerProcessId
    Visible=$Visible; BoundsAvailable=$BoundsAvailable; Left=$Left; Top=$Top; Width=$Width; Height=$Height
  }
}

function Get-WindowGateAssessment {
  param(
    [uint32]$ProcessIdValue,
    [IntPtr]$MainHandle,
    [IntPtr]$TargetHandle,
    [string]$ProcessWindowTitle,
    $Rows
  )
  $allRows=@($Rows)
  $ownedRows=@($allRows | Where-Object { $_.ProcessId -eq $ProcessIdValue -and $_.Visible })
  $mainRows=@($ownedRows | Where-Object { $_.Hwnd -eq $TargetHandle })
  $mainRow=if($mainRows.Count -eq 1){$mainRows[0]}else{$null}
  $expectedTitle='无限人族进化'
  $windowTitle=if($null -ne $mainRow){[string]$mainRow.Title}else{''}
  $mainHwndMatches=($MainHandle -ne [IntPtr]::Zero -and $TargetHandle -ne [IntPtr]::Zero -and $MainHandle -eq $TargetHandle -and $mainRows.Count -eq 1)
  $mainMetadataMatches=($null -ne $mainRow -and $mainRow.ProcessId -eq $ProcessIdValue -and $mainRow.ClassName -ceq 'Tauri Window' -and
    $mainRow.Title -ceq $expectedTitle -and $mainRow.Visible -and $mainRow.BoundsAvailable -and $mainRow.Width -gt 0 -and $mainRow.Height -gt 0)
  $titleMatches=($null -ne $mainRow -and ([string]$ProcessWindowTitle).Equals($windowTitle,[StringComparison]::Ordinal))
  $expectedTitleMatches=($windowTitle.Equals($expectedTitle,[StringComparison]::Ordinal) -and ([string]$ProcessWindowTitle).Equals($expectedTitle,[StringComparison]::Ordinal))
  $extraRows=@($ownedRows | Where-Object { $_.Hwnd -ne $TargetHandle })
  $auxKinds=@(); $auxAllowed=$true
  foreach($row in $extraRows){
    $isTao=($row.Title -ceq '' -and $row.ClassName -ceq 'Tao Thread Event Target' -and $row.Visible -and $row.BoundsAvailable -and
      $row.Left -eq 0 -and $row.Top -eq 0 -and $row.Width -eq 14 -and $row.Height -eq 14 -and
      $row.Owner -eq [IntPtr]::Zero -and $row.Parent -eq [IntPtr]::Zero -and $row.Root -eq $row.Hwnd -and $row.RootOwner -eq $row.Hwnd)
    $isPseudo=($row.Title -ceq '' -and $row.ClassName -ceq 'PseudoConsoleWindow' -and $row.Visible -and $row.BoundsAvailable -and
      $row.Left -eq 0 -and $row.Top -eq 0 -and $row.Width -eq 0 -and $row.Height -eq 0 -and
      $row.Owner -ne [IntPtr]::Zero -and $row.Parent -eq $row.Owner -and $row.Root -eq $row.Hwnd -and $row.RootOwner -eq $row.Owner)
    if($isTao){$auxKinds+='TaoThreadEventTarget'}elseif($isPseudo){$auxKinds+='PseudoConsoleWindow'}else{$auxAllowed=$false}
  }
  if(@($auxKinds | Where-Object {$_ -eq 'TaoThreadEventTarget'}).Count -gt 1 -or @($auxKinds | Where-Object {$_ -eq 'PseudoConsoleWindow'}).Count -gt 1){$auxAllowed=$false}
  $sameTitleRows=@($allRows | Where-Object { $_.Visible -and $_.Title -ceq $expectedTitle -and -not [string]::IsNullOrWhiteSpace([string]$_.Title) })
  return [pscustomobject]@{
    MainHwndMatches=$mainHwndMatches; MainWindowMetadataMatches=$mainMetadataMatches; AuxiliaryWindowsAllowed=$auxAllowed
    MainWindowTitleMatches=$titleMatches; ExpectedWindowTitleMatches=$expectedTitleMatches; WindowTitle=$windowTitle
    VisibleWindowCount=$ownedRows.Count; SameTitleWindowCount=$sameTitleRows.Count; AuxiliaryWindowKinds=@($auxKinds)
    MainWindowClass=if($null -ne $mainRow){[string]$mainRow.ClassName}else{$null}
    MainWindowBounds=if($null -ne $mainRow){@($mainRow.Left,$mainRow.Top,$mainRow.Width,$mainRow.Height)}else{@()}
    VisibleWindowRows=@($ownedRows | ForEach-Object {
      [ordered]@{ hwnd=('0x{0:X}' -f $_.Hwnd.ToInt64()); processId=[uint32]$_.ProcessId; title=[string]$_.Title; className=[string]$_.ClassName;
        ownerHwnd=('0x{0:X}' -f $_.Owner.ToInt64()); parentHwnd=('0x{0:X}' -f $_.Parent.ToInt64()); rootHwnd=('0x{0:X}' -f $_.Root.ToInt64()); rootOwnerHwnd=('0x{0:X}' -f $_.RootOwner.ToInt64());
        ownerProcessId=[uint32]$_.OwnerProcessId; parentProcessId=[uint32]$_.ParentProcessId; rootOwnerProcessId=[uint32]$_.RootOwnerProcessId;
        visible=[bool]$_.Visible; boundsAvailable=[bool]$_.BoundsAvailable; bounds=@{left=[int]$_.Left;top=[int]$_.Top;width=[int]$_.Width;height=[int]$_.Height} }
    })
  }
}

if ($SelfTestMode) {
  Initialize-NativeIdentityEvidence
  $visibleWindows = @([NativeIdentityEvidenceNative]::VisibleTopLevelWindows())
  $script:simulatedKeyCount = 0
  $simulationResults = @(Invoke-PreKeySimulationTests)
  $expectedSimulatedKeyCount = @($simulationResults | Where-Object { $_.f3Calls -eq 1 }).Count
  if ($script:simulatedKeyCount -ne $expectedSimulatedKeyCount) { throw 'E_SELFTEST_SIMULATED_KEY_COUNT' }
  if ($expectedSimulatedKeyCount -ne 2) { throw 'E_SELFTEST_POSITIVE_CASE_COUNT' }
  [ordered]@{
    mode = 'simulation-only'
    nativeApi = [ordered]@{ compile = 'PASS'; visibleTopLevelWindowEnumeration = 'PASS'; visibleWindowCount = $visibleWindows.Count }
    realF3Sent = $false
    simulatedF3Calls = $script:simulatedKeyCount
    results = $simulationResults
  } | ConvertTo-Json -Depth 5
  exit 0
}

function Get-ProcessModulePath([System.Diagnostics.Process]$Process) {
  try { return [IO.Path]::GetFullPath($Process.MainModule.FileName) } catch { return $null }
}

function Get-WindowRows {
  return @([NativeIdentityEvidenceNative]::VisibleTopLevelWindows())
}

function Set-TargetForeground([IntPtr]$Hwnd) {
  [void][NativeIdentityEvidenceNative]::ShowWindow($Hwnd, 9)
  for ($attempt = 0; $attempt -lt 8; $attempt++) {
    [void][NativeIdentityEvidenceNative]::SetForegroundWindow($Hwnd)
    Start-Sleep -Milliseconds 120
    if ([NativeIdentityEvidenceNative]::GetForegroundWindow() -eq $Hwnd) { return $true }
  }
  return ([NativeIdentityEvidenceNative]::GetForegroundWindow() -eq $Hwnd)
}

function Get-PreKeyObservation {
  $current = Get-Process -Id $script:ownedPid -ErrorAction SilentlyContinue
  if ($null -eq $current) { throw 'E_TARGET_PROCESS_EXITED_BEFORE_F3' }
  $modulePath = Get-ProcessModulePath $current
  $moduleSha = if ($null -ne $modulePath -and (Test-Path -LiteralPath $modulePath -PathType Leaf)) { (Get-FileHash -LiteralPath $modulePath -Algorithm SHA256).Hash.ToUpperInvariant() } else { $null }
  $rows = @(Get-WindowRows)
  $mainHandle = [IntPtr]$current.MainWindowHandle
  $foreground = [NativeIdentityEvidenceNative]::GetForegroundWindow()
  $assessment = Get-WindowGateAssessment -ProcessIdValue ([uint32]$current.Id) -MainHandle $mainHandle -TargetHandle $script:targetHwnd -ProcessWindowTitle ([string]$current.MainWindowTitle) -Rows $rows
  return [pscustomobject]@{
    ExeShaMatches = ($script:ownedSha256 -eq $ExpectedSha256.ToUpperInvariant())
    PidMatches = ($current.Id -eq $script:ownedPid)
    ModulePathMatches = ($null -ne $modulePath -and $modulePath.Equals($script:ownedPath, [StringComparison]::OrdinalIgnoreCase))
    ModuleShaMatches = ($moduleSha -eq $script:ownedSha256)
    MainHwndMatches = $assessment.MainHwndMatches
    MainWindowMetadataMatches = $assessment.MainWindowMetadataMatches
    AuxiliaryWindowsAllowed = $assessment.AuxiliaryWindowsAllowed
    MainWindowTitleMatches = $assessment.MainWindowTitleMatches
    ExpectedWindowTitleMatches = $assessment.ExpectedWindowTitleMatches
    VisibleWindowCount = $assessment.VisibleWindowCount
    SameTitleWindowCount = $assessment.SameTitleWindowCount
    WindowTitle = $assessment.WindowTitle
    MainWindowClass = $assessment.MainWindowClass
    MainWindowBounds = $assessment.MainWindowBounds
    AuxiliaryWindowKinds = $assessment.AuxiliaryWindowKinds
    VisibleWindowRows = $assessment.VisibleWindowRows
    MainWindowTitle = [string]$current.MainWindowTitle
    Hwnd = ('0x{0:X}' -f $script:targetHwnd.ToInt64())
    ForegroundHwnd = ('0x{0:X}' -f $foreground.ToInt64())
    ForegroundMatches = ($foreground -eq $script:targetHwnd)
    ModulePath = $modulePath
    ModuleSha256 = $moduleSha
    MainWindowHandle = ('0x{0:X}' -f $mainHandle.ToInt64())
    VisibleWindowCountForProcess = $assessment.VisibleWindowCount
    SameTitleWindowCountGlobal = $assessment.SameTitleWindowCount
  }
}

function Get-UiaTexts {
  $root = [System.Windows.Automation.AutomationElement]::FromHandle($script:targetHwnd)
  if ($null -eq $root) { throw 'E_UIA_ROOT_UNAVAILABLE' }
  $items = $root.FindAll([System.Windows.Automation.TreeScope]::Descendants, [System.Windows.Automation.Condition]::TrueCondition)
  $texts = [Collections.Generic.List[string]]::new()
  $count = [Math]::Min($items.Count, 1600)
  for ($i = 0; $i -lt $count; $i++) {
    try {
      $element = $items.Item($i)
      if ($element.Current.IsOffscreen) { continue }
      $name = [string]$element.Current.Name
      if (-not [string]::IsNullOrWhiteSpace($name)) { $texts.Add($name) }
      $pattern = $null
      if ($element.TryGetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern, [ref]$pattern)) {
        $value = [string]$pattern.Current.Value
        if (-not [string]::IsNullOrWhiteSpace($value)) { $texts.Add($value) }
      }
      $pattern = $null
      if ($element.TryGetCurrentPattern([System.Windows.Automation.TextPattern]::Pattern, [ref]$pattern)) {
        $value = [string]$pattern.DocumentRange.GetText(20000)
        if (-not [string]::IsNullOrWhiteSpace($value)) { $texts.Add($value) }
      }
    } catch { continue }
  }
  return @($texts | Select-Object -Unique)
}

function Get-ObservedEntryStatus([string[]]$Texts) {
  $all = $Texts -join "`n"
  $passText = $all.Contains('当前 WebView 可取回入口字节与 Rust 锁定入口一致')
  $unsatisfiedText = $all.Contains('入口字节核验：UNSATISFIED') -or $all.Contains('UNSATISFIED：')
  if ($passText -and -not $unsatisfiedText) { return 'PASS' }
  if ($unsatisfiedText -and -not $passText) { return 'UNSATISFIED' }
  return 'UNREADABLE'
}

function Capture-TargetWindow {
  if ([NativeIdentityEvidenceNative]::GetForegroundWindow() -ne $script:targetHwnd) { throw 'E_SCREENSHOT_TARGET_NOT_FOREGROUND' }
  $bounds = [NativeIdentityEvidenceNative]::Bounds($script:targetHwnd)
  if ($bounds.Count -ne 4 -or $bounds[2] -le 0 -or $bounds[3] -le 0) { throw 'E_SCREENSHOT_WINDOW_BOUNDS_UNAVAILABLE' }
  $path = Join-Path $script:evidenceRoot 'f3-window.png'
  [NativeIdentityEvidenceNative]::CaptureWindow($path, $bounds[0], $bounds[1], $bounds[2], $bounds[3])
  $item = Get-Item -LiteralPath $path
  if ($item.Length -le 0) { throw 'E_SCREENSHOT_EMPTY' }
  return [ordered]@{ status = 'PASS'; path = $path; sizeBytes = $item.Length; sha256 = (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant(); bounds = @($bounds); capture = 'CopyFromScreen visible desktop window' }
}

function Save-Evidence {
  if ($null -eq $script:record) {
    if ($null -ne $script:evidenceStream) { $script:evidenceStream.Dispose(); $script:evidenceStream = $null }
    return
  }
  if ([string]::IsNullOrWhiteSpace($script:evidenceRoot)) { return }
  $script:record.completedAtUtc = (Get-Date).ToUniversalTime().ToString('o')
  $json = $script:record | ConvertTo-Json -Depth 10
  $stream = $script:evidenceStream
  if ($null -eq $stream) { throw 'E_EVIDENCE_FILE_RESERVATION_MISSING' }
  try {
    $bytes = [Text.UTF8Encoding]::new($false).GetBytes($json)
    $stream.SetLength(0)
    $stream.Position = 0
    $stream.Write($bytes, 0, $bytes.Length)
    $stream.Flush($true)
  } finally { $stream.Dispose(); $script:evidenceStream = $null }
}

function Stop-OwnedProcess {
  if ($null -eq $script:ownedProcess -or $null -eq $script:ownedPid) { return }
  $current = Get-Process -Id $script:ownedPid -ErrorAction SilentlyContinue
  if ($null -eq $current) { $script:record.process.stop = 'already-exited'; $script:record.process.stoppedAtUtc = (Get-Date).ToUniversalTime().ToString('o'); return }
  $path = Get-ProcessModulePath $current
  if ($null -eq $path -or -not $path.Equals($script:ownedPath, [StringComparison]::OrdinalIgnoreCase)) {
    $script:record.process.stop = 'BLOCKED_PID_PATH_NO_LONGER_MATCHES'
    return
  }
  $hash = (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToUpperInvariant()
  if ($hash -ne $script:ownedSha256) { $script:record.process.stop = 'BLOCKED_PID_SHA_NO_LONGER_MATCHES'; return }
  Stop-Process -Id $script:ownedPid -Force
  [void]$script:ownedProcess.WaitForExit(5000)
  $script:record.process.stop = 'stopped-tool-owned-pid-after-path-and-sha-recheck'
  $script:record.process.stoppedAtUtc = (Get-Date).ToUniversalTime().ToString('o')
}

try {
  if ([Environment]::OSVersion.Platform -ne [PlatformID]::Win32NT) { throw 'E_WINDOWS_REQUIRED' }
  $inputs = Assert-IdentityInputs
  Initialize-NativeIdentityEvidence
  if (-not (Test-Path -LiteralPath $inputs.Evidence)) {
    if (-not [NativeIdentityEvidenceNative]::CreateNewDirectory($inputs.Evidence) -and
        (@(Get-ChildItem -LiteralPath $inputs.Evidence -Force -ErrorAction SilentlyContinue).Count -gt 0)) { throw 'E_EVIDENCE_DIRECTORY_NOT_EMPTY' }
  }
  if (@(Get-ChildItem -LiteralPath $inputs.Evidence -Force).Count -gt 0) { throw 'E_EVIDENCE_DIRECTORY_NOT_EMPTY' }
  $script:evidenceRoot = $inputs.Evidence
  $script:evidenceStream = [IO.File]::Open((Join-Path $script:evidenceRoot 'identity-evidence.json'), [IO.FileMode]::CreateNew, [IO.FileAccess]::ReadWrite, [IO.FileShare]::None)
  $script:ownedPath = $inputs.Executable
  $script:ownedSha256 = $inputs.Sha256
  $script:record = [ordered]@{
    schemaVersion = 1
    status = 'BLOCKED_PRE_F3'
    startedAtUtc = (Get-Date).ToUniversalTime().ToString('o')
    completedAtUtc = $null
    executable = [ordered]@{ requestedPath = $ExecutablePath; actualPath = $inputs.Executable; expectedSha256 = $inputs.Sha256; actualSha256 = $inputs.Sha256; sizeBytes = $inputs.SizeBytes; prelaunchSha256Match = $true }
    process = [ordered]@{ pid = $null; mainModulePath = $null; mainModuleSha256 = $null; startedByHarness = $false; startedAtUtc = $null; stoppedAtUtc = $null; stop = $null }
    window = [ordered]@{ observedAtUtc = $null; title = $null; mainWindowTitle = $null; hwnd = $null; processMainWindowHandle = $null; mainWindowClass = $null; mainWindowBounds = @(); auxiliaryWindowKinds = @(); visibleWindowRows = @(); preF3UniqueProcessWindowCount = 0; preF3UniqueTitleCount = 0; preF3InitialGate = $null; preF3SendGate = $null }
    isolation = [ordered]@{ saveDirectory = (Join-Path $script:evidenceRoot 'isolated-save'); diagnosticsDirectory = (Join-Path $script:evidenceRoot 'diagnostics'); userSaveDirectoryUsed = $false }
    focus = [ordered]@{ status = 'BLOCKED'; foregroundHwndBeforeF3 = $null; matchedBeforeF3 = $false; matchedAfterF3 = $false }
    f3 = [ordered]@{ keySent = $false; sentAtUtc = $null; observedAtUtc = $null; entryByteStatus = 'BLOCKED'; observedText = @(); uiaStatus = 'NOT_READ'; uiaError = $null }
    screenshot = [ordered]@{ status = 'BLOCKED'; capturedAtUtc = $null; path = $null; sizeBytes = $null; sha256 = $null; bounds = @(); error = $null }
    claimBoundary = [ordered]@{ nativeVerified = $false; installerVerified = $false; publicReleaseEligible = $false; scope = 'Only direct F3 text and visible window bytes are observed; this does not establish full bundle closure, EXE or installer acceptance, or release eligibility.' }
    errors = @()
  }

  [void](New-Item -ItemType Directory -Path $script:record.isolation.saveDirectory, $script:record.isolation.diagnosticsDirectory)
  $env:WUXIAN_FORMAL_SAVE_DIR = $script:record.isolation.saveDirectory
  $env:WUXIAN_NATIVE_DIAGNOSTICS_DIR = $script:record.isolation.diagnosticsDirectory
  $script:ownedProcess = Start-Process -FilePath $inputs.Executable -WorkingDirectory (Split-Path -Parent $inputs.Executable) -PassThru -WindowStyle Normal
  $script:ownedPid = $script:ownedProcess.Id
  $script:record.process.pid = $script:ownedPid
  $script:record.process.startedByHarness = $true
  $script:record.process.startedAtUtc = (Get-Date).ToUniversalTime().ToString('o')
  $deadline = (Get-Date).AddSeconds($StartupTimeoutSeconds)
  $modulePath = $null; $mainHandle = [IntPtr]::Zero; $windowTitle = ''
  while ((Get-Date) -lt $deadline) {
    $process = Get-Process -Id $script:ownedPid -ErrorAction SilentlyContinue
    if ($null -eq $process) { throw 'E_TARGET_PROCESS_EXITED_DURING_STARTUP' }
    try { $process.Refresh(); $modulePath = Get-ProcessModulePath $process; $mainHandle = [IntPtr]$process.MainWindowHandle } catch {}
    if ($mainHandle -ne [IntPtr]::Zero -and $null -ne $modulePath) { $windowTitle = [NativeIdentityEvidenceNative]::Title($mainHandle); if (-not [string]::IsNullOrWhiteSpace($windowTitle)) { break } }
    Start-Sleep -Milliseconds 200
  }
  if ($mainHandle -eq [IntPtr]::Zero -or [string]::IsNullOrWhiteSpace($windowTitle)) { throw 'E_MAIN_WINDOW_TIMEOUT_OR_TITLE_UNAVAILABLE' }
  $script:targetHwnd = $mainHandle
  $moduleSha = (Get-FileHash -LiteralPath $modulePath -Algorithm SHA256).Hash.ToUpperInvariant()
  $script:record.process.mainModulePath = $modulePath
  $script:record.process.mainModuleSha256 = $moduleSha
  $script:record.window.observedAtUtc = (Get-Date).ToUniversalTime().ToString('o')
  $script:record.window.title = $windowTitle
  $script:record.window.mainWindowTitle = [string]$script:ownedProcess.MainWindowTitle
  $script:record.window.hwnd = '0x{0:X}' -f $mainHandle.ToInt64()
  if (-not $modulePath.Equals($inputs.Executable, [StringComparison]::OrdinalIgnoreCase)) { throw 'E_MAINMODULE_PATH_MISMATCH' }
  if ($moduleSha -ne $inputs.Sha256) { throw 'E_MAINMODULE_SHA256_MISMATCH' }
  if (-not ([string]$script:ownedProcess.MainWindowTitle).Equals($windowTitle, [StringComparison]::Ordinal)) { throw 'E_MAIN_WINDOW_TITLE_MISMATCH' }

  [void](Set-TargetForeground $mainHandle)
  $observation = Get-PreKeyObservation
  $script:record.window.preF3UniqueProcessWindowCount = $observation.VisibleWindowCount
  $script:record.window.preF3UniqueTitleCount = $observation.SameTitleWindowCount
  $script:record.window.processMainWindowHandle = $observation.MainWindowHandle
  $script:record.window.mainWindowClass = $observation.MainWindowClass
  $script:record.window.mainWindowBounds = @($observation.MainWindowBounds)
  $script:record.window.auxiliaryWindowKinds = @($observation.AuxiliaryWindowKinds)
  $script:record.window.visibleWindowRows = @($observation.VisibleWindowRows)
  $script:record.window.preF3InitialGate = [ordered]@{ mainHwndMatches=$observation.MainHwndMatches; mainWindowMetadataMatches=$observation.MainWindowMetadataMatches; auxiliaryWindowsAllowed=$observation.AuxiliaryWindowsAllowed; mainWindowTitleMatches=$observation.MainWindowTitleMatches; expectedWindowTitleMatches=$observation.ExpectedWindowTitleMatches; sameTitleWindowCount=$observation.SameTitleWindowCount; foregroundMatches=$observation.ForegroundMatches }
  $script:record.focus.foregroundHwndBeforeF3 = $observation.ForegroundHwnd
  $script:record.focus.matchedBeforeF3 = [bool]$observation.ForegroundMatches
  $script:record.focus.status = if ($observation.ForegroundMatches) { 'PASS' } else { 'BLOCKED' }
  $gate = Invoke-F3WithPreKeyGate $observation {
    $fresh = Get-PreKeyObservation
    $script:record.window.preF3SendGate = [ordered]@{ processMainWindowHandle=$fresh.MainWindowHandle; hwnd=$fresh.Hwnd; mainHwndMatches=$fresh.MainHwndMatches; mainWindowMetadataMatches=$fresh.MainWindowMetadataMatches; auxiliaryWindowsAllowed=$fresh.AuxiliaryWindowsAllowed; visibleWindowCount=$fresh.VisibleWindowCount; sameTitleWindowCount=$fresh.SameTitleWindowCount; foregroundHwnd=$fresh.ForegroundHwnd; foregroundMatches=$fresh.ForegroundMatches; modulePath=$fresh.ModulePath; moduleSha256=$fresh.ModuleSha256 }
    $freshFailure = Get-PreKeyFailure $fresh
    if ($null -ne $freshFailure) { throw $freshFailure }
    if (-not [NativeIdentityEvidenceNative]::SendF3IfForeground($script:targetHwnd)) { throw 'E_PREKEY_FOREGROUND_CHANGED' }
  }
  if (-not $gate.Sent) {
    $script:record.f3.entryByteStatus = 'BLOCKED'
    $script:record.errors += $gate.Failure
    if ($gate.Failure -match 'FOREGROUND') { $script:record.focus.status = 'BLOCKED' }
    throw $gate.Failure
  }
  $script:record.f3.keySent = $true
  $script:record.f3.sentAtUtc = (Get-Date).ToUniversalTime().ToString('o')
  Start-Sleep -Milliseconds 300
  $script:record.focus.matchedAfterF3 = ([NativeIdentityEvidenceNative]::GetForegroundWindow() -eq $mainHandle)
  if (-not $script:record.focus.matchedAfterF3) { $script:record.focus.status = 'BLOCKED' }

  $lastTexts = @(); $lastError = $null; $previousClass = $null; $stableCount = 0; $uiaDeadline = (Get-Date).AddSeconds($ObservationTimeoutSeconds)
  while ((Get-Date) -lt $uiaDeadline) {
    try {
      $texts = @(Get-UiaTexts)
      $lastTexts = $texts
      $classification = Get-ObservedEntryStatus $texts
      if ($classification -ne 'UNREADABLE' -and $classification -eq $previousClass) { $stableCount++ } else { $stableCount = 0 }
      $previousClass = $classification
      if ($stableCount -ge 1) { break }
    } catch { $lastError = $_.Exception.Message }
    Start-Sleep -Milliseconds 300
  }
  $script:record.f3.observedText = @($lastTexts)
  if ($stableCount -ge 1) {
    $script:record.f3.entryByteStatus = $previousClass
    $script:record.f3.uiaStatus = 'PASS_READABLE_STABLE'
  } else {
    $script:record.f3.entryByteStatus = 'UNREADABLE'
    $script:record.f3.uiaStatus = 'BLOCKED'
    $script:record.f3.uiaError = $lastError
  }
  $script:record.f3.observedAtUtc = (Get-Date).ToUniversalTime().ToString('o')

  try {
    $capture = Capture-TargetWindow
    $capture.capturedAtUtc = (Get-Date).ToUniversalTime().ToString('o')
    $script:record.screenshot = $capture
  } catch {
    $script:record.screenshot.status = 'BLOCKED'
    $script:record.screenshot.error = $_.Exception.Message
  }
  if ($script:record.focus.status -eq 'BLOCKED') {
    $script:record.status = 'BLOCKED_FOCUS'
  } elseif ($script:record.screenshot.status -ne 'PASS') {
    $script:record.status = 'BLOCKED_SCREENSHOT'
  } elseif ($script:record.f3.entryByteStatus -eq 'UNREADABLE') {
    $script:record.status = 'BLOCKED_UIA_UNREADABLE'
  } elseif ($script:record.f3.entryByteStatus -eq 'PASS') {
    $script:record.status = 'OBSERVED_NARROW_ENTRY_PASS'
    $script:exitCode = 0
  } else {
    $script:record.status = 'OBSERVED_UNSATISFIED'
    $script:exitCode = 3
  }
} catch {
  $script:lastFailure = $_.Exception.Message
  if ($null -ne $script:record) {
    $script:record.errors += $_.Exception.Message
    if ($script:record.status -eq 'BLOCKED_PRE_F3') { $script:record.status = 'BLOCKED_PRE_F3' }
    if (-not $script:record.f3.keySent -and $_.Exception.Message -match 'FOREGROUND') { $script:record.focus.status = 'BLOCKED'; $script:record.f3.entryByteStatus = 'BLOCKED' }
  }
} finally {
  try { Stop-OwnedProcess } catch { if ($null -ne $script:record) { $script:record.errors += ('process cleanup: ' + $_.Exception.Message) } }
  try { Save-Evidence } catch { Write-Error ('E_EVIDENCE_WRITE_FAILED: ' + $_.Exception.Message); if ($null -ne $script:evidenceStream) { $script:evidenceStream.Dispose(); $script:evidenceStream = $null } }
  $env:WUXIAN_FORMAL_SAVE_DIR = $script:previousSaveDirectory
  $env:WUXIAN_NATIVE_DIAGNOSTICS_DIR = $script:previousDiagnosticsDirectory
}

if ($null -ne $script:record) {
  Write-Output ('Evidence: ' + $script:evidenceRoot)
  Write-Output ('Status: ' + $script:record.status)
  Write-Output ('F3 entry byte status: ' + $script:record.f3.entryByteStatus)
} else {
  Write-Error ('Identity preflight failed before creating evidence. No process was started and no F3 key was sent. ' + [string]$script:lastFailure)
}
exit $script:exitCode
