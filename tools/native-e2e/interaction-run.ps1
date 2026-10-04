[CmdletBinding()]
param(
  [Parameter(Mandatory = $true)] [string]$ExecutablePath,
  [Parameter(Mandatory = $true)] [string]$ExpectedSha256,
  [Parameter(Mandatory = $true)] [string]$EvidenceDirectory,
  [ValidateSet('JourneyRepeat', 'SaveContinue')] [string]$Scenario = 'JourneyRepeat',
  [ValidateRange(5, 45)] [int]$StartupTimeoutSeconds = 25,
  [ValidateRange(5, 45)] [int]$StepTimeoutSeconds = 20
)

$ErrorActionPreference = 'Stop'
$script:repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$script:evidenceRoot = [IO.Path]::GetFullPath($EvidenceDirectory)
$repoPrefix = [IO.Path]::GetFullPath($script:repoRoot).TrimEnd([IO.Path]::DirectorySeparatorChar) + [IO.Path]::DirectorySeparatorChar
$script:startedUtc = (Get-Date).ToUniversalTime().ToString('o')
$script:exitCode = 2
$script:activeProcess = $null
$script:activeHwnd = [IntPtr]::Zero
$script:activeProcessRecord = $null
$script:ownedProcesses = @()
$script:scriptPath = $PSCommandPath

if ($script:evidenceRoot.StartsWith($repoPrefix, [StringComparison]::OrdinalIgnoreCase) -or
    $script:evidenceRoot.Equals($script:repoRoot, [StringComparison]::OrdinalIgnoreCase)) {
  throw 'E_EVIDENCE_DIRECTORY_INSIDE_REPOSITORY'
}
if (Test-Path -LiteralPath $script:evidenceRoot) {
  if ((Get-ChildItem -LiteralPath $script:evidenceRoot -Force | Measure-Object).Count -gt 0) {
    throw 'E_EVIDENCE_DIRECTORY_NOT_EMPTY'
  }
} else {
  New-Item -ItemType Directory -Path $script:evidenceRoot | Out-Null
}
$script:saveDirectory = Join-Path $script:evidenceRoot 'isolated-save'
$script:diagnosticsDirectory = Join-Path $script:evidenceRoot 'diagnostics'
$script:screenshotDirectory = Join-Path $script:evidenceRoot 'screenshots'
New-Item -ItemType Directory -Force -Path $script:saveDirectory, $script:diagnosticsDirectory, $script:screenshotDirectory | Out-Null
$script:jsonPath = Join-Path $script:evidenceRoot 'interaction.json'
$script:logPath = Join-Path $script:evidenceRoot 'interaction.log'

$script:record = [ordered]@{
  schemaVersion = 1
  status = 'BLOCKED'
  scenario = $Scenario
  startedAtUtc = $script:startedUtc
  completedAtUtc = $null
  repository = [ordered]@{ root = $script:repoRoot; head = $null; branch = $null }
  executable = [ordered]@{ requestedPath = $ExecutablePath; fullPath = $null; expectedSha256 = $ExpectedSha256.ToUpperInvariant(); actualSha256 = $null; sizeBytes = $null; buildIdentity = 'UNSATISFIED; this EXE is from the previously captured 0d315e00 build, not a 34b5aa build.' }
  isolation = [ordered]@{ saveDirectory = $script:saveDirectory; diagnosticsDirectory = $script:diagnosticsDirectory; userSaveDirectoryUsed = $false }
  timeLimitsSeconds = [ordered]@{ startup = $StartupTimeoutSeconds; eachState = $StepTimeoutSeconds; journeyEntry = $StepTimeoutSeconds }
  display = [ordered]@{ width = $null; height = $null; dpiAware = $null }
  processes = @()
  steps = @()
  events = @()
  screenshots = @()
  diagnostics = @()
  errors = @()
}

function Save-Record {
  $script:record.completedAtUtc = (Get-Date).ToUniversalTime().ToString('o')
  $script:record | ConvertTo-Json -Depth 14 | Set-Content -LiteralPath $script:jsonPath -Encoding UTF8
}

function Write-Event([string]$Name, [string]$Status, [string]$Detail, [System.Collections.IDictionary]$Data = @{}) {
  $row = [ordered]@{
    timeUtc = (Get-Date).ToUniversalTime().ToString('o')
    name = $Name
    status = $Status
    detail = $Detail
    pid = if ($null -ne $script:activeProcess) { $script:activeProcess.Id } else { $null }
    hwnd = if ($script:activeHwnd -ne [IntPtr]::Zero) { '0x{0:X}' -f $script:activeHwnd.ToInt64() } else { $null }
    foregroundHwnd = if ('InteractionNative' -as [type]) { '0x{0:X}' -f [InteractionNative]::GetForegroundWindow().ToInt64() } else { $null }
    data = $Data
  }
  $script:record.events += $row
  ($row | ConvertTo-Json -Compress -Depth 8) | Add-Content -LiteralPath $script:logPath -Encoding UTF8
}

function Add-Step([string]$Name, [string]$Status, [string]$Detail, [System.Collections.IDictionary]$Data = @{}) {
  $script:record.steps += [ordered]@{ name = $Name; status = $Status; detail = $Detail; data = $Data; timeUtc = (Get-Date).ToUniversalTime().ToString('o') }
  Write-Event $Name $Status $Detail $Data
  Save-Record
}

