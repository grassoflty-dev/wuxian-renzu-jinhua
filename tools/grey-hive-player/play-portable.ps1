param([string]$ProfileRoot, [string]$DiagnosticFirstLaunchUtc, [string]$DiagnosticHardDeadlineUtc,
    [ValidateSet('Play','RequestStop')][string]$Operation = 'Play', [string]$ProfileId, [string]$AttemptId,
    [int]$ControllerPid, [string]$ControllerBirthUtc)
$ErrorActionPreference = 'Stop'
$phase = 'host'
try {
    if (-not $IsWindows -or $PSVersionTable.PSEdition -cne 'Core' -or $PSVersionTable.PSVersion -lt [version]'7.4') { throw 'GH_PORTABLE_HOST' }
    $phase = 'layout'
    # Validate the adjacent path definitions before loading their functions.
    $definitions = [IO.FileInfo]::new([IO.Path]::Combine($PSScriptRoot, 'paths.ps1'))
    if (-not $definitions.Exists -or ($definitions.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) { throw 'GH_PORTABLE_LAYOUT' }
    for ($ancestor = $definitions.Directory; $null -ne $ancestor; $ancestor = $ancestor.Parent) {
        if (-not $ancestor.Exists -or ($ancestor.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) { throw 'GH_PORTABLE_LAYOUT' }
    }
    . ($definitions.FullName)
    $launcherDir = Resolve-GhOrdinaryPath $PSScriptRoot Directory
    $packageRoot = Resolve-GhOrdinaryPath ([IO.Path]::GetDirectoryName($launcherDir)) Directory
    $pwsh = Resolve-GhOrdinaryPath ($packageRoot + '/runtime/pwsh/pwsh.exe') File
    $native = Resolve-GhOrdinaryPath ($packageRoot + '/native/wuxian_v1_authority.exe') File
    $godot = Resolve-GhOrdinaryPath ($packageRoot + '/runtime/godot/Godot.exe') File
    $project = Resolve-GhOrdinaryPath ($packageRoot + '/project') Directory
    $null = Resolve-GhOrdinaryPath ($project + '/project.godot') File
    $launcher = Resolve-GhOrdinaryPath ($launcherDir + '/launch.ps1') File
    $metadataPath = Resolve-GhOrdinaryPath ($packageRoot + '/package-runtime.json') File
    $phase = 'host'
    $hostPath = Resolve-GhOrdinaryPath ([Environment]::ProcessPath) File
    if (-not [string]::Equals($hostPath, $pwsh, [StringComparison]::OrdinalIgnoreCase)) { throw 'GH_PORTABLE_USE_PACKAGED_PWSH' }
    $phase = 'metadata'
    $stream = [IO.File]::Open($metadataPath, [IO.FileMode]::Open, [IO.FileAccess]::Read, [IO.FileShare]::Read)
    try { $metadata = Read-GhJsonObject $stream 4096 @('NativeSha256','GodotSha256') } finally { $stream.Dispose() }
    $hashes = @{}
    foreach ($key in @('NativeSha256','GodotSha256')) {
        if ($metadata[$key].ValueKind -ne [Text.Json.JsonValueKind]::String) { throw 'GH_PORTABLE_HASH' }
        $value = $metadata[$key].GetString()
        if ($value -cnotmatch '^[0-9a-fA-F]{64}$') { throw 'GH_PORTABLE_HASH' }
        $hashes[$key] = $value
    }
    $phase = 'diagnostic'
    . (Resolve-GhOrdinaryPath ($launcherDir + '/lifetime.ps1') File)
    if ($Operation -eq 'RequestStop') {
        if ($PSBoundParameters.ContainsKey('DiagnosticFirstLaunchUtc') -or $PSBoundParameters.ContainsKey('DiagnosticHardDeadlineUtc') -or -not $PSBoundParameters.ContainsKey('ProfileRoot')) { throw 'GH_STOP_ARGUMENT' }
        $phase = 'stop'
        $published = Send-GhStopRequest -ProfileRoot $ProfileRoot -ProfileId $ProfileId -AttemptId $AttemptId -ControllerPid $ControllerPid -ControllerBirthUtc $ControllerBirthUtc
        [Console]::WriteLine($published)
        exit 0
    }
    foreach ($key in @('ProfileId','AttemptId','ControllerPid','ControllerBirthUtc')) { if ($PSBoundParameters.ContainsKey($key)) { throw 'GH_STOP_ARGUMENT' } }
    $diagnosticArguments = Resolve-GhDiagnosticArguments $PSBoundParameters ([Environment]::GetEnvironmentVariable('WUXIAN_GH_DIAGNOSTIC_FIRST_UTC')) ([Environment]::GetEnvironmentVariable('WUXIAN_GH_DIAGNOSTIC_HARD_UTC'))
    $phase = 'profile'
    if (-not $PSBoundParameters.ContainsKey('ProfileRoot')) {
        $local = Resolve-GhOrdinaryPath $env:LOCALAPPDATA Directory
        $ProfileRoot = $local.TrimEnd('/') + '/WuxianGreyHivePlayer'
    }
    # Only a missing leaf is admitted; overrides never create unknown parents.
    if ([string]::IsNullOrWhiteSpace($ProfileRoot)) { throw 'GH_PORTABLE_PROFILE' }
    $profile = Resolve-GhOrdinaryPath $ProfileRoot Directory $true
    $rootPrefix = $packageRoot.TrimEnd('/') + '/'
    $profilePrefix = $profile.TrimEnd('/') + '/'
    if ([string]::Equals($profile, $packageRoot, [StringComparison]::OrdinalIgnoreCase) -or
        $profile.StartsWith($rootPrefix, [StringComparison]::OrdinalIgnoreCase) -or
        $packageRoot.StartsWith($profilePrefix, [StringComparison]::OrdinalIgnoreCase)) { throw 'GH_PORTABLE_PROFILE_OVERLAP' }
    $attempt = $profile.TrimEnd('/') + '/attempts/' + [guid]::NewGuid().ToString('D')
    $phase = 'launch'
    & $launcher -PwshExe $pwsh -NativeExe $native -NativeSha256 $hashes.NativeSha256 -GodotExe $godot -GodotSha256 $hashes.GodotSha256 -ProjectDir $project -ProfileRoot $profile -AttemptDir $attempt @diagnosticArguments
    exit $LASTEXITCODE
} catch {
    $reason = 'GH_PORTABLE_REJECTED'
    if ($_.Exception.Message -cmatch '^GH_[A-Z_]{1,80}$') { $reason = $_.Exception.Message }
    [Console]::Error.WriteLine((ConvertTo-Json -Compress -InputObject @{ phase=$phase; error=$reason }))
    exit 64
}
