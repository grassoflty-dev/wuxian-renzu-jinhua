param(
    [ValidateSet('Plan','Run')][string]$Mode = 'Plan',
    [string]$ManifestPath,
    [string]$ManifestSha256,
    [string]$ReceiptPath,
    [string]$ReceiptSha256
)
$ErrorActionPreference = 'Stop'
$script:TaskId = 'EXEC-FOUNDRY-SHORT-FACES-OWNED-OBSERVATION-INFRA-20261021-001/r1'
$script:PipeStates = $null
$script:ManifestSpecJson = '{"schema":"wuxian.foundry-owned-observation/v1","rootKeys":["schema","runId","sourceHead","sourcePins","artifacts","seed","scene","resources","optionalAssets","paths","limits","deadlineUtc"],"pinKeys":["path","bytes","sha256"],"resourceKeys":["path","bytes","sha256","presence","required","purposeRef"],"seed":{"path":"server-rs/tests/fixtures/earned-cw-foundry-entry-v6.json","bytes":14242,"sha256":"233275e296937b66956159f6bd89f9e6d08130298fd3ebfef84e52363158a0fb"},"scene":{"path":"apps/godot_playable/data/scenes/cw_entry_foundry.json","bytes":5551,"sha256":"d90f852367b998cf5874b82fed583f1254664d4a88a8175bb3c94524b8f496b1"},"limits":{"pairs":1,"photos":4,"rawPipeBytes":1048576,"naturalSeconds":120,"drainSeconds":30,"disposeSeconds":5}}'
function Assert-ExactKeys {
    param([object]$Value,[string[]]$Expected)
    if ($Value -isnot [System.Collections.IDictionary]) { throw 'Expected object' }
    $actual = @($Value.Keys | ForEach-Object { [string]$_ } | Sort-Object)
    $wanted = @($Expected | Sort-Object)
    if (($actual -join '|') -cne ($wanted -join '|')) { throw 'Unexpected object keys' }
}
function Assert-Pin {
    param([object]$Pin,[string[]]$Keys = @('path','bytes','sha256'))
    Assert-ExactKeys $Pin $Keys
    if ($Pin.path -isnot [string] -or [string]::IsNullOrWhiteSpace($Pin.path)) { throw 'Invalid pin path' }
    if ($Pin.bytes -is [bool] -or $Pin.bytes -isnot [ValueType] -or $Pin.bytes -lt 1 -or [math]::Floor([double]$Pin.bytes) -ne [double]$Pin.bytes) { throw 'Invalid pin length' }
    if ($Pin.sha256 -isnot [string] -or $Pin.sha256 -cnotmatch '^[a-f0-9]{64}$') { throw 'Invalid pin digest' }
}
function Assert-ResourceRow {
    param([object]$Row)
    Assert-Pin $Row @('path','bytes','sha256','presence','required','purposeRef')
    if ($Row.presence -cnotin @('present','absent') -or $Row.required -isnot [bool]) { throw 'Invalid resource state' }
    if ($Row.presence -eq 'present' -and ($Row.purposeRef -isnot [string] -or [string]::IsNullOrWhiteSpace($Row.purposeRef))) { throw 'Present resource lacks purpose reference' }
    if ($Row.presence -eq 'absent' -and ($Row.required -or $null -ne $Row.purposeRef)) { throw 'Absent resource is not optional' }
}
function Test-DeepEqual {
    param([object]$Left,[object]$Right)
    if ($Left -is [System.Collections.IDictionary] -or $Right -is [System.Collections.IDictionary]) {
        if ($Left -isnot [System.Collections.IDictionary] -or $Right -isnot [System.Collections.IDictionary] -or $Left.Count -ne $Right.Count) { return $false }
        foreach ($key in $Left.Keys) { if (-not $Right.Contains($key) -or -not (Test-DeepEqual $Left[$key] $Right[$key])) { return $false } }
        return $true
    }
    if ($Left -is [array] -or $Right -is [array]) {
        if ($Left -isnot [array] -or $Right -isnot [array] -or $Left.Count -ne $Right.Count) { return $false }
        for ($index=0;$index -lt $Left.Count;$index++) { if (-not (Test-DeepEqual $Left[$index] $Right[$index])) { return $false } }
        return $true
    }
    if ($Left -is [ValueType] -and $Right -is [ValueType] -and $Left -isnot [bool] -and $Right -isnot [bool]) { return [double]$Left -eq [double]$Right }
    return $Left -ceq $Right
}
function Assert-NoReparseAncestors {
    param([string]$Path)
    $full = [System.IO.Path]::GetFullPath($Path)
    $root = [System.IO.Path]::GetPathRoot($full)
    $cursor = $root
    $rest = $full.Substring($root.Length)
    foreach ($part in ($rest -split '[\\/]')) {
        if ([string]::IsNullOrEmpty($part)) { continue }
        $cursor = Join-Path $cursor $part
        if (-not (Test-Path -LiteralPath $cursor)) { break }
        $item = Get-Item -LiteralPath $cursor -Force
        if (($item.Attributes -band [System.IO.FileAttributes]::ReparsePoint) -ne 0) { throw 'Reparse point in owned path' }
    }
}
function Assert-OrdinaryNamespace {
    param([object]$Manifest)
    $paths = $Manifest.paths
    Assert-ExactKeys $paths @('projectRoot','root','roaming','local','temp','run','saves','outputs','ready','logFile')
    $project = 'C:/Users/GWL/Desktop/wuxian-renzu-jinhua'
    $prefix = 'C:/Users/GWL/.codex/visualizations/2026/10/21/01a147f2-ef43-7cd3-9785-e0263fd4e60e/FOUNDRY-SHORT-FACES-OBSERVATION-20261021-001/r1'
    $id = [guid]::ParseExact([string]$Manifest.runId,'D').ToString('D')
    $root = '{0}/{1}' -f $prefix,$id
    $expected = @{
        projectRoot=$project; root=$root; roaming=('{0}/profile/roaming' -f $root); local=('{0}/profile/local' -f $root); temp=('{0}/profile/temp' -f $root)
        run=('{0}/session/run' -f $root); saves=('{0}/session/saves' -f $root); outputs=('{0}/outputs' -f $root)
        ready=('{0}/session/run/bridge-ready.json' -f $root); logFile=('{0}/outputs/godot.log' -f $root)
    }
    foreach ($key in $expected.Keys) {
        $value = [string]$paths[$key]
        if ($value -cne $expected[$key] -or [System.IO.Path]::GetFullPath($value).Replace('\','/') -cne $value.TrimEnd('/')) { throw 'Noncanonical namespace path' }
        Assert-NoReparseAncestors $value
    }
    if (Test-Path -LiteralPath $root) { throw 'Run namespace already exists' }
}
function Assert-Manifest {
    param([object]$Manifest)
    $spec = ConvertFrom-Json -InputObject $script:ManifestSpecJson -AsHashtable
    Assert-ExactKeys $Manifest $spec.rootKeys
    if ($Manifest.schema -cne $spec.schema -or $Manifest.runId -isnot [string]) { throw 'Manifest identity mismatch' }
    $null = [guid]::ParseExact($Manifest.runId,'D')
    if ($Manifest.sourceHead -isnot [string] -or $Manifest.sourceHead -cnotmatch '^[a-f0-9]{40}$') { throw 'Invalid source HEAD' }
    if (-not (Test-DeepEqual $Manifest.seed $spec.seed) -or -not (Test-DeepEqual $Manifest.scene $spec.scene) -or -not (Test-DeepEqual $Manifest.limits $spec.limits)) { throw 'Fixed manifest contract mismatch' }
    if ($Manifest.sourcePins -isnot [array] -or $Manifest.sourcePins.Count -lt 7) { throw 'Incomplete source closure' }
    $seenPins = @{}
    foreach ($pin in $Manifest.sourcePins) {
        Assert-Pin $pin $spec.pinKeys
        if ($pin.path.StartsWith('/') -or $pin.path.Contains('\') -or $pin.path.Contains(':') -or $pin.path.Split('/') -contains '..' -or $pin.path -match '\.(png|jpe?g|webp|bmp|tga|ctex|svg|gif|dds|ktx2?|ico)$') { throw 'Unsafe source pin path' }
        if ($seenPins.ContainsKey($pin.path)) { throw 'Duplicate source pin' }
        $seenPins[$pin.path] = $true
    }
    $required = @('apps/godot_playable/tests/world/foundry_short_faces_owned_observation.gd','tools/foundry-short-faces-observation/launch.ps1','tools/foundry-short-faces-observation/contract.test.mjs','apps/godot_playable/scripts/bridge/protocol.gd','apps/godot_playable/scripts/bridge/rpc_client.gd','apps/godot_playable/scripts/world/scene_catalog.gd','apps/godot_playable/scripts/world/renderer.gd','server-rs/tests/fixtures/earned-cw-foundry-entry-v6.json','apps/godot_playable/data/scenes/cw_entry_foundry.json')
    foreach ($path in $required) { if (-not $seenPins.ContainsKey($path)) { throw 'Missing required source pin' } }
    Assert-ExactKeys $Manifest.artifacts @('native','godot')
    foreach ($name in @('native','godot')) {
        $artifact = $Manifest.artifacts[$name]
        Assert-Pin $artifact @('path','bytes','sha256','provenance','version')
        if ($artifact.path -isnot [string] -or -not [System.IO.Path]::IsPathFullyQualified($artifact.path) -or $artifact.provenance -isnot [string] -or [string]::IsNullOrWhiteSpace($artifact.provenance) -or $artifact.version -isnot [string] -or [string]::IsNullOrWhiteSpace($artifact.version)) { throw 'Invalid executable identity' }
    }
    if ($Manifest.resources -isnot [array] -or $Manifest.resources.Count -lt 1) { throw 'Resource table must be nonempty' }
    $seenResources = @{}
    foreach ($row in $Manifest.resources) {
        Assert-ResourceRow $row
        $key = ([string]$row.path).ToLowerInvariant()
        if ($seenResources.ContainsKey($key)) { throw 'Duplicate resource path' }
        $seenResources[$key] = $row
    }
    if ($Manifest.optionalAssets -isnot [array] -or $Manifest.optionalAssets.Count -ne 1) { throw 'Optional asset presence record is required' }
    $seenOptional = @{}
    foreach ($row in $Manifest.optionalAssets) {
        Assert-ExactKeys $row @('id','presence','mapPin','resourcePath','purposeRef')
        if ($row.id -isnot [string] -or $row.id -cne 'foundryWallChain' -or $seenOptional.ContainsKey($row.id) -or $row.presence -cnotin @('present','absent')) { throw 'Invalid optional asset record' }
        $seenOptional[$row.id] = $true
        Assert-Pin $row.mapPin $spec.pinKeys
        if (-not $seenPins.ContainsKey($row.mapPin.path) -or -not (Test-DeepEqual $seenPins[$row.mapPin.path] $row.mapPin)) { throw 'Optional source-map pin is not in the source closure' }
        if ($row.resourcePath -isnot [string] -or [string]::IsNullOrWhiteSpace($row.resourcePath)) { throw 'Invalid optional resource path' }
        if ($row.presence -eq 'absent') {
            if ($null -ne $row.purposeRef) { throw 'Absent optional asset has purpose' }
            if ($seenResources.ContainsKey($row.resourcePath.ToLowerInvariant()) -and $seenResources[$row.resourcePath.ToLowerInvariant()].presence -ne 'absent') { throw 'Optional presence mismatch' }
        } else {
            if ($row.purposeRef -isnot [string] -or -not $seenResources.ContainsKey($row.resourcePath.ToLowerInvariant())) { throw 'Present optional resource is unbound' }
            $resource = $seenResources[$row.resourcePath.ToLowerInvariant()]
            if ($resource.presence -ne 'present' -or $resource.purposeRef -cne $row.purposeRef) { throw 'Optional purpose mismatch' }
        }
    }
    Assert-OrdinaryNamespace $Manifest
    $deadline = [DateTimeOffset]::ParseExact([string]$Manifest.deadlineUtc,'yyyy-MM-ddTHH:mm:ssZ',[Globalization.CultureInfo]::InvariantCulture,[Globalization.DateTimeStyles]::AssumeUniversal)
    if ($deadline.Offset -ne [TimeSpan]::Zero) { throw 'Deadline must be UTC' }
}
function Read-Manifest {
    param([string]$Path,[string]$Sha256)
    if (-not [System.IO.Path]::IsPathFullyQualified($Path) -or $Sha256 -cnotmatch '^[a-f0-9]{64}$') { throw 'Manifest locator is invalid' }
    $item = Get-Item -LiteralPath $Path
    if ($item.Length -lt 1 -or $item.Length -gt 262144) { throw 'Manifest length is invalid' }
    if ((Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant() -cne $Sha256) { throw 'Manifest digest mismatch' }
    $encoding = [System.Text.UTF8Encoding]::new($false,$true)
    $text = [System.IO.File]::ReadAllText($Path,$encoding)
    $value = ConvertFrom-Json -InputObject $text -AsHashtable
    Assert-Manifest $value
    return $value
}

function Resolve-Plan {
    param([object]$Manifest)
    Assert-OrdinaryNamespace $Manifest
    $paths = $Manifest.paths
    return [pscustomobject]@{
        RunId=$Manifest.runId; ProjectRoot=$paths.projectRoot; Root=$paths.root; Session=$paths.run; Saves=$paths.saves
        Outputs=$paths.outputs; Ready=$paths.ready; LogFile=$paths.logFile; Roaming=$paths.roaming; Local=$paths.local; Temp=$paths.temp
        Native=$Manifest.artifacts.native.path; Godot=$Manifest.artifacts.godot.path
        NativeArguments=@(Get-NativeArguments $paths.run $paths.saves 'PLAN_TOKEN')
        GodotArguments=@(Get-GodotArguments $Manifest 'PLAN_TOKEN')
        RequiredSourcePins=@($Manifest.sourcePins.Count); RequiredResources=@($Manifest.resources.Count); ReadyForRun=$false
    }
}

function Assert-RunAuthority {
    param([object]$Manifest,[object]$Plan,[string]$Path,[string]$Sha256,[switch]$ClosureOnly)
    $script:RunReceipt = Read-Receipt $Path $Sha256
    Assert-ReceiptBinding $script:RunReceipt $Manifest $Plan $Sha256
    Assert-NotExpired $Manifest $script:RunReceipt -ClosureOnly:$ClosureOnly
    Assert-PurposeApproval $Manifest $script:RunReceipt
    if (-not $script:RunReceipt.fallbackPolicy.approved -or $script:RunReceipt.fallbackPolicy.allowedKinds -cnotcontains 'Native' -or $script:RunReceipt.fallbackPolicy.allowedKinds -cnotcontains 'Godot') { throw 'Qualified owned-closure fallback is required before Run' }
}

function Read-Receipt {
    param([string]$Path,[string]$Sha256)
    if (-not [System.IO.Path]::IsPathFullyQualified($Path) -or $Sha256 -cnotmatch '^[a-f0-9]{64}$') { throw 'Receipt locator is invalid' }
    $item = Get-Item -LiteralPath $Path
    if ($item.Length -lt 1 -or $item.Length -gt 524288 -or (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant() -cne $Sha256) { throw 'Receipt identity mismatch' }
    return ConvertFrom-Json -InputObject ([System.IO.File]::ReadAllText($Path,[System.Text.UTF8Encoding]::new($false,$true))) -AsHashtable
}
function Assert-ReceiptBinding {
    param([object]$Receipt,[object]$Manifest,[object]$Plan,[string]$ManifestDigest)
    Assert-ExactKeys $Receipt @('schema','taskId','runId','manifestSha256','checkout','sourcePins','artifacts','resources','purposeApprovals','runtimeLease','deadlineUtc','fallbackPolicy')
    if ($Receipt.schema -cne 'wuxian.foundry-owned-observation-receipt/v1' -or $Receipt.taskId -cne $script:TaskId -or $Receipt.runId -cne $Manifest.runId -or $Receipt.manifestSha256 -cne $ManifestDigest -or $Receipt.deadlineUtc -cne $Manifest.deadlineUtc) { throw 'Receipt does not bind this run' }
    Assert-ExactKeys $Receipt.checkout @('projectRoot','head','indexSha256')
    if ($Receipt.checkout.projectRoot -cne $Manifest.paths.projectRoot -or $Receipt.checkout.head -cne $Manifest.sourceHead -or $Receipt.checkout.indexSha256 -cnotmatch '^[a-f0-9]{64}$') { throw 'Checkout receipt mismatch' }
    Assert-ExactKeys $Receipt.runtimeLease @('leaseId','owner','runId','projectRoot','scope','deadlineUtc')
    $leaseId = [guid]::ParseExact([string]$Receipt.runtimeLease.leaseId,'D').ToString('D')
    if ($leaseId -cne $Receipt.runtimeLease.leaseId) { throw 'Runtime lease ID is not canonical' }
    if ($Receipt.runtimeLease.owner -cne 'MAIN' -or $Receipt.runtimeLease.runId -cne $Manifest.runId -or $Receipt.runtimeLease.projectRoot -cne $Manifest.paths.projectRoot -or $Receipt.runtimeLease.scope -cne 'exclusive_runtime_profile' -or $Receipt.runtimeLease.deadlineUtc -cne $Manifest.deadlineUtc) { throw 'Exclusive runtime lease mismatch' }
    if ($Receipt.sourcePins -isnot [array] -or $Receipt.sourcePins.Count -ne $Manifest.sourcePins.Count) { throw 'Source closure receipt mismatch' }
    foreach ($pin in $Manifest.sourcePins) {
        $matches = @($Receipt.sourcePins | Where-Object { $_.path -ceq $pin.path })
        if ($matches.Count -ne 1 -or -not (Test-DeepEqual $matches[0] $pin)) { throw 'Source pin receipt mismatch' }
    }
    if (-not (Test-DeepEqual $Receipt.artifacts $Manifest.artifacts) -or $Receipt.resources -isnot [array]) { throw 'Artifact or resource receipt mismatch' }
    if ($Receipt.resources.Count -ne $Manifest.resources.Count) { throw 'Resource receipt count mismatch' }
    foreach ($row in $Manifest.resources) {
        $matches = @($Receipt.resources | Where-Object { $_.path -ceq $row.path })
        if ($matches.Count -ne 1 -or -not (Test-DeepEqual $matches[0] $row)) { throw 'Resource receipt mismatch' }
    }
    Assert-ExactKeys $Receipt.fallbackPolicy @('approved','deadlineUtc','allowedKinds')
    if ($Receipt.fallbackPolicy.approved -isnot [bool] -or $Receipt.fallbackPolicy.deadlineUtc -cne $Manifest.deadlineUtc -or $Receipt.fallbackPolicy.allowedKinds -isnot [array] -or $Receipt.fallbackPolicy.allowedKinds.Count -ne 2 -or $Receipt.fallbackPolicy.allowedKinds -cnotcontains 'Native' -or $Receipt.fallbackPolicy.allowedKinds -cnotcontains 'Godot') { throw 'Fallback receipt mismatch' }
}

function Assert-NotExpired {
    param([object]$Manifest,[object]$Receipt,[switch]$ClosureOnly)
    $deadline = [DateTimeOffset]::ParseExact([string]$Manifest.deadlineUtc,'yyyy-MM-ddTHH:mm:ssZ',[Globalization.CultureInfo]::InvariantCulture,[Globalization.DateTimeStyles]::AssumeUniversal)
    $seconds = if ($ClosureOnly) { [double]$Manifest.limits.drainSeconds + [double]$Manifest.limits.disposeSeconds } else { [double]$Manifest.limits.naturalSeconds + [double]$Manifest.limits.drainSeconds + [double]$Manifest.limits.disposeSeconds + 60 }
    $needed = [TimeSpan]::FromSeconds($seconds)
    if ([DateTimeOffset]::UtcNow.Add($needed) -ge $deadline -or $Receipt.deadlineUtc -cne $Manifest.deadlineUtc) { throw 'Insufficient authorized run window' }
}
function Assert-PurposeApproval {
    param([object]$Manifest,[object]$Receipt)
    if ($Receipt.purposeApprovals -isnot [array]) { throw 'Purpose approvals must be an array' }
    $present = @($Manifest.resources | Where-Object { $_.presence -eq 'present' })
    if ($Receipt.purposeApprovals.Count -ne $present.Count) { throw 'Purpose coverage is incomplete or excessive' }
    foreach ($row in $present) {
        $matches = @($Receipt.purposeApprovals | Where-Object { $_.purposeRef -ceq $row.purposeRef -and $_.resourcePath -ceq $row.path })
        if ($matches.Count -ne 1) { throw 'Missing exact resource-purpose approval' }
        $approval = $matches[0]
        Assert-ExactKeys $approval @('purposeRef','resourcePath','sha256','room','operations','approved')
        if ($approval.sha256 -cne $row.sha256 -or $approval.room -cne 'clockworks:cw_entry_foundry' -or $approval.approved -isnot [bool] -or -not $approval.approved -or $approval.operations -isnot [array] -or -not (Test-DeepEqual $approval.operations @('read','decode','display'))) { throw 'Resource purpose is not approved' }
        foreach ($operation in @('read','decode','display')) { if ($approval.operations -cnotcontains $operation) { throw 'Resource purpose lacks required operation' } }
    }
}

function Assert-RunPins {
    param([object]$Manifest,[object]$Plan)
    Assert-HeadIndex $Manifest $script:RunReceipt
    Assert-TextPins $Manifest $Plan
    Assert-ArtifactPins $Manifest
    Assert-ResourcePins $Manifest $Plan
    Assert-SeedPins $Manifest $Plan
}

function Assert-HeadIndex {
    param([object]$Manifest,[object]$Receipt)
    $git = Join-Path $Manifest.paths.projectRoot '.git'
    if (-not (Test-Path -LiteralPath $git -PathType Container)) { throw 'Ordinary .git directory required' }
    $headText = (Get-Content -LiteralPath (Join-Path $git 'HEAD') -Raw -Encoding utf8).Trim()
    if ($headText.StartsWith('ref: ')) {
        $ref = Join-Path $git ($headText.Substring(5).Replace('/',[System.IO.Path]::DirectorySeparatorChar))
        if (Test-Path -LiteralPath $ref) { $head = (Get-Content -LiteralPath $ref -Raw -Encoding ascii).Trim() }
        else {
            $match = Get-Content -LiteralPath (Join-Path $git 'packed-refs') | Where-Object { $_ -match ('^([a-f0-9]{40})\s+'+[regex]::Escape($headText.Substring(5))+'$') } | Select-Object -First 1
            if (-not $match) { throw 'HEAD reference is unresolved' }; $head = $match.Split(' ')[0]
        }
    } else { $head = $headText }
    $indexPath = Join-Path $git 'index'
    if ($head -cne $Manifest.sourceHead -or $head -cne $Receipt.checkout.head -or -not (Test-Path -LiteralPath $indexPath)) { throw 'HEAD or index changed' }
    if ((Get-FileHash -LiteralPath $indexPath -Algorithm SHA256).Hash.ToLowerInvariant() -cne $Receipt.checkout.indexSha256) { throw 'Index identity changed' }
}

function Assert-TextPins {
    param([object]$Manifest,[object]$Plan)
    foreach ($pin in $Manifest.sourcePins) {
        $path = Join-Path $Plan.ProjectRoot $pin.path.Replace('/',[System.IO.Path]::DirectorySeparatorChar)
        $item = Get-Item -LiteralPath $path
        if ($item.Length -ne $pin.bytes -or (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant() -cne $pin.sha256) { throw 'Text/source pin changed' }
    }
}

function Assert-ArtifactPins {
    param([object]$Manifest)
    foreach ($name in @('native','godot')) {
        $row = $Manifest.artifacts[$name]
        $item = Get-Item -LiteralPath $row.path
        $version = [System.Diagnostics.FileVersionInfo]::GetVersionInfo($row.path).FileVersion
        if ($item.Length -ne $row.bytes -or (Get-FileHash -LiteralPath $row.path -Algorithm SHA256).Hash.ToLowerInvariant() -cne $row.sha256 -or $version -cne $row.version) { throw 'Executable artifact changed' }
    }
}

function Assert-ResourcePins {
    param([object]$Manifest,[object]$Plan)
    foreach ($row in $Manifest.resources) {
        if ($row.presence -eq 'absent') { continue }
        if (-not $row.path.StartsWith('res://')) { throw 'Resource locator is not project-relative' }
        $relative = $row.path.Substring(6)
        if ($relative.Contains('..') -or $relative.Contains('\') -or $relative.StartsWith('/')) { throw 'Noncanonical resource locator' }
        $path = Join-Path (Join-Path $Plan.ProjectRoot 'apps/godot_playable') $relative.Replace('/',[System.IO.Path]::DirectorySeparatorChar)
        Assert-NoReparseAncestors $path
        $item = Get-Item -LiteralPath $path
        if ($item.Length -ne $row.bytes -or (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant() -cne $row.sha256) { throw 'Approved resource changed' }
    }
}

function Assert-SeedPins {
    param([object]$Manifest,[object]$Plan)
    $seedPath = Join-Path $Plan.ProjectRoot $Manifest.seed.path.Replace('/',[System.IO.Path]::DirectorySeparatorChar)
    $item = Get-Item -LiteralPath $seedPath
    if ($item.Length -ne $Manifest.seed.bytes -or (Get-FileHash -LiteralPath $seedPath -Algorithm SHA256).Hash.ToLowerInvariant() -cne $Manifest.seed.sha256) { throw 'Earned seed changed' }
}

function New-OwnedNamespace {
    param([object]$Plan)
    Assert-NoReparseAncestors $Plan.Root
    if (Test-Path -LiteralPath $Plan.Root) { throw 'Unique namespace already exists' }
    $null = New-Item -ItemType Directory -Path $Plan.Root -ErrorAction Stop
    foreach ($path in @((Join-Path $Plan.Root 'profile'),$Plan.Roaming,$Plan.Local,$Plan.Temp,(Join-Path $Plan.Root 'session'),$Plan.Session,$Plan.Saves,$Plan.Outputs)) {
        if (Test-Path -LiteralPath $path) { throw 'Unexpected existing namespace descendant' }
        $null = New-Item -ItemType Directory -Path $path -ErrorAction Stop
    }
}

function Materialize-Seed {
    param([object]$Manifest,[object]$Plan)
    $source = Join-Path $Plan.ProjectRoot $Manifest.seed.path.Replace('/',[System.IO.Path]::DirectorySeparatorChar)
    $destination = Join-Path $Plan.Saves 'default.json'
    $seedStream = [System.IO.File]::OpenRead($source)
    try {
        if ($seedStream.Length -ne $Manifest.seed.bytes) { throw 'Seed length changed before materialization' }
        $output = [System.IO.File]::Open($destination,[System.IO.FileMode]::CreateNew,[System.IO.FileAccess]::Write,[System.IO.FileShare]::None)
        try { $seedStream.CopyTo($output); $output.Flush($true) } finally { $output.Dispose() }
    } finally { $seedStream.Dispose() }
    $item = Get-Item -LiteralPath $destination
    if ($item.Length -ne $Manifest.seed.bytes -or (Get-FileHash -LiteralPath $destination -Algorithm SHA256).Hash.ToLowerInvariant() -cne $Manifest.seed.sha256) { throw 'Materialized seed verification failed' }
}

function Get-NativeArguments {
    param([string]$Session,[string]$Saves,[string]$Token)
    return @('--session-dir',$Session,'--save-dir',$Saves,'--session-token',$Token)
}

function Get-RpcArguments {
    param([string]$Ready,[string]$Token)
    return @('--v1-ready',$Ready,'--v1-token',$Token)
}

function Get-GodotArguments {
    param([object]$Manifest,[string]$Token)
    $project = Join-Path $Manifest.paths.projectRoot 'apps/godot_playable'
    $script = Join-Path $project 'tests/world/foundry_short_faces_owned_observation.gd'
    $godotArgs = @('--path',$project,'--script',$script,'--rendering-method','gl_compatibility','--audio-driver','Dummy','--log-file',$Manifest.paths.logFile,'--')
    $godotArgs += Get-RpcArguments $Manifest.paths.ready $Token
    return $godotArgs
}

function Start-OwnedProcess {
    param([string]$Kind,[string]$Executable,[string[]]$Arguments,[string]$WorkingDirectory,[hashtable]$Environment,[ref]$Registry)
    $psi = [System.Diagnostics.ProcessStartInfo]::new()
    $psi.FileName = $Executable
    $psi.WorkingDirectory = $WorkingDirectory
    $psi.UseShellExecute = $false
    $psi.CreateNoWindow = $true
    $psi.RedirectStandardOutput = $true
    $psi.RedirectStandardError = $true
    foreach ($argument in $Arguments) { $psi.ArgumentList.Add([string]$argument) }
    $psi.Environment['APPDATA'] = [string]$Environment['APPDATA']
    $psi.Environment['LOCALAPPDATA'] = [string]$Environment['LOCALAPPDATA']
    $psi.Environment['TEMP'] = [string]$Environment['TEMP']
    $psi.Environment['TMP'] = [string]$Environment['TMP']
    $psi.Environment['WX_FOUNDRY_OBS_MANIFEST'] = [string]$Environment['WX_FOUNDRY_OBS_MANIFEST']
    $psi.Environment['WX_FOUNDRY_OBS_MANIFEST_SHA256'] = [string]$Environment['WX_FOUNDRY_OBS_MANIFEST_SHA256']
    $process = [System.Diagnostics.Process]::new()
    $process.StartInfo = $psi
    $record = [pscustomobject]@{ Kind=$Kind; Process=$process; Executable=$Executable; Arguments=@($Arguments); Started=$false; StartTimeUtc=$null; MainModule=$null; CommandLine=$null; PipeState=$null; ExitCode=$null }
    $Registry.Value[$Kind] = $record
    if (-not $process.Start()) { throw 'Owned process did not start' }
    $record.Started = $true
    $process.Refresh()
    $record.StartTimeUtc = $process.StartTime.ToUniversalTime()
    $record.MainModule = $process.MainModule.FileName
    $record.CommandLine = (Get-CimInstance Win32_Process -Filter ('ProcessId = {0}' -f $process.Id)).CommandLine
    if ([string]::IsNullOrWhiteSpace($record.CommandLine)) { throw 'Started process identity unavailable' }
    return $record
}

function Assert-ProcessIdentity {
    param([object]$Record)
    if (-not $Record.Started -or $null -eq $Record.Process) { throw 'No retained process identity' }
    $process = $Record.Process
    $process.Refresh()
    if ($process.HasExited -or $process.StartTime.ToUniversalTime() -ne $Record.StartTimeUtc -or $process.MainModule.FileName -cne $Record.MainModule -or $Record.MainModule -cne $Record.Executable -or -not (Test-DeepEqual @($process.StartInfo.ArgumentList) $Record.Arguments)) { throw 'Retained process identity changed' }
    $current = Get-CimInstance Win32_Process -Filter ('ProcessId = {0}' -f $process.Id)
    if ($null -eq $current -or $current.CommandLine -cne $Record.CommandLine) { throw 'Process command line changed' }
    return $true
}

function Assert-ReadyOwner {
    param([hashtable]$Owned,[object]$Plan,[object]$Manifest,[string]$Token,[int]$TimeoutSeconds = 25)
    $native = $Owned['Native']
    $until = [DateTimeOffset]::UtcNow.AddSeconds($TimeoutSeconds)
    while ([DateTimeOffset]::UtcNow -lt $until) {
        Assert-ProcessIdentity $native
        $null = Pump-Pipes $Owned $Manifest ([DateTimeOffset]::UtcNow.AddMilliseconds(25))
        if (Test-Path -LiteralPath $Plan.Ready -PathType Leaf) {
            $item = Get-Item -LiteralPath $Plan.Ready
            if ($item.Length -lt 1 -or $item.Length -gt 16384) { throw 'Ready record length invalid' }
            $ready = ConvertFrom-Json -InputObject ([System.IO.File]::ReadAllText($Plan.Ready,[System.Text.UTF8Encoding]::new($false,$true))) -AsHashtable
            Assert-ExactKeys $ready @('schema','protocolVersion','host','port','processId','sessionToken','sessionDir','saveDir')
            foreach ($field in @('protocolVersion','port','processId')) { if ($ready[$field] -is [bool] -or $ready[$field] -isnot [ValueType] -or [double]$ready[$field] -lt 0 -or [math]::Floor([double]$ready[$field]) -ne [double]$ready[$field]) { throw 'Ready numeric field is invalid' } }
            foreach ($field in @('host','sessionToken','sessionDir','saveDir')) { if ($ready[$field] -isnot [string]) { throw 'Ready string field is invalid' } }
            $tokenValue = [string]$Token
            if ($ready.schema -cne 'wuxian.godot-v1.ready/v1' -or $ready.protocolVersion -is [bool] -or [int]$ready.protocolVersion -ne 1 -or $ready.host -cne '127.0.0.1' -or $ready.sessionToken -cne $tokenValue -or $ready.sessionDir.Replace('\','/') -cne $Plan.Session.Replace('\','/') -or $ready.saveDir.Replace('\','/') -cne $Plan.Saves.Replace('\','/') -or $ready.port -is [bool] -or [int]$ready.port -lt 1 -or [int]$ready.port -gt 65535 -or [int64]$ready.processId -ne $native.Process.Id) { throw 'Ready record owner mismatch' }
            $listeners = @(Get-NetTCPConnection -State Listen -LocalPort ([int]$ready.port) -ErrorAction Stop)
            if ($listeners.Count -ne 1 -or [int]$listeners[0].OwningProcess -ne $native.Process.Id) { throw 'Loopback listener owner mismatch' }
            return $ready
        }
        Start-Sleep -Milliseconds 25
    }
    throw 'Native ready record timed out'
}

function Pump-Pipes {
    param([hashtable]$Owned,[object]$Manifest,[DateTimeOffset]$Deadline)
    $rawPipeBytes = [int]$Manifest.limits.rawPipeBytes
    if ($null -eq $script:PipeStates) {
        $script:PipeStates = [System.Collections.Generic.List[object]]::new()
    }
    foreach ($record in $Owned.Values) {
        if (-not $record.Started) { continue }
        foreach ($kind in @('stdout','stderr')) {
            if (@($script:PipeStates | Where-Object { $_.Owner -eq $record.Kind -and $_.Kind -eq $kind }).Count -gt 0) { continue }
            $stream = if ($kind -eq 'stdout') { $record.Process.StandardOutput.BaseStream } else { $record.Process.StandardError.BaseStream }
            $script:PipeStates.Add([pscustomobject]@{ Owner=$record.Kind; Kind=$kind; Stream=$stream; Buffer=[byte[]]::new(16384); Task=$null; Eof=$false; Overflow=$false; Fault=$false; Bytes=0L; Memory=[System.IO.MemoryStream]::new() })
        }
    }
    $streams = $script:PipeStates
    while (@($streams | Where-Object { -not $_.Eof -and -not $_.Fault }).Count -gt 0 -and [DateTimeOffset]::UtcNow -lt $Deadline) {
        foreach ($state in $streams) { if (-not $state.Eof -and -not $state.Fault -and $null -eq $state.Task) { $state.Task = $state.Stream.ReadAsync($state.Buffer,0,$state.Buffer.Length) } }
        $pending = @($streams | Where-Object { $null -ne $_.Task } | ForEach-Object { $_.Task })
        if ($pending.Count -eq 0) { break }
        $any = [System.Threading.Tasks.Task]::WhenAny([System.Threading.Tasks.Task[]]$pending)
        if (-not $any.Wait(25)) { continue }
        $winner = $any.Result
        foreach ($state in $streams) {
            if ($state.Task -ne $winner) { continue }
            try {
                $count = [int]$state.Task.GetAwaiter().GetResult()
                if ($count -eq 0) { $state.Eof = $true }
                else {
                    $state.Bytes += $count
                    $room = [math]::Max(0,$rawPipeBytes - [int]$state.Memory.Length)
                    $keep = [math]::Min($room,$count)
                    if ($keep -gt 0) { $state.Memory.Write($state.Buffer,0,$keep) }
                    if ($keep -lt $count) { $state.Overflow = $true }
                }
            } catch { $state.Fault = $true }
            $state.Task = $null
        }
    }
    foreach ($record in $Owned.Values) { $record.PipeState = @($streams | Where-Object { $_.Owner -eq $record.Kind }) }
    return @($streams)
}

function Assert-Drained {
    param([hashtable]$Owned,[int]$rawPipeBytes)
    foreach ($record in $Owned.Values) {
        foreach ($pipe in $record.PipeState) {
            if (-not $pipe.Eof -or $pipe.Overflow -or $pipe.Fault -or $pipe.Bytes -gt $rawPipeBytes) { throw 'Raw pipe did not drain cleanly' }
        }
    }
    return $true
}

function Wait-OwnedClosure {
    param([hashtable]$Owned,[object]$Manifest)
    $natural = [DateTimeOffset]::UtcNow.AddSeconds([double]$Manifest.limits.naturalSeconds)
    $null = Pump-Pipes $Owned $Manifest $natural
    $waited = @{}
    foreach ($record in $Owned.Values) {
        $remaining = [math]::Max(0,($natural - [DateTimeOffset]::UtcNow).TotalMilliseconds)
        $waited[$record.Kind] = $record.Process.WaitForExit([int]$remaining)
    }
    $drainDeadline = [DateTimeOffset]::UtcNow.AddSeconds([double]$Manifest.limits.drainSeconds)
    $null = Pump-Pipes $Owned $Manifest $drainDeadline
    foreach ($record in $Owned.Values) { if (-not $waited[$record.Kind]) { $remaining = [math]::Max(0,($drainDeadline - [DateTimeOffset]::UtcNow).TotalMilliseconds); $waited[$record.Kind] = $record.Process.WaitForExit([int]$remaining) } }
    $drained = $true
    try { $null = Assert-Drained $Owned ([int]$Manifest.limits.rawPipeBytes) } catch { $drained = $false }
    $codes = @{}
    foreach ($record in $Owned.Values) {
        if (-not $waited[$record.Kind] -or -not $record.Process.HasExited) { throw 'Owned child exit is unresolved' }
        $record.ExitCode = $record.Process.ExitCode
        $codes[$record.Kind] = $record.ExitCode
    }
    $disposeDeadline = [DateTimeOffset]::UtcNow.AddSeconds([double]$Manifest.limits.disposeSeconds)
    foreach ($record in $Owned.Values) {
        if ([DateTimeOffset]::UtcNow -ge $disposeDeadline) { throw 'Owned process disposal window expired' }
        $record.Process.Dispose()
    }
    if (-not $drained -or @($codes.Values | Where-Object { $_ -ne 0 }).Count -ne 0) { throw 'Owned child closure failed' }
    return [pscustomobject]@{ ExitCodes=$codes; Drained=$drained; Eof=$true }
}

function Invoke-OwnedFallback {
    param([object]$Manifest,[object]$Plan,[string]$ReceiptPath,[string]$ReceiptHash,[hashtable]$Owned)
    Assert-RunAuthority $Manifest $Plan $ReceiptPath $ReceiptHash -ClosureOnly
    if (-not $script:RunReceipt.fallbackPolicy.approved) { return $false }
    foreach ($record in $Owned.Values) {
        if (-not $record.Started -or $record.Process.HasExited) { continue }
        Assert-ProcessIdentity $record
        if ($script:RunReceipt.fallbackPolicy.allowedKinds -ccontains $record.Kind) { $record.Process.Kill() }
    }
    $deadline = [DateTimeOffset]::UtcNow.AddSeconds([double]$Manifest.limits.drainSeconds)
    $null = Pump-Pipes $Owned $Manifest $deadline
    foreach ($record in $Owned.Values) {
        $remaining = [math]::Max(0,($deadline - [DateTimeOffset]::UtcNow).TotalMilliseconds)
        if ($record.Started -and -not $record.Process.WaitForExit([int]$remaining)) { throw 'Fallback child exit is unresolved' }
    }
    $drained = $true
    try { $null = Assert-Drained $Owned ([int]$Manifest.limits.rawPipeBytes) } catch { $drained = $false }
    foreach ($record in $Owned.Values) { if ($record.Started) { $record.ExitCode = $record.Process.ExitCode }; $record.Process.Dispose() }
    if (-not $drained) { throw 'Fallback pipes did not drain cleanly' }
    return $true
}

function Invoke-OwnedRun {
    param([object]$Manifest,[object]$Plan,[string]$ManifestDigest,[string]$ManifestFile,[string]$ReceiptPath,[string]$ReceiptHash)
    New-OwnedNamespace $Plan
    Materialize-Seed $Manifest $Plan
    $token = '{0}{1}' -f [guid]::NewGuid().ToString('N'),[guid]::NewGuid().ToString('N')
    $environment = @{
        APPDATA=$Plan.Roaming; LOCALAPPDATA=$Plan.Local; TEMP=$Plan.Temp; TMP=$Plan.Temp
        WX_FOUNDRY_OBS_MANIFEST=$ManifestFile; WX_FOUNDRY_OBS_MANIFEST_SHA256=$ManifestDigest
    }
    $owned = @{}
    try {
        $native = Start-OwnedProcess 'Native' $Manifest.artifacts.native.path (Get-NativeArguments $Plan.Session $Plan.Saves $token) $Plan.ProjectRoot $environment ([ref]$owned)
        $null = Assert-ReadyOwner $owned $Plan $Manifest $token
        $godot = Start-OwnedProcess 'Godot' $Manifest.artifacts.godot.path (Get-GodotArguments $Manifest $token) (Join-Path $Plan.ProjectRoot 'apps/godot_playable') $environment ([ref]$owned)
        $closure = Wait-OwnedClosure $owned $Manifest
        return [pscustomobject]@{ RunId=$Plan.RunId; Outcome='CLOSED'; ExitCodes=$closure.ExitCodes; RawPipesDrained=$closure.Drained; ReadyOwnerPid=$native.Process.Id; GodotPid=$godot.Process.Id }
    } catch {
        $null = Invoke-OwnedFallback $Manifest $Plan $ReceiptPath $ReceiptHash $owned
        throw
    }
}

function Invoke-Observation {
    param([string]$Mode,[string]$ManifestPath,[string]$ManifestSha256,[string]$ReceiptPath,[string]$ReceiptSha256)
    if ($Mode -eq 'Plan') {
        if ([string]::IsNullOrWhiteSpace($ManifestPath)) { return [pscustomobject]@{ ReadyForRun=$false; Reason='A MAIN-qualified manifest and receipt are required'; Mode='Plan' } }
        $manifest = Read-Manifest $ManifestPath $ManifestSha256
        $plan = Resolve-Plan $manifest
        return $plan
    }
    if ([string]::IsNullOrWhiteSpace($ManifestPath) -or [string]::IsNullOrWhiteSpace($ReceiptPath)) { throw 'Run requires external manifest and receipt' }
    $manifest = Read-Manifest $ManifestPath $ManifestSha256
    $plan = Resolve-Plan $manifest
    Assert-RunAuthority $manifest $plan $ReceiptPath $ReceiptSha256
    Assert-RunPins $manifest $plan
    return Invoke-OwnedRun $manifest $plan $ManifestSha256 $ManifestPath $ReceiptPath $ReceiptSha256
}

try {
    $result = Invoke-Observation $Mode $ManifestPath $ManifestSha256 $ReceiptPath $ReceiptSha256
    $result | ConvertTo-Json -Depth 20
} catch {
    [Console]::Error.WriteLine($_.Exception.Message)
    exit 2
}