function Get-ProcessPath([System.Diagnostics.Process]$Process) {
  try { return [IO.Path]::GetFullPath($Process.MainModule.FileName) } catch { return $null }
}

function Assert-ActiveProcess([switch]$AllowExited) {
  if ($null -eq $script:activeProcess) { throw 'E_NO_TOOL_OWNED_PROCESS' }
  $p = Get-Process -Id $script:activeProcess.Id -ErrorAction SilentlyContinue
  if ($null -eq $p) {
    if ($AllowExited) { return $false }
    throw 'E_TARGET_PROCESS_EXITED'
  }
  $path = Get-ProcessPath $p
  if ($null -eq $path -or -not $path.Equals($script:record.executable.fullPath, [StringComparison]::OrdinalIgnoreCase)) {
    throw 'E_PROCESS_PATH_CHANGED_OR_UNAVAILABLE'
  }
  $hash = (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash
  if ($hash -ne $script:record.executable.actualSha256) { throw 'E_RUNNING_EXECUTABLE_HASH_CHANGED' }
  return $true
}

function Assert-TargetForeground {
  [void](Assert-ActiveProcess)
  if ($script:activeHwnd -eq [IntPtr]::Zero -or -not [InteractionNative]::IsWindow($script:activeHwnd)) { throw 'E_TARGET_WINDOW_UNAVAILABLE' }
  $foreground = [InteractionNative]::GetForegroundWindow()
  if ($foreground -ne $script:activeHwnd) { throw 'E_TARGET_WINDOW_LOST_FOREGROUND' }
  $title = [InteractionNative]::Title($script:activeHwnd)
  if (-not $title.Contains('无限人族进化')) { throw 'E_TARGET_WINDOW_TITLE_CHANGED' }
  return $title
}

function Focus-TargetWindow {
  [void](Assert-ActiveProcess)
  [void][InteractionNative]::ShowWindow($script:activeHwnd, 9)
  for ($attempt = 0; $attempt -lt 5; $attempt++) {
    [void][InteractionNative]::SetForegroundWindow($script:activeHwnd)
    Start-Sleep -Milliseconds 120
    if ([InteractionNative]::GetForegroundWindow() -eq $script:activeHwnd) {
      $title = Assert-TargetForeground
      Write-Event 'foreground-verified' 'PASS' 'Target HWND is the active foreground window before input.' @{ title = $title; attempt = $attempt + 1 }
      return
    }
  }
  try {
    $root = [System.Windows.Automation.AutomationElement]::FromHandle($script:activeHwnd)
    if ($null -ne $root) { $root.SetFocus() }
    Start-Sleep -Milliseconds 150
    if ([InteractionNative]::GetForegroundWindow() -eq $script:activeHwnd) {
      [void](Assert-TargetForeground)
      Write-Event 'foreground-verified' 'PASS' 'UI Automation focused the verified target HWND before input.' @{}
      return
    }
  } catch {
    Write-Event 'foreground-attempt' 'OBSERVED' 'UI Automation focus attempt did not grant foreground.' @{ error = $_.Exception.Message }
  }
  $foreground = [InteractionNative]::GetForegroundWindow()
  $foregroundPid = [uint32]0
  $foregroundThread = [InteractionNative]::GetWindowThreadProcessId($foreground, [ref]$foregroundPid)
  $currentThread = [InteractionNative]::GetCurrentThreadId()
  if ($foregroundThread -gt 0 -and $currentThread -ne $foregroundThread -and
      [InteractionNative]::AttachThreadInput($currentThread, $foregroundThread, $true)) {
    try { [void][InteractionNative]::SetForegroundWindow($script:activeHwnd) }
    finally { [void][InteractionNative]::AttachThreadInput($currentThread, $foregroundThread, $false) }
    Start-Sleep -Milliseconds 150
    if ([InteractionNative]::GetForegroundWindow() -eq $script:activeHwnd) {
      [void](Assert-TargetForeground)
      Write-Event 'foreground-verified' 'PASS' 'Attached to the existing foreground thread for the target focus request, then detached before input.' @{ previousForegroundHwnd = ('0x{0:X}' -f $foreground.ToInt64()); previousForegroundPid = $foregroundPid }
      return
    }
    Write-Event 'foreground-attempt' 'OBSERVED' 'Foreground-thread attachment was released without foreground focus being granted.' @{ previousForegroundHwnd = ('0x{0:X}' -f $foreground.ToInt64()); previousForegroundPid = $foregroundPid }
  }
  throw 'E_TARGET_WINDOW_FOCUS_DENIED'
}

function Get-AccessibleSnapshot {
  [void](Assert-TargetForeground)
  $root = [System.Windows.Automation.AutomationElement]::FromHandle($script:activeHwnd)
  if ($null -eq $root) { throw 'E_UIA_ROOT_UNAVAILABLE' }
  $all = $root.FindAll([System.Windows.Automation.TreeScope]::Descendants, [System.Windows.Automation.Condition]::TrueCondition)
  $items = [System.Collections.Generic.List[object]]::new()
  $count = [Math]::Min($all.Count, 1200)
  for ($i = 0; $i -lt $count; $i++) {
    try {
      $el = $all.Item($i)
      $c = $el.Current
      $name = [string]$c.Name
      $offscreen = [bool]$c.IsOffscreen
      $rect = $c.BoundingRectangle
      $items.Add([pscustomobject]@{
        Element = $el
        Name = $name
        AutomationId = [string]$c.AutomationId
        ControlTypeId = [int]$c.ControlType.Id
        ControlType = [string]$c.ControlType.ProgrammaticName
        IsEnabled = [bool]$c.IsEnabled
        IsOffscreen = $offscreen
        Bounds = [ordered]@{ left = [double]$rect.Left; top = [double]$rect.Top; width = [double]$rect.Width; height = [double]$rect.Height }
      })
    } catch {
      continue
    }
  }
  return [pscustomobject]@{ RootName = [string]$root.Current.Name; Elements = @($items); Names = @($items | Where-Object { -not $_.IsOffscreen -and -not [string]::IsNullOrWhiteSpace($_.Name) } | Select-Object -ExpandProperty Name -Unique) }
}

function Save-UiDump([string]$Name, $Snapshot) {
  $path = Join-Path $script:evidenceRoot ($Name + '-uia-tree.json')
  $rows = @($Snapshot.Elements | ForEach-Object { [ordered]@{ name = $_.Name; automationId = $_.AutomationId; controlType = $_.ControlType; isEnabled = $_.IsEnabled; isOffscreen = $_.IsOffscreen; bounds = $_.Bounds } })
  [ordered]@{ root = $Snapshot.RootName; visibleNames = $Snapshot.Names; elements = $rows } | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $path -Encoding UTF8
  return $path
}

function Capture-Stage([string]$Name) {
  $title = Assert-TargetForeground
  $bounds = [InteractionNative]::Bounds($script:activeHwnd)
  if ($bounds.Count -ne 4 -or $bounds[2] -le 0 -or $bounds[3] -le 0) { throw 'E_TARGET_WINDOW_BOUNDS_UNAVAILABLE' }
  $path = Join-Path $script:screenshotDirectory ($Name + '.png')
  [InteractionNative]::CaptureVisibleWindow($path, $bounds[0], $bounds[1], $bounds[2], $bounds[3])
  $item = Get-Item -LiteralPath $path
  if ($item.Length -le 0) { throw 'E_EMPTY_SCREENSHOT' }
  $sha = (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash
  $script:record.screenshots += [ordered]@{ name = $Name; path = $path; sizeBytes = $item.Length; sha256 = $sha; bounds = @($bounds); title = $title; pid = $script:activeProcess.Id; source = 'CopyFromScreen visible native desktop' }
  Write-Event 'screenshot' 'PASS' 'Captured the visible target Tauri window after foreground verification.' @{ name = $Name; path = $path; sha256 = $sha; sizeBytes = $item.Length; bounds = @($bounds) }
  Save-Record
  return $path
}

function Get-ButtonMatches($Snapshot, [string[]]$Labels) {
  $buttons = @($Snapshot.Elements | Where-Object { $_.ControlTypeId -eq 50000 -and -not $_.IsOffscreen })
  $matches = @()
  foreach ($label in $Labels) {
    $exact = @($buttons | Where-Object { $_.Name.Trim() -eq $label })
    if ($exact.Count -eq 1) { return $exact }
    if ($exact.Count -gt 1) { return $exact }
    $partial = @($buttons | Where-Object { $_.Name.Contains($label) })
    if ($partial.Count -eq 1) { return $partial }
    if ($partial.Count -gt 1) { return $partial }
  }
  return @()
}

function Find-Button([string[]]$Labels, [string]$Stage, [switch]$RequireEnabled) {
  $snapshot = Get-AccessibleSnapshot
  $matches = @(Get-ButtonMatches $snapshot $Labels)
  if ($matches.Count -ne 1) {
    $treePath = Save-UiDump $Stage $snapshot
    if ($matches.Count -eq 0) { throw "E_UIA_BUTTON_NOT_FOUND:$($Labels -join '|'):$treePath" }
    throw "E_UIA_BUTTON_AMBIGUOUS:$($Labels -join '|'):$($matches.Count):$treePath"
  }
  $match = $matches[0]
  if ($RequireEnabled -and -not $match.IsEnabled) { throw "E_UIA_BUTTON_DISABLED:$($match.Name)" }
  if ($match.Bounds.width -le 1 -or $match.Bounds.height -le 1) { throw "E_UIA_BUTTON_BOUNDS_INVALID:$($match.Name)" }
  return [pscustomobject]@{ Snapshot = $snapshot; Item = $match }
}

function Invoke-UiButton([string[]]$Labels, [string]$Stage, [switch]$AllowExit) {
  [void](Assert-TargetForeground)
  $found = Find-Button $Labels $Stage -RequireEnabled
  $item = $found.Item
  $patternObject = $null
  if (-not $item.Element.TryGetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern, [ref]$patternObject)) {
    throw "E_UIA_INVOKE_PATTERN_UNAVAILABLE:$($item.Name)"
  }
  $client = [InteractionNative]::ClientBoundsOnScreen($script:activeHwnd)
  $cx = $item.Bounds.left + ($item.Bounds.width / 2)
  $cy = $item.Bounds.top + ($item.Bounds.height / 2)
  if ($client.Count -ne 4 -or $cx -lt $client[0] -or $cy -lt $client[1] -or $cx -gt ($client[0] + $client[2]) -or $cy -gt ($client[1] + $client[3])) {
    throw "E_UIA_CONTROL_OUTSIDE_TARGET_CLIENT:$($item.Name)"
  }
  $title = Assert-TargetForeground
  if (-not $item.IsEnabled -or $item.IsOffscreen) { throw "E_UIA_CONTROL_STATE_CHANGED:$($item.Name)" }
  $before = [ordered]@{ name = $item.Name; automationId = $item.AutomationId; bounds = $item.Bounds; title = $title; pid = $script:activeProcess.Id; hwnd = '0x{0:X}' -f $script:activeHwnd.ToInt64(); foreground = '0x{0:X}' -f [InteractionNative]::GetForegroundWindow().ToInt64() }
  Write-Event $Stage 'INPUT_BEGIN' 'UI Automation InvokePattern targets the unique enabled button discovered in the focused native HWND.' $before
  [void]$item.Element.SetFocus()
  if ([InteractionNative]::GetForegroundWindow() -ne $script:activeHwnd) { throw 'E_TARGET_WINDOW_LOST_FOREGROUND_BEFORE_INVOKE' }
  $patternObject.Invoke()
  Start-Sleep -Milliseconds 150
  if (-not $AllowExit -or $null -ne (Get-Process -Id $script:activeProcess.Id -ErrorAction SilentlyContinue)) {
    [void](Assert-TargetForeground)
  }
  Write-Event $Stage 'INPUT_SENT' 'InvokePattern completed; post-input process path/hash and foreground HWND rechecked.' @{ name = $item.Name; automationId = $item.AutomationId; pid = $script:activeProcess.Id; foregroundHwnd = '0x{0:X}' -f [InteractionNative]::GetForegroundWindow().ToInt64() }
  Add-Step $Stage 'INPUT_SENT' "UIA invoked '$($item.Name)' in the verified native window." @{ button = $item.Name; automationId = $item.AutomationId; bounds = $item.Bounds; pid = $script:activeProcess.Id; hwnd = '0x{0:X}' -f $script:activeHwnd.ToInt64() }
}

function Get-ReturnStationResult($Snapshot) {
  $n = $Snapshot.Names -join ' | '
  $tickMatch = [regex]::Match($n, '世界状态已连接\s*·\s*第\s*(\d+)\s*帧')
  return (($n -match 'rs_core_room') -and $tickMatch.Success -and [int]$tickMatch.Groups[1].Value -ge 30)
}

function Get-HomeResult($Snapshot) {
  $n = $Snapshot.Names -join ' | '
  $buttons = Get-ButtonMatches $Snapshot @('新旅程')
  $exits = Get-ButtonMatches $Snapshot @('退出')
  return ($buttons.Count -eq 1 -and $exits.Count -eq 1 -and $n -match '归航站|RETURN STATION')
}

function Wait-State([string]$State, [int]$TimeoutSeconds, [string]$Stage) {
  $deadline = (Get-Date).AddSeconds($TimeoutSeconds)
  $lastSignature = ''
  $last = $null
  while ((Get-Date) -lt $deadline) {
    [void](Assert-TargetForeground)
    $last = Get-AccessibleSnapshot
    $signature = ($last.Names -join ' | ')
    if ($signature -ne $lastSignature) {
      $lastSignature = $signature
      Write-Event ($Stage + '-visible-state') 'OBSERVED' 'Current accessible names from the live Tauri window.' @{ names = @($last.Names) }
    }
    $ok = $false
    switch ($State) {
      'ReturnStation' { $ok = Get-ReturnStationResult $last }
      'Home' { $ok = Get-HomeResult $last }
      'Paused' { $ok = (($last.Names -join ' | ') -match '旅程已暂停') }
      'Unpaused' { $ok = (($last.Names -join ' | ') -notmatch '旅程已暂停') -and (Get-ReturnStationResult $last) }
      'Saved' { $ok = (($last.Names -join ' | ') -match '已保存.*现场记录') }
      'ContinuePicker' {
        $hasButton = (Get-ButtonMatches $last @('继续所选存档')).Count -eq 1
        $hasCombo = @($last.Elements | Where-Object { $_.ControlTypeId -eq 50003 -and -not $_.IsOffscreen }).Count -ge 1
        $ok = $hasButton -and $hasCombo
      }
      'ContinueStatus' { $ok = (($last.Names -join ' | ') -match '将从「现场记录」恢复') }
    }
    if ($ok) {
      Add-Step $Stage 'PASS' "Observed required live-window state '$State'." @{ names = @($last.Names) }
      return [pscustomobject]@{ Passed = $true; Snapshot = $last }
    }
    Start-Sleep -Milliseconds 400
  }
  if ($null -ne $last) {
    $treePath = Save-UiDump $Stage $last
    Add-Step $Stage 'TIMEOUT' "Did not observe '$State' within ${TimeoutSeconds}s; last accessible names retained." @{ names = @($last.Names); uiTree = $treePath }
  } else {
    Add-Step $Stage 'TIMEOUT' "No accessible snapshot for '$State' within ${TimeoutSeconds}s." @{}
  }
  return [pscustomobject]@{ Passed = $false; Snapshot = $last }
}

function Start-TestProcess([string]$Label) {
  $sameName = @(Get-Process -Name ([IO.Path]::GetFileNameWithoutExtension($script:record.executable.fullPath)) -ErrorAction SilentlyContinue)
  if ($sameName.Count -gt 0) { throw 'E_EXISTING_GAME_PROCESS_PRESENT' }
  $process = Start-Process -FilePath $script:record.executable.fullPath -WorkingDirectory (Split-Path -Parent $script:record.executable.fullPath) -PassThru -WindowStyle Normal
  $script:activeProcess = $process
  $script:ownedProcesses += $process
  $processRecord = [ordered]@{ label = $Label; pid = $process.Id; startedByTool = $true; executablePath = $script:record.executable.fullPath; executableSha256 = $script:record.executable.actualSha256; startedAtUtc = (Get-Date).ToUniversalTime().ToString('o'); mainWindowHandle = $null; title = $null; responding = $null; stoppedAtUtc = $null; stopMethod = $null }
  $script:record.processes += $processRecord
  $script:activeProcessRecord = $processRecord
  $deadline = (Get-Date).AddSeconds($StartupTimeoutSeconds)
  $handle = [IntPtr]::Zero
  while ((Get-Date) -lt $deadline) {
    try { $process.Refresh(); $handle = $process.MainWindowHandle } catch {}
    if ($handle -ne [IntPtr]::Zero) { break }
    Start-Sleep -Milliseconds 200
  }
  if ($handle -eq [IntPtr]::Zero) { throw 'E_MAIN_WINDOW_TIMEOUT' }
  $script:activeHwnd = $handle
  $path = Get-ProcessPath $process
  if ($null -eq $path -or -not $path.Equals($script:record.executable.fullPath, [StringComparison]::OrdinalIgnoreCase)) { throw 'E_LAUNCHED_PROCESS_PATH_MISMATCH' }
  $sha = (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash
  if ($sha -ne $script:record.executable.actualSha256) { throw 'E_LAUNCHED_PROCESS_SHA_MISMATCH' }
  $title = [InteractionNative]::Title($handle)
  if (-not $title.Contains('无限人族进化')) { throw 'E_WINDOW_TITLE_MISMATCH' }
  $processRecord.mainWindowHandle = '0x{0:X}' -f $handle.ToInt64()
  $processRecord.title = $title
  $processRecord.responding = $process.Responding
  Add-Step ($Label + '-launch') 'PASS' 'Started only this verified EXE; PID, module path, SHA, HWND and title match.' @{ pid = $process.Id; executablePath = $path; sha256 = $sha; hwnd = $processRecord.mainWindowHandle; title = $title; responding = $process.Responding }
  Focus-TargetWindow
  return $process
}

function Stop-OwnedProcess([string]$Method) {
  if ($null -eq $script:activeProcess -or $null -eq $script:activeProcessRecord) { return }
  $p = Get-Process -Id $script:activeProcess.Id -ErrorAction SilentlyContinue
  if ($null -eq $p) {
    $script:activeProcessRecord.stoppedAtUtc = (Get-Date).ToUniversalTime().ToString('o')
    $script:activeProcessRecord.stopMethod = 'exited-by-application'
    Write-Event 'process-exit' 'PASS' 'Tool-owned game process exited.' @{ pid = $script:activeProcess.Id }
  } else {
    $path = Get-ProcessPath $p
    if ($null -eq $path -or -not $path.Equals($script:record.executable.fullPath, [StringComparison]::OrdinalIgnoreCase)) {
      Write-Event 'process-stop' 'BLOCKED' 'PID path no longer matches; did not stop it.' @{ pid = $script:activeProcess.Id; path = $path }
      return
    }
    Stop-Process -Id $script:activeProcess.Id -Force
    [void]$p.WaitForExit(5000)
    $script:activeProcessRecord.stoppedAtUtc = (Get-Date).ToUniversalTime().ToString('o')
    $script:activeProcessRecord.stopMethod = $Method
    Write-Event 'process-stop' 'PASS' 'Stopped only the tool-owned PID after exact executable path recheck.' @{ pid = $script:activeProcess.Id; method = $Method }
  }
  $script:activeProcess = $null
  $script:activeHwnd = [IntPtr]::Zero
  $script:activeProcessRecord = $null
}

function Get-SaveFiles {
  return @(Get-ChildItem -LiteralPath $script:saveDirectory -File -Recurse -ErrorAction SilentlyContinue | ForEach-Object { [ordered]@{ path = $_.FullName; bytes = $_.Length; sha256 = (Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash } })
}

function Invoke-JourneyRepeatScenario {
  [void](Start-TestProcess 'journey-repeat')
  Capture-Stage '00-initial-menu'
  $initial = Get-AccessibleSnapshot
  $tree = Save-UiDump '00-initial-menu' $initial
  Add-Step 'initial-menu-controls' 'OBSERVED' 'Initial native menu accessibility tree captured before input.' @{ uiTree = $tree; names = @($initial.Names) }
  Invoke-UiButton @('新旅程', '新的旅程') 'new-journey-first'
  $first = Wait-State 'ReturnStation' $StepTimeoutSeconds 'first-return-station'
  Capture-Stage '01-first-new-journey-result'
  if (-not $first.Passed) {
    $script:record.status = 'FAIL'
    $script:record.errors += 'First New Journey did not reach the visible Return Station state within the time limit.'
    return
  }
  Invoke-UiButton @('返回主界面') 'return-to-home'
  $homeResult = Wait-State 'Home' $StepTimeoutSeconds 'returned-home'
  Capture-Stage '02-returned-home'
  if (-not $homeResult.Passed) {
    $script:record.status = 'BLOCKED'
    $script:record.errors += 'Could not observe the main menu after invoking Return to Home.'
    return
  }
  Invoke-UiButton @('新旅程', '新的旅程') 'new-journey-second'
  $second = Wait-State 'ReturnStation' $StepTimeoutSeconds 'second-return-station'
  Capture-Stage '03-second-new-journey-result'
  if ($second.Passed) {
    Add-Step 'second-new-journey' 'PASS_NOT_REPRODUCED' 'The second New Journey reached the visible Return Station state; the reported stall did not reproduce in this run.' @{ observation = @($second.Snapshot.Names) }
    $script:record.status = 'PASS'
  } else {
    $stateNames = if ($null -ne $second.Snapshot) { $second.Snapshot.Names -join ' | ' } else { '' }
    if ($stateNames -match '正在建立世界链路|世界操作超时|链路响应失败') {
      Add-Step 'second-new-journey' 'FAIL_REPRODUCED' 'The second New Journey remained in a visible pending/error state until the bounded timeout.' @{ observation = @($second.Snapshot.Names) }
      $script:record.status = 'FAIL'
      $script:record.errors += 'Second New Journey stall was reproduced; preserve screenshots and diagnostics for investigation.'
    } else {
      Add-Step 'second-new-journey' 'BLOCKED' 'Timed out without enough accessible UI state to classify the second New Journey result.' @{ observation = if ($null -ne $second.Snapshot) { @($second.Snapshot.Names) } else { @() } }
      $script:record.status = 'BLOCKED'
    }
  }
}

function Invoke-KeyboardKey([byte]$VirtualKey, [string]$Name) {
  [void](Assert-TargetForeground)
  Write-Event ('key-' + $Name) 'INPUT_BEGIN' 'Verified foreground before real keyboard input.' @{ virtualKey = $VirtualKey }
  [InteractionNative]::KeyDown($VirtualKey)
  [InteractionNative]::KeyUp($VirtualKey)
  Start-Sleep -Milliseconds 120
  [void](Assert-TargetForeground)
  Write-Event ('key-' + $Name) 'INPUT_SENT' 'Verified PID/path/hash and foreground HWND after real keyboard input.' @{ virtualKey = $VirtualKey }
}

function Invoke-SaveContinueScenario {
  [void](Start-TestProcess 'save-create')
  Capture-Stage '00-initial-menu-save-flow'
  Invoke-UiButton @('新旅程', '新的旅程') 'save-flow-new-journey'
  $entered = Wait-State 'ReturnStation' $StepTimeoutSeconds 'save-flow-return-station'
  Capture-Stage '01-save-flow-return-station'
  if (-not $entered.Passed) { $script:record.status = 'FAIL'; $script:record.errors += 'Could not enter the visible Return Station state to create an isolated save.'; return }
  Invoke-UiButton @('暂停') 'pause-before-save'
  $paused = Wait-State 'Paused' $StepTimeoutSeconds 'pause-confirmed'
  Capture-Stage '02-paused-save-panel'
  if (-not $paused.Passed) { $script:record.status = 'BLOCKED'; $script:record.errors += 'Could not observe the authoritative paused state.'; return }
  $saveButton = $null
  $deadline = (Get-Date).AddSeconds($StepTimeoutSeconds)
  while ((Get-Date) -lt $deadline) {
    $saveSnapshot = Get-AccessibleSnapshot
    $saveMatches = @(Get-ButtonMatches $saveSnapshot @('另存为新存档'))
    if ($saveMatches.Count -gt 1) { throw "E_SAVE_BUTTON_AMBIGUOUS:$($saveMatches.Count)" }
    $saveButton = if ($saveMatches.Count -eq 1) { $saveMatches[0] } else { $null }
    if ($null -ne $saveButton -and $saveButton.IsEnabled) { break }
    Start-Sleep -Milliseconds 400
  }
  if ($null -eq $saveButton -or -not $saveButton.IsEnabled) { throw 'E_SAVE_BUTTON_NOT_ENABLED' }
  Invoke-UiButton @('另存为新存档') 'save-isolated-slot'
  $saved = Wait-State 'Saved' $StepTimeoutSeconds 'save-confirmed'
  Capture-Stage '03-save-confirmed'
  $saveFiles = Get-SaveFiles
  Add-Step 'isolated-save-files' $(if ($saveFiles.Count -gt 0) { 'PASS' } else { 'BLOCKED' }) 'Enumerated only files under the isolated save directory.' @{ directory = $script:saveDirectory; files = $saveFiles }
  if (-not $saved.Passed -or $saveFiles.Count -eq 0) { $script:record.status = 'BLOCKED'; $script:record.errors += 'UI or filesystem evidence did not confirm a persisted isolated save.'; return }
  Invoke-UiButton @('继续') 'resume-after-save'
  $unpaused = Wait-State 'Unpaused' $StepTimeoutSeconds 'unpaused-after-save'
  if (-not $unpaused.Passed) { $script:record.status = 'BLOCKED'; $script:record.errors += 'Could not confirm the pause overlay closed after saving.'; return }
  Invoke-UiButton @('返回主界面') 'return-home-after-save'
  $homeResult = Wait-State 'Home' $StepTimeoutSeconds 'home-before-exit'
  Capture-Stage '04-home-before-exit'
  if (-not $homeResult.Passed) { $script:record.status = 'BLOCKED'; $script:record.errors += 'Could not return to the main menu after saving.'; return }
  Invoke-UiButton @('退出') 'quit-after-save' -AllowExit
  $exitDeadline = (Get-Date).AddSeconds(6)
  while ((Get-Date) -lt $exitDeadline -and $null -ne (Get-Process -Id $script:activeProcess.Id -ErrorAction SilentlyContinue)) { Start-Sleep -Milliseconds 200 }
  Stop-OwnedProcess 'fallback-stop-after-quit-button'
  [void](Start-TestProcess 'continue-relaunch')
  Capture-Stage '05-relaunched-with-isolated-save'
  $home2 = Wait-State 'Home' $StepTimeoutSeconds 'home-after-relaunch'
  if (-not $home2.Passed) { $script:record.status = 'BLOCKED'; $script:record.errors += 'Could not observe main menu after relaunch.'; return }
  $continue = Find-Button @('继续') 'continue-button' -RequireEnabled
  Invoke-UiButton @('继续') 'open-continue-picker'
  $picker = Wait-State 'ContinuePicker' $StepTimeoutSeconds 'continue-picker-open'
  Capture-Stage '06-continue-picker'
  if (-not $picker.Passed) { $script:record.status = 'BLOCKED'; $script:record.errors += 'Could not reliably inspect the expanded native Continue picker.'; return }
  $statusNow = $picker.Snapshot.Names -join ' | '
  if ($statusNow -notmatch '将从「现场记录」恢复') {
    $combo = @($picker.Snapshot.Elements | Where-Object { $_.ControlTypeId -eq 50003 -and -not $_.IsOffscreen })
    if ($combo.Count -ne 1) { throw "E_CONTINUE_COMBO_AMBIGUOUS:$($combo.Count)" }
    [void](Assert-TargetForeground)
    $combo[0].Element.SetFocus()
    [void](Assert-TargetForeground)
    Invoke-KeyboardKey 0x24 'home-select-first-save'
    Invoke-KeyboardKey 0x0D 'confirm-save-selection'
  }
  $slotStatus = Wait-State 'ContinueStatus' $StepTimeoutSeconds 'continue-slot-selected'
  Capture-Stage '07-selected-save'
  if (-not $slotStatus.Passed) { $script:record.status = 'BLOCKED'; $script:record.errors += 'The isolated save could not be reliably selected in the native picker.'; return }
  Invoke-UiButton @('继续所选存档') 'continue-selected-save'
  $continued = Wait-State 'ReturnStation' $StepTimeoutSeconds 'continued-return-station'
  Capture-Stage '08-continue-result'
  if ($continued.Passed) {
    Add-Step 'save-exit-relaunch-continue' 'PASS' 'A save file was observed in the isolated directory; after application quit and relaunch, the selected save reached the visible Return Station state.' @{ names = @($continued.Snapshot.Names); files = (Get-SaveFiles) }
    $script:record.status = 'PASS'
  } else {
    Add-Step 'save-exit-relaunch-continue' 'FAIL' 'The selected isolated save did not reach the visible Return Station state within the time limit.' @{ names = if ($null -ne $continued.Snapshot) { @($continued.Snapshot.Names) } else { @() } }
    $script:record.status = 'FAIL'
  }
}

try {
  $script:record.repository.head = (& git -C $script:repoRoot rev-parse HEAD 2>$null | Out-String).Trim()
  $script:record.repository.branch = (& git -C $script:repoRoot branch --show-current 2>$null | Out-String).Trim()
  if (-not (Test-Path -LiteralPath $ExecutablePath -PathType Leaf)) {
    $script:record.executable.fullPath = [IO.Path]::GetFullPath($ExecutablePath)
    $script:record.errors += 'E_EXECUTABLE_NOT_FOUND'
    $script:record.status = 'BLOCKED'
    $script:exitCode = 10
    throw 'E_EXECUTABLE_NOT_FOUND'
  }
  $script:record.executable.fullPath = (Resolve-Path -LiteralPath $ExecutablePath).Path
  $file = Get-Item -LiteralPath $script:record.executable.fullPath
  $script:record.executable.sizeBytes = $file.Length
  $script:record.executable.actualSha256 = (Get-FileHash -LiteralPath $file.FullName -Algorithm SHA256).Hash
  if ($script:record.executable.actualSha256 -ne $script:record.executable.expectedSha256) {
    $script:record.errors += 'E_EXECUTABLE_SHA256_MISMATCH'
    $script:record.status = 'BLOCKED'
    $script:exitCode = 10
    throw 'E_EXECUTABLE_SHA256_MISMATCH'
  }

  $nativeCode = @"
using System;
using System.Runtime.InteropServices;
using System.Text;
using System.Drawing;
public static class InteractionNative {
  [StructLayout(LayoutKind.Sequential)] public struct RECT { public int Left; public int Top; public int Right; public int Bottom; }
  [StructLayout(LayoutKind.Sequential)] public struct POINT { public int X; public int Y; }
  [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
  [DllImport("user32.dll")] public static extern int GetSystemMetrics(int index);
  [DllImport("user32.dll")] public static extern bool IsWindow(IntPtr hwnd);
  [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr hwnd, int command);
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr hwnd);
  [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr hwnd, out uint processId);
  [DllImport("kernel32.dll")] public static extern uint GetCurrentThreadId();
  [DllImport("user32.dll")] public static extern bool AttachThreadInput(uint attachThread, uint attachToThread, bool attach);
  [DllImport("user32.dll", SetLastError=true)] public static extern bool GetWindowRect(IntPtr hwnd, out RECT rect);
  [DllImport("user32.dll", SetLastError=true)] public static extern bool GetClientRect(IntPtr hwnd, out RECT rect);
  [DllImport("user32.dll", SetLastError=true)] public static extern bool ClientToScreen(IntPtr hwnd, ref POINT point);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern int GetWindowText(IntPtr hwnd, StringBuilder text, int maxCount);
  [DllImport("user32.dll")] public static extern void keybd_event(byte key, byte scan, uint flags, UIntPtr extraInfo);
  public static string Title(IntPtr hwnd) { var b = new StringBuilder(1024); GetWindowText(hwnd,b,b.Capacity); return b.ToString(); }
  public static int[] Bounds(IntPtr hwnd) { RECT r; if(!GetWindowRect(hwnd,out r)) return new int[0]; return new int[]{r.Left,r.Top,r.Right-r.Left,r.Bottom-r.Top}; }
  public static int[] ClientBoundsOnScreen(IntPtr hwnd) { RECT r; if(!GetClientRect(hwnd,out r)) return new int[0]; POINT p=new POINT{X=r.Left,Y=r.Top}; if(!ClientToScreen(hwnd,ref p)) return new int[0]; return new int[]{p.X,p.Y,r.Right-r.Left,r.Bottom-r.Top}; }
  public static void KeyDown(byte key) { keybd_event(key,0,0,UIntPtr.Zero); }
  public static void KeyUp(byte key) { keybd_event(key,0,2,UIntPtr.Zero); }
  public static void CaptureVisibleWindow(string path,int x,int y,int width,int height) { using(var image=new Bitmap(width,height)) { using(var g=Graphics.FromImage(image)) { g.CopyFromScreen(x,y,0,0,new Size(width,height),CopyPixelOperation.SourceCopy); } image.Save(path,System.Drawing.Imaging.ImageFormat.Png); } }
}
"@
  $drawingAssemblies = @('System.Drawing.dll','System.Drawing.Common.dll','System.Drawing.Primitives.dll','System.Private.Windows.GdiPlus.dll','System.Private.Windows.Core.dll') | ForEach-Object { Join-Path ([System.AppContext]::BaseDirectory) $_ }
  Add-Type -TypeDefinition $nativeCode -ReferencedAssemblies $drawingAssemblies
  $script:record.display.dpiAware = [InteractionNative]::SetProcessDPIAware()
  $script:record.display.width = [InteractionNative]::GetSystemMetrics(0)
  $script:record.display.height = [InteractionNative]::GetSystemMetrics(1)
  Add-Type -AssemblyName UIAutomationClient
  Add-Type -AssemblyName UIAutomationTypes
  $env:WUXIAN_FORMAL_SAVE_DIR = $script:saveDirectory
  $env:WUXIAN_NATIVE_DIAGNOSTICS_DIR = $script:diagnosticsDirectory
  Write-Event 'preflight' 'PASS' 'Executable SHA and clean-window process preconditions passed; isolated save and diagnostics directories are set.' @{ head = $script:record.repository.head; branch = $script:record.repository.branch; executableSha256 = $script:record.executable.actualSha256; saveDir = $script:saveDirectory; diagnosticsDir = $script:diagnosticsDirectory; display = @($script:record.display.width,$script:record.display.height) }
  if ($Scenario -eq 'JourneyRepeat') { Invoke-JourneyRepeatScenario } else { Invoke-SaveContinueScenario }
  if ($script:record.status -eq 'PASS') { $script:exitCode = 0 }
  elseif ($script:record.status -eq 'FAIL') { $script:exitCode = 1 }
  else { $script:exitCode = 2 }
} catch {
  $message = $_.Exception.Message
  if ($script:record.errors.Count -eq 0 -or $script:record.errors[-1] -ne $message) { $script:record.errors += $message }
  if ($script:exitCode -ne 10) {
    $script:record.status = 'BLOCKED'
    $script:exitCode = 2
  }
  Write-Event 'exception' $script:record.status $message @{}
} finally {
  try { Stop-OwnedProcess 'script-finally-cleanup' } catch { $script:record.errors += ('cleanup: ' + $_.Exception.Message) }
  try {
    $script:record.diagnostics = @(Get-ChildItem -LiteralPath $script:diagnosticsDirectory -File -Recurse -ErrorAction SilentlyContinue | ForEach-Object { [ordered]@{ path = $_.FullName; bytes = $_.Length; sha256 = (Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash } })
  } catch { $script:record.errors += ('diagnostics enumeration: ' + $_.Exception.Message) }
  Save-Record
}

Write-Output ('Evidence: ' + $script:evidenceRoot)
Write-Output ('Scenario: ' + $Scenario)
Write-Output ('Status: ' + $script:record.status)
exit $script:exitCode
