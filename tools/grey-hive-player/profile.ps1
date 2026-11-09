# Admission: only a new/empty root or an exact launcher-owned namespace is accepted.
function Open-GhProfile {
    param([Parameter(Mandatory)][Collections.IDictionary]$Plan)
    $lease = [pscustomobject]@{ Plan=$Plan; ProfileId=$null; LockStream=$null; OwnerStream=$null; JournalHandle=$null; JournalIdentity=$null; JournalSha256=$null; AttemptId=$null; ActivePath=$Plan.ProfileRoot + '/active-launch.json'; Started=$false; Completed=$false; Released=$false }
    try {
        $root = Resolve-GhOrdinaryPath $Plan.ProfileRoot Directory $true
        $present = $true; try { $null = [IO.File]::GetAttributes($root) } catch [IO.FileNotFoundException] { $present = $false } catch [IO.DirectoryNotFoundException] { $present = $false }
        if (-not $present) { [Wuxian.Player.PlayerWindows]::CreateDirectoryNew($root) }
        $entries = [IO.Directory]::EnumerateFileSystemEntries($root).GetEnumerator()
        try { $empty = -not $entries.MoveNext() } finally { if ($entries -is [IDisposable]) { $entries.Dispose() } }
        $ownerPath = $root + '/profile-owner.json'; $lockPath = $root + '/profile.lock'
        if ($empty) {
            $null = Resolve-GhOrdinaryPath $lockPath File $true
            $lease.LockStream = [IO.File]::Open($lockPath, [IO.FileMode]::CreateNew, [IO.FileAccess]::ReadWrite, [IO.FileShare]::None)
            foreach ($path in @($Plan.SessionDir,$Plan.SaveDir,$Plan.RunDir,$Plan.RoamingDir,$Plan.LocalDir,$Plan.TempDir,$Plan.AttemptsDir,$Plan.ClosuresDir)) {
                Assert-GhContainedPath $root $path; $null = Resolve-GhOrdinaryPath $path Directory $true
                [Wuxian.Player.PlayerWindows]::CreateDirectoryNew($path)
            }
            $lease.ProfileId = [guid]::NewGuid().ToString('D')
            $owner = [ordered]@{ schema='wuxian.grey-hive.profile/v1'; profileId=$lease.ProfileId; profileRoot=$root; sessionDir=$Plan.SessionDir; saveDir=$Plan.SaveDir }
            $bytes = [Text.UTF8Encoding]::new($false,$true).GetBytes(($owner | ConvertTo-Json -Compress -Depth 3))
            if ($bytes.Length -gt 4096) { throw 'GH_PROFILE_OWNER_SIZE' }
            $writer = [IO.File]::Open($ownerPath,[IO.FileMode]::CreateNew,[IO.FileAccess]::ReadWrite,[IO.FileShare]::None)
            try { $writer.Write($bytes,0,$bytes.Length); $writer.Flush($true) } finally { $writer.Dispose() }
        } else {
            $null = Resolve-GhOrdinaryPath $ownerPath File; $null = Resolve-GhOrdinaryPath $lockPath File
            $lease.LockStream = [IO.File]::Open($lockPath,[IO.FileMode]::Open,[IO.FileAccess]::ReadWrite,[IO.FileShare]::None)
        }
        $allowed = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
        foreach ($name in @('profile-owner.json','profile.lock','session','roaming','local','temp','attempts','closures')) { $null = $allowed.Add($name) }
        foreach ($path in [IO.Directory]::EnumerateFileSystemEntries($root)) {
            $name = [IO.Path]::GetFileName($path)
            if (-not $allowed.Remove($name)) { throw 'GH_PROFILE_UNKNOWN_OR_ACTIVE' }
            $kind = if ($name -in 'profile-owner.json','profile.lock') { 'File' } else { 'Directory' }
            $null = Resolve-GhOrdinaryPath $path $kind
        }
        if ($allowed.Count -ne 0) { throw 'GH_PROFILE_NAMESPACE' }
        $sessionNames = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
        $null = $sessionNames.Add('saves'); $null = $sessionNames.Add('run')
        foreach ($path in [IO.Directory]::EnumerateFileSystemEntries($Plan.SessionDir)) { if (-not $sessionNames.Remove([IO.Path]::GetFileName($path))) { throw 'GH_PROFILE_SESSION' }; $null = Resolve-GhOrdinaryPath $path Directory }
        if ($sessionNames.Count -ne 0) { throw 'GH_PROFILE_SESSION' }
        $entries = [IO.Directory]::EnumerateFileSystemEntries($Plan.RunDir).GetEnumerator()
        try { if ($entries.MoveNext()) { throw 'GH_PROFILE_RUN_OCCUPIED' } } finally { if ($entries -is [IDisposable]) { $entries.Dispose() } }
        $handle = [Wuxian.Player.PlayerWindows]::OpenReadPinned($ownerPath,$false)
        try { $lease.OwnerStream = [IO.FileStream]::new($handle,[IO.FileAccess]::Read) } catch { $handle.Dispose(); throw }
        $owner = Read-GhJsonObject $lease.OwnerStream 4096 @('schema','profileId','profileRoot','sessionDir','saveDir')
        foreach ($key in $owner.Keys) { if ($owner[$key].ValueKind -ne [Text.Json.JsonValueKind]::String) { throw 'GH_PROFILE_OWNER_TYPE' } }
        $id = $owner['profileId'].GetString(); $uuid = [guid]::Empty
        if (-not [guid]::TryParseExact($id,'D',[ref]$uuid) -or $id -cne $uuid.ToString('D') -or $uuid -eq [guid]::Empty) { throw 'GH_PROFILE_OWNER_ID' }
        if ($owner['schema'].GetString() -cne 'wuxian.grey-hive.profile/v1' -or $owner['profileRoot'].GetString() -cne $root -or $owner['sessionDir'].GetString() -cne $Plan.SessionDir -or $owner['saveDir'].GetString() -cne $Plan.SaveDir) { throw 'GH_PROFILE_OWNER_BINDING' }
        $lease.ProfileId = $id
        return $lease
    } catch { Release-GhProfile $lease; throw }
}
# The create-new, flushed journal precedes all child starts and survives any failure.
function Start-GhProfileAttempt {
    param([Parameter(Mandatory)]$ProfileLease,[Parameter(Mandatory)][Collections.IDictionary]$Plan)
    $l = $ProfileLease
    if ($l.Released -or $l.Started -or $l.Completed -or -not [object]::ReferenceEquals($l.Plan,$Plan) -or $null -eq $l.LockStream -or -not $l.LockStream.CanWrite) { throw 'GH_PROFILE_LEASE' }
    $null = Resolve-GhOrdinaryPath $Plan.RunDir Directory
    $entries = [IO.Directory]::EnumerateFileSystemEntries($Plan.RunDir).GetEnumerator()
    try { if ($entries.MoveNext()) { throw 'GH_PROFILE_RUN_OCCUPIED' } } finally { if ($entries -is [IDisposable]) { $entries.Dispose() } }
    $null = Resolve-GhOrdinaryPath $l.ActivePath File $true
    $record = [ordered]@{ schema='wuxian.grey-hive.active-launch/v1'; profileId=$l.ProfileId; attemptId=$Plan.AttemptId }
    $bytes = [Text.UTF8Encoding]::new($false,$true).GetBytes(($record | ConvertTo-Json -Compress -Depth 3))
    if ($bytes.Length -gt 1024) { throw 'GH_PROFILE_JOURNAL_SIZE' }
    $writer = [IO.File]::Open($l.ActivePath,[IO.FileMode]::CreateNew,[IO.FileAccess]::ReadWrite,[IO.FileShare]::None)
    $l.Started = $true; $l.AttemptId = $Plan.AttemptId
    try { $writer.Write($bytes,0,$bytes.Length); $writer.Flush($true); $l.JournalIdentity = [Wuxian.Player.PlayerWindows]::ReadIdentity($writer.SafeFileHandle) } finally { $writer.Dispose() }
    $l.JournalSha256 = [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($bytes))
    $l.JournalHandle = [Wuxian.Player.PlayerWindows]::OpenReadPinned($l.ActivePath,$false)
    $identity = [Wuxian.Player.PlayerWindows]::ReadIdentity($l.JournalHandle)
    foreach ($key in @('VolumeSerialNumber','FileIdLow','FileIdHigh','CreationTimeUtcTicks','Length','Attributes')) { if ($identity.$key -ne $l.JournalIdentity.$key) { throw 'GH_PROFILE_JOURNAL_CHANGED' } }
    $null = Resolve-GhOrdinaryPath $Plan.AttemptsDir Directory
    $null = Resolve-GhOrdinaryPath $Plan.AttemptDir Directory $true
    Assert-GhContainedPath $Plan.AttemptsDir $Plan.AttemptDir
    [Wuxian.Player.PlayerWindows]::CreateDirectoryNew($Plan.AttemptDir)
}
# Caller supplies only observed clean closure facts; zero exit codes do not attest saves.
function Complete-GhProfile {
    param([Parameter(Mandatory)]$ProfileLease,[Parameter(Mandatory)][Collections.IDictionary]$ClosureFacts)
    $l = $ProfileLease
    if ($l.Released -or -not $l.Started -or $l.Completed -or $null -eq $l.JournalHandle -or $l.JournalHandle.IsClosed -or $null -eq $l.LockStream -or -not $l.LockStream.CanWrite) { throw 'GH_PROFILE_LEASE' }
    $keys = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    foreach ($key in @('nativeNaturalExit','godotNaturalExit','nativeExitCode','godotExitCode','drainsCompleted','readyRemoved')) { $null = $keys.Add($key) }
    foreach ($key in $ClosureFacts.Keys) { if ($key -isnot [string] -or -not $keys.Remove($key)) { throw 'GH_PROFILE_CLOSURE_KEYS' } }
    if ($keys.Count -ne 0) { throw 'GH_PROFILE_CLOSURE_KEYS' }
    foreach ($key in @('nativeNaturalExit','godotNaturalExit','drainsCompleted','readyRemoved')) { if ($ClosureFacts[$key] -isnot [bool] -or -not $ClosureFacts[$key]) { throw 'GH_PROFILE_UNCLEAN' } }
    foreach ($key in @('nativeExitCode','godotExitCode')) { if ($ClosureFacts[$key] -isnot [int] -or $ClosureFacts[$key] -ne 0) { throw 'GH_PROFILE_UNCLEAN' } }
    $null = Resolve-GhOrdinaryPath $l.Plan.RunDir Directory
    $entries = [IO.Directory]::EnumerateFileSystemEntries($l.Plan.RunDir).GetEnumerator()
    try { if ($entries.MoveNext()) { throw 'GH_PROFILE_RUN_OCCUPIED' } } finally { if ($entries -is [IDisposable]) { $entries.Dispose() } }
    $receipt = [ordered]@{ schema='wuxian.grey-hive.closure/v1'; profileId=$l.ProfileId; attemptId=$l.AttemptId; nativeNaturalExit=$true; godotNaturalExit=$true; nativeExitCode=0; godotExitCode=0; drainsCompleted=$true; readyRemoved=$true; saveAcceptance='NOT_OBSERVED'; continueAcceptance='NOT_OBSERVED' }
    $bytes = [Text.UTF8Encoding]::new($false,$true).GetBytes(($receipt | ConvertTo-Json -Compress -Depth 3))
    if ($bytes.Length -gt 4096) { throw 'GH_PROFILE_RECEIPT_SIZE' }
    $path = $l.Plan.ClosuresDir + '/' + $l.AttemptId + '.json'
    $null = Resolve-GhOrdinaryPath $path File $true; Assert-GhContainedPath $l.Plan.ClosuresDir $path
    $writer = [IO.File]::Open($path,[IO.FileMode]::CreateNew,[IO.FileAccess]::ReadWrite,[IO.FileShare]::None)
    try { $writer.Write($bytes,0,$bytes.Length); $writer.Flush($true) } finally { $writer.Dispose() }
    $l.JournalHandle.Dispose(); $l.JournalHandle = $null
    $null = Resolve-GhOrdinaryPath $l.ActivePath File
    $l.JournalHandle = [Wuxian.Player.PlayerWindows]::OpenReadPinned($l.ActivePath,$true)
    $identity = [Wuxian.Player.PlayerWindows]::ReadIdentity($l.JournalHandle)
    foreach ($key in @('VolumeSerialNumber','FileIdLow','FileIdHigh','CreationTimeUtcTicks','Length','Attributes')) { if ($identity.$key -ne $l.JournalIdentity.$key) { throw 'GH_PROFILE_JOURNAL_CHANGED' } }
    $stream = [IO.FileStream]::new($l.JournalHandle,[IO.FileAccess]::Read)
    try {
        if ([Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($stream)) -cne $l.JournalSha256) { throw 'GH_PROFILE_JOURNAL_CHANGED' }
        [Wuxian.Player.PlayerWindows]::DeletePinned($l.JournalHandle); $l.JournalHandle = $null; $l.Completed = $true
    } finally { $stream.Dispose() }
}
# Release resources only: never remove failed journals, saves, ready files or directories.
function Release-GhProfile {
    param([Parameter(Mandatory)]$ProfileLease)
    if ($ProfileLease.Released) { return }
    try {
        try { if ($null -ne $ProfileLease.JournalHandle) { $ProfileLease.JournalHandle.Dispose(); $ProfileLease.JournalHandle = $null } }
        finally { try { if ($null -ne $ProfileLease.OwnerStream) { $ProfileLease.OwnerStream.Dispose(); $ProfileLease.OwnerStream = $null } } finally { if ($null -ne $ProfileLease.LockStream) { $ProfileLease.LockStream.Dispose(); $ProfileLease.LockStream = $null } } }
    } finally { $ProfileLease.Released = $true }
}
