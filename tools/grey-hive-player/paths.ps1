# Host checks (source-only; no work occurs until a function is called).
function Assert-GhHost {
    param([Parameter(Mandatory)][string]$PwshExe)
    if (-not $IsWindows -or $PSVersionTable.PSEdition -cne 'Core' -or $PSVersionTable.PSVersion -lt [version]'7.4') { throw 'GH_HOST' }
    if ($null -eq [Diagnostics.ProcessStartInfo].GetProperty('ArgumentList')) { throw 'GH_HOST_ARGUMENTS' }
    $strictUtf8 = [Text.UTF8Encoding]::new($false, $true)
    $jsonOptions = [Text.Json.JsonDocumentOptions]::new(); $jsonOptions.MaxDepth = 16
    if ($null -eq $strictUtf8 -or $jsonOptions.MaxDepth -ne 16) { throw 'GH_HOST_JSON' }
    $expected = Resolve-GhOrdinaryPath $PwshExe File
    $actual = Resolve-GhOrdinaryPath ([Environment]::ProcessPath) File
    if (-not [string]::Equals($actual, $expected, [StringComparison]::OrdinalIgnoreCase)) { throw 'GH_HOST_IMAGE' }
    $cim = Get-Command -Name 'CimCmdlets\Get-CimClass' -CommandType Cmdlet -ErrorAction Stop
    if ($null -eq (Get-Command -Name 'CimCmdlets\Get-CimInstance' -CommandType Cmdlet -ErrorAction Stop)) { throw 'GH_HOST_CIM' }
    $null = & $cim -Namespace 'root/cimv2' -ClassName 'Win32_Process' -OperationTimeoutSec 5 -ErrorAction Stop
    $null = & $cim -Namespace 'root/StandardCimv2' -ClassName 'MSFT_NetTCPConnection' -OperationTimeoutSec 5 -ErrorAction Stop
    return $expected
}
# Path/containment checks use the OS filesystem, never PowerShell drive expansion.
function Resolve-GhOrdinaryPath {
    param([Parameter(Mandatory)][string]$Path, [Parameter(Mandatory)][ValidateSet('File','Directory')][string]$Kind, [bool]$MissingLeafAllowed = $false)
    $p = $Path.Replace('\', '/')
    if ($p -cnotmatch '^[A-Za-z]:/' -or $p -match '[\p{Cc}<>"|?*\[\]]') { throw 'GH_PATH_LITERAL' }
    $root = $p.Substring(0, 1).ToUpperInvariant() + ':/'
    $parts = @(); if ($p.Length -ne 3) { $parts = @($p.Substring(3).Split('/')) }
    foreach ($part in $parts) {
        if ($part.Length -eq 0 -or $part -in '.', '..' -or $part -match '[:\\]' -or $part -match '[. ]$' -or $part -match '^(?i:CON|PRN|AUX|NUL|CONIN\$|CONOUT\$|COM[1-9¹²³]|LPT[1-9¹²³])(?:\.|$)') { throw 'GH_PATH_COMPONENT' }
    }
    $a = [IO.File]::GetAttributes($root)
    if (($a -band [IO.FileAttributes]::ReparsePoint) -ne 0 -or ($a -band [IO.FileAttributes]::Directory) -eq 0) { throw 'GH_PATH_ROOT' }
    $current = $root
    for ($i = 0; $i -lt $parts.Count; $i++) {
        $current = $current.TrimEnd('/') + '/' + $parts[$i]; $last = $i -eq ($parts.Count - 1)
        try { $a = [IO.File]::GetAttributes($current) }
        catch [IO.FileNotFoundException] { if ($last -and $MissingLeafAllowed) { return $current }; throw 'GH_PATH_MISSING' }
        catch [IO.DirectoryNotFoundException] { if ($last -and $MissingLeafAllowed) { return $current }; throw 'GH_PATH_MISSING' }
        if (($a -band [IO.FileAttributes]::ReparsePoint) -ne 0) { throw 'GH_PATH_REPARSE' }
        $directory = ($a -band [IO.FileAttributes]::Directory) -ne 0
        if ((-not $last -and -not $directory) -or ($last -and ($directory -ne ($Kind -eq 'Directory')))) { throw 'GH_PATH_KIND' }
    }
    if ($parts.Count -eq 0 -and $Kind -ne 'Directory') { throw 'GH_PATH_KIND' }
    return $current
}
function Assert-GhContainedPath {
    param([Parameter(Mandatory)][string]$Root, [Parameter(Mandatory)][string]$Candidate)
    foreach ($p in @($Root, $Candidate)) {
        if ($p -cnotmatch '^[A-Z]:/' -or $p -match '[\\\p{Cc}<>"|?*\[\]]') { throw 'GH_PATH_CANONICAL' }
        if ($p.Length -gt 3) { foreach ($part in $p.Substring(3).Split('/')) { if ($part.Length -eq 0 -or $part -in '.', '..' -or $part -match ':|[. ]$' -or $part -match '^(?i:CON|PRN|AUX|NUL|CONIN\$|CONOUT\$|COM[1-9¹²³]|LPT[1-9¹²³])(?:\.|$)') { throw 'GH_PATH_COMPONENT' } } }
    }
    if (-not $Candidate.StartsWith($Root.TrimEnd('/') + '/', [StringComparison]::OrdinalIgnoreCase)) { throw 'GH_PATH_OUTSIDE' }
}
# Strict JSON: bounded bytes, no BOM/coercion, ordinal keys at every object level.
function Read-GhJsonObject {
    param([Parameter(Mandatory)][IO.Stream]$Stream, [Parameter(Mandatory)][int]$MaximumBytes, [Parameter(Mandatory)][string[]]$ExactKeys)
    if (-not $Stream.CanRead -or $MaximumBytes -lt 1 -or $MaximumBytes -gt 1048576) { throw 'GH_JSON_LIMIT' }
    $bytes = [byte[]]::new($MaximumBytes + 1); $count = 0
    while ($count -lt $bytes.Length) { $n = $Stream.Read($bytes, $count, $bytes.Length - $count); if ($n -eq 0) { break }; $count += $n }
    if ($count -eq 0 -or $count -gt $MaximumBytes) { throw 'GH_JSON_SIZE' }
    $text = [Text.UTF8Encoding]::new($false, $true).GetString($bytes, 0, $count)
    $options = [Text.Json.JsonDocumentOptions]::new(); $options.MaxDepth = 16; $options.AllowTrailingCommas = $false; $options.CommentHandling = [Text.Json.JsonCommentHandling]::Disallow
    $doc = [Text.Json.JsonDocument]::Parse($text, $options)
    try {
        if ($doc.RootElement.ValueKind -ne [Text.Json.JsonValueKind]::Object) { throw 'GH_JSON_OBJECT' }
        $pending = [Collections.Generic.Stack[Text.Json.JsonElement]]::new(); $pending.Push($doc.RootElement)
        while ($pending.Count -gt 0) {
            $node = $pending.Pop(); $seen = [Collections.Generic.HashSet[string]]::new([StringComparer]::OrdinalIgnoreCase)
            if ($node.ValueKind -eq [Text.Json.JsonValueKind]::Object) { foreach ($property in $node.EnumerateObject()) { if (-not $seen.Add($property.Name)) { throw 'GH_JSON_DUPLICATE' }; $pending.Push($property.Value) } }
            elseif ($node.ValueKind -eq [Text.Json.JsonValueKind]::Array) { foreach ($value in $node.EnumerateArray()) { $pending.Push($value) } }
        }
        $keys = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
        foreach ($key in $ExactKeys) { if ([string]::IsNullOrEmpty($key) -or -not $keys.Add($key)) { throw 'GH_JSON_KEYS' } }
        $result = [Collections.Generic.Dictionary[string,Text.Json.JsonElement]]::new([StringComparer]::Ordinal)
        foreach ($property in $doc.RootElement.EnumerateObject()) { if (-not $keys.Remove($property.Name)) { throw 'GH_JSON_KEYS' }; $result.Add($property.Name, $property.Value.Clone()) }
        if ($keys.Count -ne 0) { throw 'GH_JSON_KEYS' }
        return ,$result
    } finally { $doc.Dispose() }
}
# Immutable plan: all values are strings; no profile allocation or mutation.
function New-GhPlayerPlan {
    param([Parameter(Mandatory)][Collections.IDictionary]$Parameters)
    $required = @('PwshExe','NativeExe','NativeSha256','GodotExe','GodotSha256','ProjectDir','ProfileRoot','AttemptDir'); $keys = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    foreach ($key in $required) { $null = $keys.Add($key) }; foreach ($key in $Parameters.Keys) { if ($key -isnot [string] -or -not $keys.Remove($key) -or $Parameters[$key] -isnot [string] -or [string]::IsNullOrWhiteSpace($Parameters[$key])) { throw 'GH_PLAN_PARAMETERS' } }; if ($keys.Count) { throw 'GH_PLAN_PARAMETERS' }
    $plan = [Collections.Generic.Dictionary[string,object]]::new([StringComparer]::Ordinal); $plan.Add('PwshExe', (Assert-GhHost $Parameters['PwshExe']))
    foreach ($key in @('NativeExe','GodotExe')) { $plan.Add($key, (Resolve-GhOrdinaryPath $Parameters[$key] File)) }
    foreach ($key in @('ProjectDir','ProfileRoot')) { $plan.Add($key, (Resolve-GhOrdinaryPath $Parameters[$key] Directory ($key -eq 'ProfileRoot'))) }
    $plan.Add('LauncherDir', (Resolve-GhOrdinaryPath $PSScriptRoot Directory))
    foreach ($key in @('Native','Godot')) {
        $hash = $Parameters[$key + 'Sha256']; if ($hash -cnotmatch '^[0-9a-fA-F]{64}$') { throw 'GH_PLAN_HASH' }
        $file = [IO.File]::Open($plan[$key + 'Exe'], [IO.FileMode]::Open, [IO.FileAccess]::Read, [IO.FileShare]::Read)
        try { $actual = [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($file)).ToLowerInvariant() } finally { $file.Dispose() }
        if ($actual -cne $hash.ToLowerInvariant()) { throw 'GH_PLAN_HASH' }; $plan.Add($key + 'Sha256', $actual)
    }
    $trees = @($plan.ProjectDir, $plan.LauncherDir) + @(@('PwshExe','NativeExe','GodotExe') | ForEach-Object { [IO.Path]::GetDirectoryName($plan[$_]).Replace('\','/') })
    foreach ($tree in $trees) { $r = $plan.ProfileRoot.TrimEnd('/'); $t = $tree.TrimEnd('/'); if ([string]::Equals($r,$t,[StringComparison]::OrdinalIgnoreCase) -or $r.StartsWith($t + '/', [StringComparison]::OrdinalIgnoreCase) -or $t.StartsWith($r + '/', [StringComparison]::OrdinalIgnoreCase)) { throw 'GH_PLAN_OVERLAP' } }
    $plan.Add('AttemptsDir', $plan.ProfileRoot.TrimEnd('/') + '/attempts'); $attempt = $Parameters['AttemptDir'].Replace('\','/'); $id = ($attempt -split '/')[-1]; $uuid = [guid]::Empty
    if (-not [guid]::TryParseExact($id, 'D', [ref]$uuid) -or $id -cne $uuid.ToString('D') -or -not [string]::Equals($attempt, $plan.AttemptsDir + '/' + $id, [StringComparison]::OrdinalIgnoreCase)) { throw 'GH_PLAN_ATTEMPT' }
    $profilePresent = $true; try { $null = [IO.File]::GetAttributes($plan.ProfileRoot) } catch [IO.FileNotFoundException] { $profilePresent = $false } catch [IO.DirectoryNotFoundException] { $profilePresent = $false }
    if ($profilePresent) { $null = Resolve-GhOrdinaryPath $plan.AttemptsDir Directory $true }
    try { $null = [IO.File]::GetAttributes($attempt); throw 'GH_PLAN_ATTEMPT_EXISTS' } catch [IO.FileNotFoundException] {} catch [IO.DirectoryNotFoundException] {}
    $plan.Add('AttemptId', $id); $plan.Add('AttemptDir', $plan.AttemptsDir + '/' + $id)
    foreach ($entry in @{ SessionDir='session'; SaveDir='session/saves'; RunDir='session/run'; ReadyPath='session/run/bridge-ready.json'; ClosuresDir='closures'; RoamingDir='roaming'; LocalDir='local'; TempDir='temp' }.GetEnumerator()) { $plan.Add($entry.Key, $plan.ProfileRoot.TrimEnd('/') + '/' + $entry.Value) }
    foreach ($key in @('SessionDir','SaveDir','RunDir','ReadyPath','ClosuresDir','RoamingDir','LocalDir','TempDir')) { try { $null = [IO.File]::GetAttributes($plan[$key]); $kind = if ($key -eq 'ReadyPath') { 'File' } else { 'Directory' }; $null = Resolve-GhOrdinaryPath $plan[$key] $kind } catch [IO.FileNotFoundException] {} catch [IO.DirectoryNotFoundException] {} }
    $plan.Add('GodotLog', $plan.AttemptDir + '/godot.log')
    return ,([Collections.ObjectModel.ReadOnlyDictionary[string,object]]::new($plan))
}
