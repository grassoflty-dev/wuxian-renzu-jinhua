[CmdletBinding()]
param(
    [ValidateSet('Plan','Run')][string]$Mode = 'Plan',
    [string]$ManifestPath,
    [string]$ManifestSha256,
    [string]$ReceiptPath,
    [string]$ReceiptSha256,
    [string]$QualifiedManifestSha256,
    [string]$LeaseId
)
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$script:OrdinaryRoot = 'C:\Users\GWL\Desktop\wuxian-renzu-jinhua'
$script:PrivateBase = 'C:\Users\GWL\.codex\visualizations\2026\10\21\01a147f2-ef43-7cd3-9785-e0263fd4e60e\GH-FIRST-PLAYABLE-20261021-001\r1'
$GH_INTERFACE_JSON = @'
{"schema":"wuxian.gh-first-playable.interface/v2","manifestSchema":"wuxian.gh-first-playable.manifest/v2","phaseConfigSchema":"wuxian.gh-first-playable.phase-config/v1","evidenceSchema":"wuxian.gh-first-playable.evidence/v1","driver":"operator","ordinaryCheckout":"C:/Users/GWL/Desktop/wuxian-renzu-jinhua","phaseOrder":["new","continue"],"mainScene":"res://scenes/main.tscn","adapter":"res://tests/campaign/grey_hive_first_playable.gd","sessionRelative":"session","readyRelative":"session/run/bridge-ready.json","saveRelative":"session/saves/formal-save-v6.json","rpcArguments":["--v1-ready","--v1-token"],"environment":["WUXIAN_GH_CONFIG","WUXIAN_GH_CONFIG_SHA256","WUXIAN_GH_PHASE","WUXIAN_GH_EXPECTED_USER_DIR"],"runRequires":["runtime","sourceBaseline","resourcePurposes","profileBinding","operator","ownedFallback"],"requiredEvents":["hive_power","hive_lockdown","hive_extraction"],"baizhiChoices":["taken","left","unresolved"],"route":["rs_core_room","gh_entry_maintenance","gh_power_room","gh_gate_a","gh_central_shaft","gh_lockdown","gh_bio_isolation","gh_gate_b","gh_deep_decon","gh_sentinel_arena","gh_beacon","gh_exit"],"returnScene":"rs_core_room","returnGate":"gh_extraction_return_to_rs","sentinelExit":"gh_sentinel_arena_to_beacon","captureBinding":["snapshot","scene","current_key","scene_hash","camera","viewport_rect"],"manifestKeys":["schema","runId","driver","authority","sourceHead","source","artifacts","resources","optionalAssets","namespace","lease","operator","limits","acceptance","unresolved","deadlineUtc"],"requiredSourcePins":["apps/godot_playable/scripts/ui/v1_main.gd","apps/godot_playable/scripts/ui/panels.gd","apps/godot_playable/scripts/world/renderer.gd","apps/godot_playable/tests/campaign/grey_hive_first_playable.gd","tools/grey-hive-first-playable/run-owned.ps1","tools/grey-hive-first-playable/contract.test.mjs"],"sourceKeys":["checkoutRoot","head","indexEmpty","indexSha256","indexReference","protectedStateReference","pins"],"artifactKeys":["path","bytes","sha256","provenance","version"],"authorityRowKeys":["state","reference"],"resourceKeys":["path","bytes","sha256","presence","required","consumer","readPhase","purposeReference","qualificationState","distinction"],"optionalAssetKeys":["path","presence","bytes","sha256","purposeReference","sourceMapSha256"],"namespaceKeys":["base","runRoot"],"leaseKeys":["id","owner","firstUtc","reserveUtc","hardUtc","maxPairs","fallbackScope"],"operatorKeys":["reference","available","baizhiChoice"],"limitsKeys":["startupSeconds","phaseSeconds","closeSeconds","drainSeconds","totalSeconds","stdoutBytes","stderrBytes","captureCap","captureList"],"captureKeys":["id","phase","sceneId"],"acceptanceKeys":["route","requiredEvents","baizhiChoice","saveFields","comparisonFields","evidence"],"evidenceRequirementKeys":["phase","kind","outcome"],"phaseConfigKeys":["schema","phase","manifestSha256","manifest","sessionDir","saveDir","expectedUserDir","sessionTokenSha256"],"eventKeys":["schema","runId","phase","sequence","utc","monotonicMs","kind","outcome","manifestSha256","sourceBaselineReference","payload"],"eventTransport":"phase-output-file","eventFileName":"events.ndjson","maxConfigBytes":262144,"maxEventLineBytes":1048576,"maxEventFileBytes":134217728,"maxEventCount":50000,"maxCaptureCount":64,"terminalKind":"session_close","terminalOutcome":"CONFIRMED","terminalPayloadKeys":["authenticated","association","closing","connectionGeneration"],"acceptedKind":"run_complete","acceptedOutcome":"ACCEPTED"}
'@
# Compiled only inside an authorized Run, after all pre-start qualifications.
$script:OwnedRuntimeSource = @'
using System;
using System.ComponentModel;
using System.IO;
using System.Runtime.InteropServices;
using System.Text;
using System.Threading;
using System.Threading.Tasks;

public sealed class WxBoundedPipe
{
    private readonly Stream stream;
    private readonly long limit;
    private readonly MemoryStream raw = new MemoryStream();
    private long rawPipeBytes;
    private volatile bool eof;
    private volatile bool overflow;
    private string fault;
    public Task Worker { get; private set; }
    public long RawPipeBytes { get { return Interlocked.Read(ref rawPipeBytes); } }
    public bool Eof { get { return eof; } }
    public bool Overflow { get { return overflow; } }
    public string Fault { get { return fault; } }
    public byte[] Raw { get { return raw.ToArray(); } }
    public WxBoundedPipe(Stream input, long maximum) { stream = input; limit = maximum; }
    public void Start()
    {
        Worker = Task.Run(async () =>
        {
            byte[] buffer = new byte[8192];
            try
            {
                while (true)
                {
                    int count = await stream.ReadAsync(buffer, 0, buffer.Length).ConfigureAwait(false);
                    if (count == 0) { eof = true; break; }
                    long total = Interlocked.Add(ref rawPipeBytes, count);
                    long room = Math.Max(0, limit - (raw.Length));
                    int keep = (int)Math.Min(room, count);
                    if (keep > 0) raw.Write(buffer, 0, keep);
                    if (total > limit) overflow = true;
                }
            }
            catch (Exception ex) { fault = ex.GetType().FullName; }
        });
    }
    public bool Wait(int milliseconds) { return Worker != null && Worker.Wait(milliseconds); }
    [DllImport("kernel32.dll", CharSet=CharSet.Unicode, SetLastError=true, EntryPoint="CreateDirectoryW")]
    private static extern bool CreateDirectoryNative(string path, IntPtr securityAttributes);
    public static void CreateDirectoryNew(string path)
    {
        if (!CreateDirectoryNative(path, IntPtr.Zero))
            throw new Win32Exception(Marshal.GetLastWin32Error(), "CreateDirectoryNew failed");
    }
    [DllImport("shell32.dll", SetLastError=true)]
    private static extern IntPtr CommandLineToArgvW([MarshalAs(UnmanagedType.LPWStr)] string commandLine, out int count);
    [DllImport("kernel32.dll")]
    private static extern IntPtr LocalFree(IntPtr memory);
    public static string[] SplitCommandLine(string commandLine)
    {
        int count;
        IntPtr ptr = CommandLineToArgvW(commandLine, out count);
        if (ptr == IntPtr.Zero) throw new Win32Exception(Marshal.GetLastWin32Error());
        try
        {
            string[] values = new string[count];
            for (int i = 0; i < count; i++)
                values[i] = Marshal.PtrToStringUni(Marshal.ReadIntPtr(ptr, i * IntPtr.Size));
            return values;
        }
        finally { LocalFree(ptr); }
    }
}
'@
function Stop-Run {
    param([int]$Code, [string]$Message)
    throw [System.InvalidOperationException]::new("WXSTATUS:${Code}:$Message")
}
function Get-Interface {
    $value = ConvertFrom-Json -InputObject $GH_INTERFACE_JSON -AsHashtable -Depth 16
    if ($value.schema -ne 'wuxian.gh-first-playable.interface/v2' -or
        $value.manifestSchema -ne 'wuxian.gh-first-playable.manifest/v2' -or
        $value.phaseConfigSchema -ne 'wuxian.gh-first-playable.phase-config/v1' -or
        $value.evidenceSchema -ne 'wuxian.gh-first-playable.evidence/v1' -or
        $value.driver -ne 'operator' -or $value.phaseOrder.Count -ne 2 -or
        $value.eventTransport -ne 'phase-output-file' -or
        $value.eventFileName -ne 'events.ndjson' -or
        $value.terminalKind -ne 'session_close' -or $value.terminalOutcome -ne 'CONFIRMED' -or
        ($value.terminalPayloadKeys -join ',') -cne 'authenticated,association,closing,connectionGeneration' -or
        $value.acceptedKind -ne 'run_complete' -or $value.acceptedOutcome -ne 'ACCEPTED') {
        Stop-Run 42 'fixed interface literal is invalid'
    }
    return $value
}
function Assert-ExactKeys { param([object]$Value, [string[]]$Keys, [string]$At)
    if ($null -eq $Value -or $Value -isnot [System.Collections.IDictionary]) {
        Stop-Run 64 "$At must be an object"
    }
    $actual = @($Value.Keys | ForEach-Object { [string]$_ })
    if ($actual.Count -ne $Keys.Count) { Stop-Run 64 "$At has an unexpected key set" }
    foreach ($key in $Keys) {
        if (-not ($actual -ccontains $key)) { Stop-Run 64 "$At is missing a required key" }
    }
    foreach ($key in $actual) {
        if (-not ($Keys -ccontains $key)) { Stop-Run 64 "$At contains an unsupported key" }
    }
}
function Assert-String { param([object]$Value, [string]$At, [int]$Minimum = 1, [int]$Maximum = 4096)
    if ($Value -isnot [string] -or $Value.Length -lt $Minimum -or $Value.Length -gt $Maximum -or
        $Value.Contains([char]0)) { Stop-Run 64 "$At is not a valid string" }
}
function Assert-Integer { param([object]$Value, [string]$At, [long]$Minimum = 0, [long]$Maximum = [long]::MaxValue)
    if ($Value -is [bool] -or $Value -isnot [ValueType]) { Stop-Run 64 "$At must be an integer" }
    try { $number = [decimal]$Value } catch { Stop-Run 64 "$At must be an integer" }
    if ($number -ne [decimal]::Truncate($number) -or $number -lt $Minimum -or $number -gt $Maximum) { Stop-Run 64 "$At is outside its integer bound" }
}
function Assert-Boolean { param([object]$Value, [string]$At); if ($Value -isnot [bool]) { Stop-Run 64 "$At must be Boolean" } }
function Assert-Sha256 { param([object]$Value, [string]$At); if ($Value -isnot [string] -or $Value -cnotmatch '^[a-f0-9]{64}$') { Stop-Run 64 "$At must be lowercase SHA256" } }
function Get-ByteSha256 {
    param([byte[]]$Bytes)
    $sha = [System.Security.Cryptography.SHA256]::Create()
    try { return ([Convert]::ToHexString($sha.ComputeHash($Bytes))).ToLowerInvariant() }
    finally { $sha.Dispose() }
}
function Get-FileSha256 {
    param([string]$Path)
    $stream = [System.IO.File]::Open($Path, [System.IO.FileMode]::Open, [System.IO.FileAccess]::Read, [System.IO.FileShare]::Read)
    try {
        $sha = [System.Security.Cryptography.SHA256]::Create()
        try { return ([Convert]::ToHexString($sha.ComputeHash($stream))).ToLowerInvariant() }
        finally { $sha.Dispose() }
    } finally { $stream.Dispose() }
}
function Test-DuplicateJsonKeys {
    param([System.Text.Json.JsonElement]$Element)
    if ($Element.ValueKind -eq [System.Text.Json.JsonValueKind]::Object) {
        $seen = [System.Collections.Generic.HashSet[string]]::new([System.StringComparer]::Ordinal)
        foreach ($property in $Element.EnumerateObject()) {
            if (-not $seen.Add($property.Name)) { Stop-Run 64 'JSON contains a duplicate key' }
            Test-DuplicateJsonKeys $property.Value
        }
    } elseif ($Element.ValueKind -eq [System.Text.Json.JsonValueKind]::Array) {
        foreach ($item in $Element.EnumerateArray()) { Test-DuplicateJsonKeys $item }
    } elseif ($Element.ValueKind -eq [System.Text.Json.JsonValueKind]::Number) {
        $number = 0.0
        if (-not $Element.TryGetDouble([ref]$number) -or [double]::IsNaN($number) -or [double]::IsInfinity($number)) {
            Stop-Run 64 'JSON contains a non-finite number'
        }
    }
}
function Read-StrictJson {
    param([string]$Path, [int]$MaximumBytes)
    if (-not [System.IO.Path]::IsPathRooted($Path) -or -not [System.IO.File]::Exists($Path)) {
        Stop-Run 64 'JSON input path is absent or not absolute'
    }
    $info = [System.IO.FileInfo]::new($Path)
    if ($info.Length -le 0 -or $info.Length -gt $MaximumBytes -or ($info.Attributes -band [System.IO.FileAttributes]::ReparsePoint) -ne 0) { Stop-Run 64 'JSON input size or path identity is outside its bound' }
    $bytes = [System.IO.File]::ReadAllBytes($Path)
    $encoding = [System.Text.UTF8Encoding]::new($false, $true)
    try { $text = $encoding.GetString($bytes) }
    catch { Stop-Run 64 'JSON input is not strict UTF-8' }
    if ($text.Length -gt 0 -and $text[0] -eq [char]0xFEFF) { Stop-Run 64 'JSON BOM is not accepted' }
    try {
        $options = [System.Text.Json.JsonDocumentOptions]::new()
        $document = [System.Text.Json.JsonDocument]::Parse($text, $options)
        try { Test-DuplicateJsonKeys $document.RootElement }
        finally { $document.Dispose() }
        $value = ConvertFrom-Json -InputObject $text -AsHashtable -Depth 100 -DateKind String
    } catch { Stop-Run 64 'JSON input is malformed or unsupported' }
    return [pscustomobject]@{ Text = $text; Bytes = $bytes; Value = $value }
}
function Assert-Pin { param([object]$Pin, [string]$At, [switch]$Source)
    Assert-ExactKeys $Pin @('path','bytes','sha256') $At
    Assert-String $Pin.path "$At.path" 1 512
    Assert-Integer $Pin.bytes "$At.bytes" 1 1099511627776
    Assert-Sha256 $Pin.sha256 "$At.sha256"
    if ($Source) {
        if ($Pin.path.StartsWith('/') -or $Pin.path.Contains('\') -or $Pin.path.Contains(':') -or
            @($Pin.path.Split('/')) -contains '..' -or $Pin.path.StartsWith('./')) {
            Stop-Run 64 "$At.path is not a canonical repository-relative path"
        }
    } else {
        Assert-String $Pin.provenance "$At.provenance" 1 1024
    }
}
function Assert-ResourceRow { param([object]$Row, [string]$At)
    Assert-ExactKeys $Row (Get-Interface).resourceKeys $At
    Assert-String $Row.path "$At.path" 1 1024
    if (-not $Row.path.StartsWith('res://', [System.StringComparison]::Ordinal) -or $Row.path.Contains('..') -or $Row.path.Contains('\')) {
        Stop-Run 64 "$At.path is not a canonical resource path"
    }
    if ($Row.presence -cnotin @('present','absent')) { Stop-Run 64 "$At.presence is invalid" }
    Assert-Boolean $Row.required "$At.required"
    Assert-String $Row.consumer "$At.consumer" 1 512
    Assert-String $Row.readPhase "$At.readPhase" 1 128
    if ($Row.presence -eq 'present') {
        Assert-Integer $Row.bytes "$At.bytes" 1 1099511627776
        Assert-Sha256 $Row.sha256 "$At.sha256"
        Assert-String $Row.purposeReference "$At.purposeReference" 1 1024
        if ($Row.qualificationState -cnotin @('QUALIFIED','UNRESOLVED')) { Stop-Run 64 "$At.qualificationState is invalid" }
    } else {
        Assert-Integer $Row.bytes "$At.bytes" 0 0
        if ($null -ne $Row.sha256 -or $null -ne $Row.purposeReference) { Stop-Run 64 "$At absent resource must have null hashes and purpose" }
        if ($Row.qualificationState -cnotin @('QUALIFIED','UNRESOLVED')) { Stop-Run 64 "$At.qualificationState is invalid" }
    }
    if ($Row.distinction -cnotin @('display','integrity','incidental')) { Stop-Run 64 "$At.distinction is invalid" }
}
function Assert-Manifest { param([object]$Manifest)
    $i = Get-Interface
    Assert-ExactKeys $Manifest $i.manifestKeys 'manifest'
    if ($Manifest.schema -cne $i.manifestSchema -or $Manifest.driver -cne $i.driver) { Stop-Run 64 'manifest schema or driver is unsupported' }
    Assert-ExactKeys $Manifest.authority @($i.runRequires) 'manifest.authority'
    foreach ($name in $i.runRequires) {
        $authority = $Manifest.authority[$name]
        Assert-ExactKeys $authority @('state','reference') "authority.$name"
        if ($authority.state -cnotin @('QUALIFIED','UNRESOLVED')) { Stop-Run 64 "authority.$name.state is invalid" }
        Assert-String $authority.reference "authority.$name.reference" 1 1024
    }
    $guid = [guid]::Empty
    if ($Manifest.runId -isnot [string] -or -not [guid]::TryParseExact($Manifest.runId, 'D', [ref]$guid) -or $guid.ToString('D') -cne $Manifest.runId) {
        Stop-Run 64 'runId is not a canonical lowercase UUID'
    }
    Assert-String $Manifest.sourceHead 'sourceHead' 40 40
    if ($Manifest.sourceHead -cnotmatch '^[a-f0-9]{40}$') { Stop-Run 64 'sourceHead is malformed' }
    Assert-ExactKeys $Manifest.source $i.sourceKeys 'source'
    Assert-String $Manifest.source.checkoutRoot 'source.checkoutRoot' 1 1024
    if ([System.IO.Path]::GetFullPath($Manifest.source.checkoutRoot).TrimEnd('\') -ine $script:OrdinaryRoot) { Stop-Run 64 'source checkout is not the qualified ordinary checkout' }
    if ($Manifest.source.head -cne $Manifest.sourceHead) { Stop-Run 64 'source HEAD bindings disagree' }
    Assert-Boolean $Manifest.source.indexEmpty 'source.indexEmpty'
    if (-not $Manifest.source.indexEmpty) { Stop-Run 64 'Run requires the externally qualified empty index' }
    Assert-Sha256 $Manifest.source.indexSha256 'source.indexSha256'
    Assert-String $Manifest.source.indexReference 'source.indexReference' 1 1024
    Assert-String $Manifest.source.protectedStateReference 'source.protectedStateReference' 1 1024
    if ($Manifest.source.pins -isnot [array] -or $Manifest.source.pins.Count -lt 6) { Stop-Run 64 'source pins are incomplete' }
    $pinPaths = [System.Collections.Generic.HashSet[string]]::new([System.StringComparer]::Ordinal)
    foreach ($pin in $Manifest.source.pins) {
        Assert-Pin $pin 'source pin' -Source
        if (-not $pinPaths.Add($pin.path)) { Stop-Run 64 'source pins contain duplicate paths' }
    }
    foreach ($path in $i.requiredSourcePins) { if (-not $pinPaths.Contains($path)) { Stop-Run 64 'source pins omit a runtime-consumed harness or production input' } }
    Assert-ExactKeys $Manifest.artifacts @('native','godot') 'artifacts'
    foreach ($name in @('native','godot')) {
        $artifact = $Manifest.artifacts[$name]
        Assert-ExactKeys $artifact $i.artifactKeys "artifacts.$name"
        Assert-String $artifact.path "artifacts.$name.path" 1 4096
        if (-not [System.IO.Path]::IsPathRooted($artifact.path)) { Stop-Run 64 "artifacts.$name.path must be absolute" }
        Assert-Integer $artifact.bytes "artifacts.$name.bytes" 1 1099511627776
        Assert-Sha256 $artifact.sha256 "artifacts.$name.sha256"
        Assert-String $artifact.provenance "artifacts.$name.provenance" 1 1024
        if ($null -ne $artifact.version) { Assert-String $artifact.version "artifacts.$name.version" 1 256 }
    }
    if ($Manifest.resources -isnot [array] -or $Manifest.resources.Count -eq 0) { Stop-Run 64 'resources must be a variable-length nonempty array' }
    $resourcePaths = [System.Collections.Generic.HashSet[string]]::new([System.StringComparer]::Ordinal)
    foreach ($row in $Manifest.resources) {
        Assert-ResourceRow $row 'resource'
        if (-not $resourcePaths.Add($row.path)) { Stop-Run 64 'resources contain duplicate paths' }
    }
    if ($Manifest.optionalAssets -isnot [array]) { Stop-Run 64 'optionalAssets must be an array' }
    $optionalPaths = [System.Collections.Generic.HashSet[string]]::new([System.StringComparer]::Ordinal)
    foreach ($row in $Manifest.optionalAssets) {
        Assert-ExactKeys $row $i.optionalAssetKeys 'optional asset'
        Assert-String $row.path 'optional asset.path' 1 1024
        if (-not $row.path.StartsWith('res://',[System.StringComparison]::Ordinal) -or $row.path.Contains('..') -or $row.path.Contains('\')) { Stop-Run 64 'optional asset path is invalid' }
        if (-not $optionalPaths.Add($row.path)) { Stop-Run 64 'optionalAssets contains duplicate paths' }
        Assert-Sha256 $row.sourceMapSha256 'optional asset sourceMapSha256'
        if ($row.presence -eq 'present') {
            Assert-Integer $row.bytes 'optional asset bytes' 1 1099511627776; Assert-Sha256 $row.sha256 'optional asset sha256'
            Assert-String $row.purposeReference 'optional asset purposeReference' 1 1024
            if (-not $resourcePaths.Contains($row.path)) { Stop-Run 64 'present optional asset has no matching resource row' }
            $bound = @($Manifest.resources | Where-Object { $_.path -ceq $row.path })
            if ($bound.Count -ne 1 -or $bound[0].presence -cne 'present' -or $bound[0].bytes -ne $row.bytes -or $bound[0].sha256 -cne $row.sha256 -or $bound[0].purposeReference -cne $row.purposeReference) { Stop-Run 64 'optional asset and qualified resource identities disagree' }
        } elseif ($row.presence -eq 'absent') {
            Assert-Integer $row.bytes 'optional asset bytes' 0 0
            if ($null -ne $row.sha256 -or $null -ne $row.purposeReference) { Stop-Run 64 'absent optional asset must not carry a binary identity or purpose' }
            if ($resourcePaths.Contains($row.path)) { Stop-Run 64 'absent optional asset conflicts with a resource row' }
        } else { Stop-Run 64 'optional asset presence is invalid' }
    }
    Assert-ExactKeys $Manifest.namespace $i.namespaceKeys 'namespace'
    Assert-String $Manifest.namespace.base 'namespace.base' 1 1024
    Assert-String $Manifest.namespace.runRoot 'namespace.runRoot' 1 2048
    Assert-ExactKeys $Manifest.lease $i.leaseKeys 'lease'
    Assert-String $Manifest.lease.id 'lease.id' 1 256; Assert-String $Manifest.lease.owner 'lease.owner' 1 256
    Assert-String $Manifest.lease.firstUtc 'lease.firstUtc' 20 40; Assert-String $Manifest.lease.reserveUtc 'lease.reserveUtc' 20 40
    Assert-String $Manifest.lease.hardUtc 'lease.hardUtc' 20 40; Assert-Integer $Manifest.lease.maxPairs 'lease.maxPairs' 2 2
    Assert-String $Manifest.lease.fallbackScope 'lease.fallbackScope' 1 1024
    Assert-String $Manifest.deadlineUtc 'deadlineUtc' 20 40
    Assert-ExactKeys $Manifest.operator $i.operatorKeys 'operator'
    Assert-String $Manifest.operator.reference 'operator.reference' 1 1024; Assert-Boolean $Manifest.operator.available 'operator.available'
    if ($Manifest.operator.baizhiChoice -cnotin $i.baizhiChoices) { Stop-Run 64 'operator branch is unsupported' }
    Assert-ExactKeys $Manifest.limits $i.limitsKeys 'limits'
    foreach ($field in @('startupSeconds','phaseSeconds','closeSeconds','drainSeconds','totalSeconds')) {
        $value = $Manifest.limits[$field]
        if ($value -is [bool] -or $value -isnot [ValueType] -or [double]::IsNaN([double]$value) -or [double]::IsInfinity([double]$value) -or [double]$value -le 0) { Stop-Run 64 "limits.$field must be finite and positive" }
    }
    foreach ($field in @('stdoutBytes','stderrBytes')) { Assert-Integer $Manifest.limits[$field] "limits.$field" 1 1048576 }
    Assert-Integer $Manifest.limits.captureCap 'limits.captureCap' 1 $i.maxCaptureCount
    if ($Manifest.limits.captureList -isnot [array] -or $Manifest.limits.captureList.Count -eq 0 -or $Manifest.limits.captureList.Count -gt $Manifest.limits.captureCap) { Stop-Run 64 'limits.captureList is outside its declared bound' }
    $captureSet = [System.Collections.Generic.HashSet[string]]::new([System.StringComparer]::Ordinal)
    foreach ($capture in $Manifest.limits.captureList) {
        Assert-ExactKeys $capture $i.captureKeys 'capture allocation'
        Assert-String $capture.id 'capture id' 1 64
        if ($capture.id -cnotmatch '^[A-Za-z0-9_.-]+$') { Stop-Run 64 'capture id is not a safe label' }
        if ($capture.phase -cnotin $i.phaseOrder -or $capture.sceneId -cnotin $i.route) { Stop-Run 64 'capture phase or scene is outside the route' }
        if (-not $captureSet.Add("$($capture.phase):$($capture.id)")) { Stop-Run 64 'capture allocation is duplicated' }
    }
    $expectedCaptures = @('new:new-rs:rs_core_room','new:gh-entry:gh_entry_maintenance',
        'new:sentinel-encounter:gh_sentinel_arena','new:gh-exit-extraction:gh_exit',
        'new:returned-rs-saved:rs_core_room','continue:continue-rs:rs_core_room')
    $actualCaptures = @($Manifest.limits.captureList | ForEach-Object { "$($_.phase):$($_.id):$($_.sceneId)" })
    if ($Manifest.limits.captureCap -ne 6 -or ($actualCaptures -join '|') -cne ($expectedCaptures -join '|')) { Stop-Run 64 'capture allocation must be the exact six milestones' }
    Assert-ExactKeys $Manifest.acceptance $i.acceptanceKeys 'acceptance'
    $expectedRoute = @($i.route)
    if ($Manifest.acceptance.route -isnot [array] -or $Manifest.acceptance.route.Count -ne $expectedRoute.Count) { Stop-Run 64 'acceptance route is incomplete' }
    for ($routeIndex=0; $routeIndex -lt $expectedRoute.Count; $routeIndex++) {
        if ($Manifest.acceptance.route[$routeIndex] -cne $expectedRoute[$routeIndex]) { Stop-Run 64 'acceptance route differs from the qualified route policy' }
    }
    $routeSet = [System.Collections.Generic.HashSet[string]]::new([System.StringComparer]::Ordinal)
    foreach ($scene in $Manifest.acceptance.route) { Assert-String $scene 'route scene' 1 128; if (-not $routeSet.Add($scene)) { Stop-Run 64 'route repeats a scene' } }
    if ($Manifest.acceptance.requiredEvents -isnot [array] -or (Compare-Object @($Manifest.acceptance.requiredEvents) @($i.requiredEvents) -CaseSensitive)) { Stop-Run 64 'required event set differs from the interface' }
    if ($Manifest.acceptance.baizhiChoice -cnotin $i.baizhiChoices -or $Manifest.acceptance.baizhiChoice -cne $Manifest.operator.baizhiChoice) { Stop-Run 64 'acceptance Bai Zhi choice must match the operator choice' }
    foreach ($field in @('saveFields','comparisonFields')) {
        if ($Manifest.acceptance[$field] -isnot [array] -or $Manifest.acceptance[$field].Count -eq 0) { Stop-Run 64 "acceptance.$field must be a nonempty array" }
        $fieldSet = [System.Collections.Generic.HashSet[string]]::new([System.StringComparer]::Ordinal)
        foreach ($value in $Manifest.acceptance[$field]) {
            Assert-String $value "acceptance.$field entry" 1 256
            if (-not $fieldSet.Add($value)) { Stop-Run 64 "acceptance.$field contains duplicates" }
        }
    }
    if ($Manifest.acceptance.evidence -isnot [array] -or $Manifest.acceptance.evidence.Count -eq 0) { Stop-Run 64 'acceptance.evidence must be a nonempty array' }
    foreach ($item in $Manifest.acceptance.evidence) {
        Assert-ExactKeys $item $i.evidenceRequirementKeys 'evidence requirement'
        if ($item.phase -cnotin $i.phaseOrder) { Stop-Run 64 'evidence phase is unsupported' }
        Assert-String $item.kind 'evidence kind' 1 128
        Assert-String $item.outcome 'evidence outcome' 1 128
    }
    if ($Manifest.unresolved -isnot [array]) { Stop-Run 64 'unresolved must be an explicit array' }
    foreach ($item in $Manifest.unresolved) {
        Assert-ExactKeys $item @('dependency','reason','reference') 'unresolved dependency'
        Assert-String $item.dependency 'unresolved dependency name' 1 256
        Assert-String $item.reason 'unresolved dependency reason' 1 2048
        Assert-String $item.reference 'unresolved dependency reference' 1 1024
    }
    Assert-OrdinaryNamespace $Manifest
}
function Assert-OrdinaryNamespace { param([object]$Manifest)
    $base = [System.IO.Path]::GetFullPath($Manifest.namespace.base).TrimEnd('\')
    $root = [System.IO.Path]::GetFullPath($Manifest.namespace.runRoot).TrimEnd('\')
    $expected = [System.IO.Path]::GetFullPath((Join-Path $script:PrivateBase $Manifest.runId)).TrimEnd('\')
    if (-not [System.IO.Path]::IsPathRooted($Manifest.namespace.base) -or -not [System.IO.Path]::IsPathRooted($Manifest.namespace.runRoot) -or $base -ine $script:PrivateBase -or $root -ine $expected -or
        $root -ine ([System.IO.Path]::GetFullPath((Join-Path $base $Manifest.runId)).TrimEnd('\'))) {
        Stop-Run 64 'namespace is not the exact private UUID descendant'
    }
    if ($Manifest.deadlineUtc -cne $Manifest.lease.hardUtc) { Stop-Run 64 'deadlineUtc and lease hard deadline disagree' }
    $first = [DateTimeOffset]::Parse($Manifest.lease.firstUtc, [Globalization.CultureInfo]::InvariantCulture, [Globalization.DateTimeStyles]::RoundtripKind)
    $reserve = [DateTimeOffset]::Parse($Manifest.lease.reserveUtc, [Globalization.CultureInfo]::InvariantCulture, [Globalization.DateTimeStyles]::RoundtripKind)
    $hard = [DateTimeOffset]::Parse($Manifest.lease.hardUtc, [Globalization.CultureInfo]::InvariantCulture, [Globalization.DateTimeStyles]::RoundtripKind)
    $stampPattern = '^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d{1,7})?Z$'
    if ($Manifest.lease.firstUtc -cnotmatch $stampPattern -or $Manifest.lease.reserveUtc -cnotmatch $stampPattern -or $Manifest.lease.hardUtc -cnotmatch $stampPattern -or $first.Offset -ne [TimeSpan]::Zero -or $reserve.Offset -ne [TimeSpan]::Zero -or $hard.Offset -ne [TimeSpan]::Zero -or $reserve -le $first -or $hard -le $reserve) { Stop-Run 64 'lease clock ordering or UTC zone is invalid' }
}
function Read-Manifest { param([string]$Path, [string]$ExpectedSha256)
    if ([string]::IsNullOrWhiteSpace($Path)) { Stop-Run 64 'ManifestPath is required' }
    if (-not [string]::IsNullOrWhiteSpace($ExpectedSha256)) { Assert-Sha256 $ExpectedSha256 'ManifestSha256' }
    $read = Read-StrictJson $Path 262144
    $sha = Get-ByteSha256 $read.Bytes
    if ($ExpectedSha256 -and $sha -cne $ExpectedSha256.ToLowerInvariant()) { Stop-Run 64 'manifest digest does not match' }
    Assert-Manifest $read.Value
    return [pscustomobject]@{ Value=$read.Value; Text=$read.Text; Bytes=$read.Bytes; Sha256=$sha; Path=[System.IO.Path]::GetFullPath($Path) }
}
function Get-Plan { param([string]$Path, [string]$ExpectedSha256)
    if ([string]::IsNullOrWhiteSpace($Path)) {
        return [ordered]@{ schema='wuxian.gh-first-playable.plan/v1'; mode='Plan'; runReady=$false; reason='manifest not supplied'; sideEffects=$false }
    }
    $manifest = Read-Manifest $Path $ExpectedSha256
    $unresolved = @($manifest.Value.unresolved).Count -gt 0 -or
        @($manifest.Value.authority.Keys | Where-Object { $manifest.Value.authority[$_].state -ne 'QUALIFIED' }).Count -gt 0 -or
        @($manifest.Value.resources | Where-Object { $_.presence -eq 'present' -and $_.qualificationState -ne 'QUALIFIED' }).Count -gt 0 -or @($manifest.Value.resources | Where-Object { $_.required -and $_.presence -ne 'present' }).Count -gt 0 -or
        -not $manifest.Value.operator.available -or $manifest.Value.operator.baizhiChoice -eq 'unresolved'
    $session = Join-Path $manifest.Value.namespace.runRoot 'session'
    $ready = Join-Path $manifest.Value.namespace.runRoot 'session/run/bridge-ready.json'
    $saves = Join-Path $manifest.Value.namespace.runRoot 'session/saves'
    $godotArgs = Get-GodotArguments $manifest.Value $ready '<TOKEN>' (Join-Path $manifest.Value.namespace.runRoot 'phases/new/outputs/godot.log')
    $nativeArgs = Get-NativeArguments $session $saves '<TOKEN>'
    $rpcArgs = Get-RpcArguments $ready '<TOKEN>'
    return [ordered]@{ schema='wuxian.gh-first-playable.plan/v1'; mode='Plan'; runId=$manifest.Value.runId;
        manifestSha256=$manifest.Sha256; manifestReady=(-not $unresolved); runReady=$false;
        reason='Plan never grants the external receipt, lease, resource or process authority needed for Run';
        nativeArguments=$nativeArgs; godotArguments=$godotArgs; rpcArguments=$rpcArgs; sideEffects=$false }
}
function Read-Receipt { param([string]$Path, [string]$ExpectedSha256)
    if ([string]::IsNullOrWhiteSpace($Path) -or [string]::IsNullOrWhiteSpace($ExpectedSha256)) { Stop-Run 64 'external receipt path and digest are required' }
    Assert-Sha256 $ExpectedSha256 'ReceiptSha256'
    $read = Read-StrictJson $Path 262144
    $sha = Get-ByteSha256 $read.Bytes
    if ($sha -cne $ExpectedSha256) { Stop-Run 64 'receipt digest does not match' }
    return [pscustomobject]@{ Value=$read.Value; Bytes=$read.Bytes; Sha256=$sha; Path=[System.IO.Path]::GetFullPath($Path) }
}
function Assert-RunAuthorization { param([object]$ManifestRecord)
    $interface = Get-Interface
    if ($interface.runRequires.Count -ne 6) { Stop-Run 64 'fixed authorization requirement set is invalid' }
    if ([string]::IsNullOrWhiteSpace($QualifiedManifestSha256) -or
        $QualifiedManifestSha256 -cne $ManifestRecord.Sha256 -or
        [string]::IsNullOrWhiteSpace($LeaseId)) { Stop-Run 64 'MAIN-qualified manifest digest and lease ID are required' }
    $script:ReceiptRecord = Read-Receipt $ReceiptPath $ReceiptSha256
    Assert-ReceiptBinding $script:ReceiptRecord.Value $ManifestRecord
    Assert-NotExpired $script:ReceiptRecord.Value $ManifestRecord.Value
    Assert-PurposeApproval $ManifestRecord.Value $script:ReceiptRecord.Value
}
function Assert-FallbackAuthorization {
    param([object]$State)
    if ($null -eq $State -or $null -eq $State.ManifestRecord -or $null -eq $script:ReceiptRecord) { Stop-Run 40 'retained Run authorization is unavailable for fallback' }
    $fresh = Read-Receipt $ReceiptPath $ReceiptSha256
    Assert-ReceiptBinding $fresh.Value $State.ManifestRecord
    if (-not $fresh.Value.fallback.allowed -or $fresh.Value.fallback.scope -cne $State.ManifestRecord.Value.lease.fallbackScope) { Stop-Run 40 'external receipt does not authorize this exact fallback' }
    $now = [DateTimeOffset]::UtcNow
    $expiry = [DateTimeOffset]::Parse($fresh.Value.expiresUtc,[Globalization.CultureInfo]::InvariantCulture,[Globalization.DateTimeStyles]::RoundtripKind)
    $hard = [DateTimeOffset]::Parse($State.ManifestRecord.Value.deadlineUtc,[Globalization.CultureInfo]::InvariantCulture,[Globalization.DateTimeStyles]::RoundtripKind)
    if ($expiry.Offset -ne [TimeSpan]::Zero -or $now -ge $expiry -or $now -ge $hard) { Stop-Run 40 'bounded fallback closure window has expired' }
    $script:ReceiptRecord = $fresh
}
function Assert-ReceiptBinding { param([object]$Receipt, [object]$ManifestRecord)
    Assert-ExactKeys $Receipt @('schema','runId','manifestSha256','leaseId','checkoutRoot','sourceHead','indexSha256','issuedUtc','expiresUtc','authority','resourcePurposes','fallback') 'receipt'
    if ($Receipt.schema -cne 'wuxian.gh-first-playable.receipt/v1' -or $Receipt.runId -cne $ManifestRecord.Value.runId -or
        $Receipt.manifestSha256 -cne $ManifestRecord.Sha256 -or $Receipt.leaseId -cne $LeaseId -or
        $Receipt.checkoutRoot -ine $ManifestRecord.Value.source.checkoutRoot -or $Receipt.sourceHead -cne $ManifestRecord.Value.sourceHead -or
        $Receipt.indexSha256 -cne $ManifestRecord.Value.source.indexSha256) { Stop-Run 64 'external receipt does not bind this exact run and source' }
    Assert-ExactKeys $Receipt.authority @((Get-Interface).runRequires) 'receipt.authority'
    foreach ($name in (Get-Interface).runRequires) {
        $row = $Receipt.authority[$name]
        Assert-ExactKeys $row @('state','reference') "receipt.authority.$name"
        if ($row.state -cne 'QUALIFIED' -or $ManifestRecord.Value.authority[$name].state -cne 'QUALIFIED' -or
            $row.reference -cne $ManifestRecord.Value.authority[$name].reference) { Stop-Run 64 "authority $name is not independently qualified" }
    }
    Assert-ExactKeys $Receipt.fallback @('allowed','scope','reference') 'receipt.fallback'
    Assert-Boolean $Receipt.fallback.allowed 'receipt.fallback.allowed'
    if (-not $Receipt.fallback.allowed -or $Receipt.fallback.scope -cne $ManifestRecord.Value.lease.fallbackScope) { Stop-Run 64 'owned fallback allocation is absent or mismatched' }
    Assert-String $Receipt.fallback.reference 'receipt.fallback.reference' 1 1024
    if ($Receipt.resourcePurposes -isnot [array]) { Stop-Run 64 'receipt resourcePurposes must be an array' }
    $seen = [System.Collections.Generic.HashSet[string]]::new([System.StringComparer]::Ordinal)
    $purposeByPath = [System.Collections.Generic.Dictionary[string,string]]::new([System.StringComparer]::Ordinal)
    foreach ($row in $Receipt.resourcePurposes) {
        Assert-ExactKeys $row @('path','purposeReference','state') 'receipt resource purpose'
        Assert-String $row.path 'receipt purpose path' 1 1024; Assert-String $row.purposeReference 'receipt purpose reference' 1 1024
        if ($row.state -cne 'QUALIFIED' -or -not $seen.Add($row.path)) { Stop-Run 64 'receipt contains unresolved or duplicate resource purpose' }
        $purposeByPath.Add($row.path,$row.purposeReference)
    }
    foreach ($resource in $ManifestRecord.Value.resources) {
        if ($resource.presence -eq 'present' -and (-not $purposeByPath.ContainsKey($resource.path) -or
            $purposeByPath[$resource.path] -cne $resource.purposeReference)) { Stop-Run 64 'receipt purpose rows do not exactly cover present resources' }
    }
    foreach ($path in $seen) {
        if (@($ManifestRecord.Value.resources | Where-Object { $_.presence -eq 'present' -and $_.path -ceq $path }).Count -ne 1) { Stop-Run 64 'receipt contains an unrelated resource purpose' }
    }
}
function Assert-NotExpired { param([object]$Receipt, [object]$Manifest)
    $now = [DateTimeOffset]::UtcNow
    $first = [DateTimeOffset]::Parse($Manifest.lease.firstUtc, [Globalization.CultureInfo]::InvariantCulture, [Globalization.DateTimeStyles]::RoundtripKind)
    $reserve = [DateTimeOffset]::Parse($Manifest.lease.reserveUtc, [Globalization.CultureInfo]::InvariantCulture, [Globalization.DateTimeStyles]::RoundtripKind)
    $hard = [DateTimeOffset]::Parse($Manifest.deadlineUtc, [Globalization.CultureInfo]::InvariantCulture, [Globalization.DateTimeStyles]::RoundtripKind)
    $expiry = [DateTimeOffset]::Parse($Receipt.expiresUtc, [Globalization.CultureInfo]::InvariantCulture, [Globalization.DateTimeStyles]::RoundtripKind)
    $issued = [DateTimeOffset]::Parse($Receipt.issuedUtc, [Globalization.CultureInfo]::InvariantCulture, [Globalization.DateTimeStyles]::RoundtripKind)
    $needed = [double]$Manifest.limits.closeSeconds + [double]$Manifest.limits.drainSeconds
    $stampPattern = '^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d{1,7})?Z$'
    if ($Receipt.issuedUtc -cnotmatch $stampPattern -or $Receipt.expiresUtc -cnotmatch $stampPattern -or $expiry.Offset -ne [TimeSpan]::Zero -or $issued.Offset -ne [TimeSpan]::Zero -or $issued -gt $now -or $expiry -le $issued -or $expiry -le $now -or
        $now -lt $first -or $now -ge $reserve -or $now.AddSeconds($needed) -ge $hard -or $expiry -lt $hard) {
        Stop-Run 64 'lease or receipt has insufficient remaining bounded time'
    }
}
function Assert-PurposeApproval { param([object]$Manifest, [object]$Receipt)
    $i = Get-Interface
    Assert-ExactKeys $Manifest.authority @($i.runRequires) 'manifest.authority'
    foreach ($name in $i.runRequires) {
        $row = $Manifest.authority[$name]
        Assert-ExactKeys $row @('state','reference') "manifest.authority.$name"
        if ($row.state -cne 'QUALIFIED') { Stop-Run 64 "authority $name remains unresolved" }
        Assert-String $row.reference "manifest.authority.$name.reference" 1 1024
    }
    if (@($Manifest.unresolved).Count -gt 0 -or -not $Manifest.operator.available -or $Manifest.operator.baizhiChoice -eq 'unresolved') {
        Stop-Run 64 'runtime dependency or operator branch remains unresolved'
    }
    $qualified = @{}
    foreach ($row in $Receipt.resourcePurposes) { $qualified[$row.path] = $row.purposeReference }
    foreach ($resource in $Manifest.resources) {
        if ($resource.presence -eq 'absent') { if ($resource.required) { Stop-Run 64 'required resource is absent' }; continue }
        if ($resource.qualificationState -cne 'QUALIFIED' -or -not $qualified.ContainsKey($resource.path) -or
            $qualified[$resource.path] -cne $resource.purposeReference) { Stop-Run 64 'resource purpose is not externally qualified' }
    }
}
function Get-HeadOid { param([string]$CheckoutRoot)
    $git = Join-Path $CheckoutRoot '.git'
    if (-not [System.IO.Directory]::Exists($git)) { Stop-Run 64 'unsupported Git metadata layout' }
    $headPath = Join-Path $git 'HEAD'
    $head = [System.IO.File]::ReadAllText($headPath, [System.Text.Encoding]::ASCII).Trim()
    if ($head -match '^([a-f0-9]{40})$') { return $head }
    if ($head -cnotmatch '^ref: (refs/heads/[A-Za-z0-9._/-]+)$') { Stop-Run 64 'unsupported HEAD reference' }
    $relative = Join-Path $git ($head.Substring(5).Replace('/','\'))
    if ([System.IO.File]::Exists($relative)) {
        $oid = [System.IO.File]::ReadAllText($relative,[System.Text.Encoding]::ASCII).Trim()
        if ($oid -cmatch '^[a-f0-9]{40}$') { return $oid }
        Stop-Run 64 'loose HEAD reference is malformed'
    }
    $packed = Join-Path $git 'packed-refs'
    if (-not [System.IO.File]::Exists($packed)) { Stop-Run 64 'HEAD reference is unresolved' }
    $pattern = '^([a-f0-9]{40}) ' + [regex]::Escape($head.Substring(5)) + '$'
    $matches = @([System.IO.File]::ReadAllLines($packed,[System.Text.Encoding]::ASCII) | Where-Object { $_ -match $pattern })
    if ($matches.Count -ne 1) { Stop-Run 64 'packed HEAD reference is absent or ambiguous' }
    return $matches[0].Substring(0,40)
}
function Assert-HeadIndex { param([object]$Manifest)
    $head = Get-HeadOid $Manifest.source.checkoutRoot
    if ($head -cne $Manifest.sourceHead) { Stop-Run 30 'ordinary checkout HEAD drifted' }
    $indexPath = Join-Path (Join-Path $Manifest.source.checkoutRoot '.git') 'index'
    if (-not [System.IO.File]::Exists($indexPath)) { Stop-Run 30 'qualified empty Git index is absent' }
    $bytes = [System.IO.File]::ReadAllBytes($indexPath)
    if ($bytes.Length -lt 12 -or [System.Text.Encoding]::ASCII.GetString($bytes,0,4) -cne 'DIRC') { Stop-Run 30 'Git index format is unsupported' }
    if ((Get-ByteSha256 $bytes) -cne $Manifest.source.indexSha256) { Stop-Run 30 'qualified Git index changed' }
}
function Assert-TextPins {
    param([object]$Manifest)
    foreach ($pin in $Manifest.source.pins) {
        $path = Join-Path $Manifest.source.checkoutRoot ($pin.path.Replace('/','\'))
        if (-not [System.IO.File]::Exists($path)) { Stop-Run 30 'a source-pinned input is absent' }
        $info = [System.IO.FileInfo]::new($path)
        if (($info.Attributes -band [System.IO.FileAttributes]::ReparsePoint) -ne 0 -or $info.Length -ne $pin.bytes -or (Get-FileSha256 $path) -cne $pin.sha256) { Stop-Run 30 'a source-pinned input changed' }
        $bytes = [System.IO.File]::ReadAllBytes($path)
        try { [void]([System.Text.UTF8Encoding]::new($false,$true).GetString($bytes)) }
        catch { Stop-Run 30 'a source-pinned input is not strict UTF-8' }
    }
}
function Assert-ArtifactPins {
    param([object]$Manifest)
    foreach ($name in @('native','godot')) {
        $artifact = $Manifest.artifacts[$name]
        $full = [System.IO.Path]::GetFullPath($artifact.path)
        if (-not [System.IO.File]::Exists($full)) { Stop-Run 64 "qualified $name artifact is absent" }
        $info = [System.IO.FileInfo]::new($full)
        if (($info.Attributes -band [System.IO.FileAttributes]::ReparsePoint) -ne 0 -or $info.Length -ne $artifact.bytes -or (Get-FileSha256 $full) -cne $artifact.sha256) { Stop-Run 30 "qualified $name artifact identity changed" }
    }
}
function Assert-ResourcePins {
    param([object]$Manifest)
    $project = Join-Path $Manifest.source.checkoutRoot 'apps/godot_playable'
    foreach ($row in $Manifest.resources) {
        if ($row.presence -eq 'absent') { continue }
        $relative = $row.path.Substring(6)
        if ($relative.StartsWith('/') -or @($relative.Split('/')) -contains '..') { Stop-Run 64 'resource path escapes the production project' }
        $path = [System.IO.Path]::GetFullPath((Join-Path $project ($relative.Replace('/','\'))))
        $prefix = [System.IO.Path]::GetFullPath($project).TrimEnd('\') + '\'
        if (-not $path.StartsWith($prefix,[System.StringComparison]::OrdinalIgnoreCase) -or -not [System.IO.File]::Exists($path)) { Stop-Run 64 'qualified resource is absent or outside project' }
        $info = [System.IO.FileInfo]::new($path)
        if (($info.Attributes -band [System.IO.FileAttributes]::ReparsePoint) -ne 0 -or $info.Length -ne $row.bytes -or (Get-FileSha256 $path) -cne $row.sha256) { Stop-Run 30 'qualified resource identity changed' }
    }
}
function Assert-SourcePins {
    param([object]$Manifest)
    Assert-HeadIndex $Manifest
    Assert-TextPins $Manifest
}
function Assert-PurposeCoverage {
    param([object]$Manifest, [object]$Receipt)
    Assert-PurposeApproval $Manifest $Receipt
    if (@($Manifest.resources | Where-Object { $_.required -and $_.presence -ne 'present' }).Count -gt 0) {
        Stop-Run 64 'a required runtime resource is not present'
    }
}
function Assert-FreshNamespace {
    param([object]$Manifest)
    Assert-OrdinaryNamespace $Manifest
    $base = [System.IO.Path]::GetFullPath($Manifest.namespace.base)
    if (-not [System.IO.Directory]::Exists($base)) { Stop-Run 64 'qualified private parent namespace is absent' }
    $cursor = [System.IO.DirectoryInfo]::new($base)
    while ($null -ne $cursor) {
        if (($cursor.Attributes -band [System.IO.FileAttributes]::ReparsePoint) -ne 0) { Stop-Run 64 'namespace ancestor is a reparse point' }
        $cursor = $cursor.Parent
    }
    $root = [System.IO.Path]::GetFullPath($Manifest.namespace.runRoot)
    if ([System.IO.Directory]::Exists($root) -or [System.IO.File]::Exists($root)) { Stop-Run 64 'run UUID namespace already exists' }
}
function New-OwnedNamespace {
    param([object]$Manifest)
    Assert-FreshNamespace $Manifest
    $root = [System.IO.Path]::GetFullPath($Manifest.namespace.runRoot)
    [WxBoundedPipe]::CreateDirectoryNew($root)
    $directories = @('session','session/run','session/saves','closed','outputs','phases','phases/new','phases/new/profile',
        'phases/new/profile/roaming','phases/new/profile/local','phases/new/profile/temp','phases/new/outputs',
        'phases/continue','phases/continue/profile','phases/continue/profile/roaming','phases/continue/profile/local',
        'phases/continue/profile/temp','phases/continue/outputs')
    foreach ($relative in $directories) { [WxBoundedPipe]::CreateDirectoryNew((Join-Path $root $relative)) }
}
function New-ChildEnvironment {
    param([object]$Manifest, [string]$Phase, [string]$ConfigPath, [string]$ConfigSha256)
    $profile = Join-Path $Manifest.namespace.runRoot "phases/$Phase/profile"
    $expected = Join-Path (Join-Path $profile 'roaming') 'WuxianV1Campaign141'
    $envMap = @{
        APPDATA=(Join-Path $profile 'roaming'); LOCALAPPDATA=(Join-Path $profile 'local');
        TEMP=(Join-Path $profile 'temp'); TMP=(Join-Path $profile 'temp');
        WUXIAN_GH_CONFIG=$ConfigPath; WUXIAN_GH_CONFIG_SHA256=$ConfigSha256;
        WUXIAN_GH_PHASE=$Phase; WUXIAN_GH_EXPECTED_USER_DIR=$expected
    }
    return [pscustomobject]@{ Values=$envMap; ExpectedUserDir=$expected }
}
function Get-NativeArguments {
    param([string]$Session, [string]$Saves, [string]$Token)
    return [string[]]@('--session-dir',$Session,'--save-dir',$Saves,'--session-token',$Token)
}
function Get-RpcArguments { param([string]$Ready, [string]$Token); $i=Get-Interface; return [string[]]@($i.rpcArguments[0],$Ready,$i.rpcArguments[1],$Token) }
function Get-GodotArguments {
    param([object]$Manifest, [string]$Ready, [string]$Token, [string]$LogPath)
    $project = Join-Path $Manifest.source.checkoutRoot 'apps/godot_playable'
    $rpc = Get-RpcArguments $Ready $Token
    return [string[]](@('--path',$project,'--rendering-method','gl_compatibility','--audio-driver','Dummy',
        '--log-file',$LogPath,'--script','res://tests/campaign/grey_hive_first_playable.gd','--') + $rpc)
}
function Pump-Pipes {
    param([object]$Owned, [object]$Manifest)
    $stdoutCap = [long]$Manifest.limits.stdoutBytes; $stderrCap = [long]$Manifest.limits.stderrBytes
    $Owned.Stdout = [WxBoundedPipe]::new($Owned.Process.StandardOutput.BaseStream,$stdoutCap)
    $Owned.Stderr = [WxBoundedPipe]::new($Owned.Process.StandardError.BaseStream,$stderrCap)
    $Owned.Stdout.Start(); $Owned.Stderr.Start()
}
function Start-OwnedProcess {
    param([object]$State, [string]$Role, [string]$Executable, [string[]]$Arguments, [hashtable]$Environment, [string]$WorkingDirectory, [object]$Manifest)
    if (-not [System.IO.Path]::IsPathRooted($Executable) -or -not [System.IO.Directory]::Exists($WorkingDirectory)) { Stop-Run 64 'process path is not qualified' }
    $psi = [System.Diagnostics.ProcessStartInfo]::new()
    $psi.FileName = [System.IO.Path]::GetFullPath($Executable); $psi.WorkingDirectory = [System.IO.Path]::GetFullPath($WorkingDirectory)
    $psi.UseShellExecute = $false; $psi.CreateNoWindow = $false; $psi.RedirectStandardOutput = $true; $psi.RedirectStandardError = $true
    foreach ($argument in $Arguments) { [void]$psi.ArgumentList.Add([string]$argument) }
    foreach ($key in @('APPDATA','LOCALAPPDATA','TEMP','TMP','WUXIAN_GH_CONFIG','WUXIAN_GH_CONFIG_SHA256','WUXIAN_GH_PHASE','WUXIAN_GH_EXPECTED_USER_DIR')) {
        if ($Environment.ContainsKey($key)) { $psi.Environment[$key] = [string]$Environment[$key] }
    }
    $process = [System.Diagnostics.Process]::new()
    $process.StartInfo = $psi
    $owned = [pscustomobject]@{ Role=$Role; Process=$process; Executable=$psi.FileName; Arguments=[string[]]$Arguments;
        Started=$false; Id=0; StartUtc=$null; CreationUtc=$null; Stdout=$null; Stderr=$null; ExitCode=$null }
    $State.Processes.Add($owned)
    try {
        if (-not $process.Start()) { Stop-Run 40 'owned process did not start' }
        $owned.Started = $true; $owned.Id = $process.Id; $owned.StartUtc = [DateTimeOffset]::UtcNow
        $owned.CreationUtc = $process.StartTime.ToUniversalTime()
        Pump-Pipes $owned $Manifest
    } catch {
        if ($_.Exception.Message -match '^WXSTATUS:') { throw }
        Stop-Run 40 'owned process start or pipe initialization failed'
    }
    return $owned
}
function Get-ProcessCreationUtc {
    param([object]$Value)
    if ($Value -is [DateTime]) { if ($Value.Kind -eq [DateTimeKind]::Unspecified) { Stop-Run 40 'process creation time has an unspecified kind' }; return $Value.ToUniversalTime() }
    if ($Value -is [DateTimeOffset]) { return $Value.UtcDateTime }
    if ($Value -is [string] -and $Value -cmatch '\A[0-9]{14}\.[0-9]{6}[+-][0-9]{3}\z') {
        try { return [System.Management.ManagementDateTimeConverter]::ToDateTime($Value).ToUniversalTime() }
        catch { Stop-Run 40 'process creation time DMTF value is invalid' } }
    Stop-Run 40 'process creation time representation is unsupported'
}
function Assert-ProcessIdentity {
    param([object]$Owned)
    if ($null -eq $Owned -or -not $Owned.Started -or $Owned.Id -le 0 -or $Owned.Process.Id -ne $Owned.Id) { Stop-Run 40 'retained process identity is incomplete' }
    try {
        $native = Get-CimInstance -ClassName Win32_Process -Filter "ProcessId = $($Owned.Id)" -ErrorAction Stop
        $identityBranch = $null; $cimImage = $null; $moduleImage = $null
        if ($null -eq $native) { $identityBranch = 'cim-absent' }
        elseif ($native.ProcessId -ne $Owned.Id) { $identityBranch = 'pid-mismatch' }
        elseif (($cimImage = [System.IO.Path]::GetFullPath($native.ExecutablePath)) -ine $Owned.Executable) { $identityBranch = 'cim-image-mismatch' }
        elseif (($moduleImage = [System.IO.Path]::GetFullPath($Owned.Process.MainModule.FileName)) -ine $Owned.Executable) { $identityBranch = 'module-image-mismatch' }
        if ($null -ne $identityBranch) {
            $detail = "branch=$identityBranch diagnostic=unavailable"
            try {
                $caller = @(Get-PSCallStack)[2]
                $detail = [ordered]@{
                    branch = $identityBranch; diagnosticUtc = [DateTime]::UtcNow.ToString('o')
                    role = [string]$Owned.Role; expectedPid = $Owned.Id
                    expectedBirthUtc = $Owned.CreationUtc.ToString('o'); birthGuard = 'not-reached'
                    queryPresent = ($null -ne $native)
                    actualPid = $(if ($null -eq $native) { $null } else { $native.ProcessId })
                    expectedImage = $Owned.Executable; cimImage = $cimImage; moduleImage = $moduleImage
                    nullImageMeaning = 'not-evaluated'; caller = [string]$caller.FunctionName
                    callerLine = [int]$caller.ScriptLineNumber
                } | ConvertTo-Json -Compress
                if ([Text.Encoding]::UTF8.GetByteCount($detail) -gt 16384) {
                    $detail = "branch=$identityBranch diagnostic=omitted-oversize"
                }
            } catch {}
            Stop-Run 40 ("retained process identity check failed $detail")
        }
        $arguments = [WxBoundedPipe]::SplitCommandLine([string]$native.CommandLine)
        if ($arguments.Count -ne ($Owned.Arguments.Count + 1) -or $arguments[0] -ine $Owned.Executable) { Stop-Run 40 'retained process argument vector changed' }
        for ($index=0; $index -lt $Owned.Arguments.Count; $index++) {
            if ($arguments[$index+1] -cne $Owned.Arguments[$index]) { Stop-Run 40 'retained process arguments changed' }
        }
        $created = Get-ProcessCreationUtc $native.CreationDate
        if ([math]::Abs(($created - $Owned.CreationUtc).TotalSeconds) -gt 2.0) { Stop-Run 40 'retained process birth time changed' }
    } catch {
        if ($_.Exception.Message -match '^WXSTATUS:') { throw }
        Stop-Run 40 'retained process ownership could not be reverified'
    }
}
function Assert-OwnedIdentity { param([object]$Owned); Assert-ProcessIdentity $Owned }
function Assert-ReadyOwner {
    param([object]$Ready, [object]$Native, [string]$Session, [string]$Saves, [string]$Token)
    foreach ($field in @('processId','sessionDir','saveDir','sessionToken','host','port')) {
        if (-not $Ready.Contains($field)) { Stop-Run 40 'ready record omits an ownership field' }
    }
    if ([long]$Ready.processId -ne $Native.Id -or [string]$Ready.sessionDir -ine $Session -or
        [string]$Ready.saveDir -ine $Saves -or [string]$Ready.sessionToken -cne $Token -or
        [string]$Ready.host -cne '127.0.0.1' -or [long]$Ready.port -lt 1 -or [long]$Ready.port -gt 65535) {
        Stop-Run 40 'ready record does not bind the retained native session'
    }
    Assert-OwnedIdentity $Native
    $listeners = @(Get-NetTCPConnection -State Listen -LocalPort ([int]$Ready.port) -ErrorAction Stop | Where-Object { $_.LocalAddress -eq '127.0.0.1' -and $_.OwningProcess -eq $Native.Id })
    if ($listeners.Count -ne 1) { Stop-Run 40 'loopback listener is not uniquely owned by retained native process' }
}
function Assert-ReadyIdentity {
    param([string]$ReadyPath, [object]$Native, [string]$Session, [string]$Saves, [string]$Token)
    if (-not [System.IO.File]::Exists($ReadyPath)) { return $false }
    try { $record = Read-StrictJson $ReadyPath 65536 }
    catch { return $false }
    if ($record.Value -isnot [System.Collections.IDictionary]) { Stop-Run 40 'ready record is not a JSON object' }
    Assert-OwnedIdentity $Native
    Assert-ReadyOwner $record.Value $Native $Session $Saves $Token
    return [pscustomobject]@{ Value=$record.Value; Bytes=$record.Bytes; Sha256=(Get-ByteSha256 $record.Bytes) }
}
function Write-PhaseConfig {
    param([object]$ManifestRecord, [string]$Phase, [string]$Token)
    $i = Get-Interface
    $manifest = $ManifestRecord.Value
    $root = $manifest.namespace.runRoot
    $profile = Join-Path $root "phases/$Phase/profile"
    $expected = Join-Path (Join-Path $profile 'roaming') 'WuxianV1Campaign141'
    $config = [ordered]@{ schema=$i.phaseConfigSchema; phase=$Phase; manifestSha256=$ManifestRecord.Sha256;
        manifest=$manifest; sessionDir=(Join-Path $root 'session'); saveDir=(Join-Path $root 'session/saves');
        expectedUserDir=$expected; sessionTokenSha256=(Get-ByteSha256 ([System.Text.Encoding]::UTF8.GetBytes($Token))) }
    $bytes = [System.Text.UTF8Encoding]::new($false).GetBytes(($config | ConvertTo-Json -Depth 16 -Compress))
    if ($bytes.Length -gt $i.maxConfigBytes) { Stop-Run 64 'phase configuration exceeds its shared byte bound' }
    $path = Join-Path $root "phases/$Phase/phase.json"
    $stream = [System.IO.File]::Open($path,[System.IO.FileMode]::CreateNew,[System.IO.FileAccess]::Write,[System.IO.FileShare]::None)
    try { $stream.Write($bytes,0,$bytes.Length); $stream.Flush($true) } finally { $stream.Dispose() }
    return [pscustomobject]@{ Path=$path; Sha256=(Get-ByteSha256 $bytes); ExpectedUserDir=$expected }
}
function New-SessionToken {
    $bytes = [byte[]]::new(32)
    $rng = [System.Security.Cryptography.RandomNumberGenerator]::Create()
    try { $rng.GetBytes($bytes) } finally { $rng.Dispose() }
    return ([System.BitConverter]::ToString($bytes)).Replace('-','').ToLowerInvariant()
}
function Start-OwnedPair {
    param([object]$State, [object]$ManifestRecord, [string]$Phase)
    $manifest = $ManifestRecord.Value; $root = $manifest.namespace.runRoot
    Assert-SourcePins $manifest; Assert-ArtifactPins $manifest; Assert-ResourcePins $manifest
    $session = Join-Path $root 'session'; $saves = Join-Path $root 'session/saves'
    $ready = Join-Path $root 'session/run/bridge-ready.json'; $token = New-SessionToken
    $native = $ManifestRecord.Value.artifacts.native
    $baseEnvironment = @{ APPDATA=(Join-Path $root "phases/$Phase/profile/roaming");
        LOCALAPPDATA=(Join-Path $root "phases/$Phase/profile/local"); TEMP=(Join-Path $root "phases/$Phase/profile/temp"); TMP=(Join-Path $root "phases/$Phase/profile/temp") }
    $nativeArgs = Get-NativeArguments $session $saves $token
    $nativeOwned = Start-OwnedProcess $State 'native' $native.path $nativeArgs $baseEnvironment $root $manifest
    $timer = [System.Diagnostics.Stopwatch]::StartNew(); $readyOk = $false
    while ($timer.Elapsed.TotalSeconds -lt [double]$manifest.limits.startupSeconds -and [DateTimeOffset]::UtcNow -lt $script:RunDeadlineUtc) {
        if ($nativeOwned.Process.HasExited) { Stop-Run 20 'native service exited before authenticated readiness' }
        $readyProof = Assert-ReadyIdentity $ready $nativeOwned $session $saves $token
        if ($readyProof -is [System.Management.Automation.PSCustomObject]) { $readyOk = $true; break }
        Start-Sleep -Milliseconds 100
    }
    if (-not $readyOk) { Stop-Run 30 'native readiness or total Run deadline expired' }
    $phaseConfig = Write-PhaseConfig $ManifestRecord $Phase $token
    $childEnvironment = New-ChildEnvironment $manifest $Phase $phaseConfig.Path $phaseConfig.Sha256
    if ($childEnvironment.ExpectedUserDir -cne $phaseConfig.ExpectedUserDir) { Stop-Run 64 'phase profile binding changed' }
    $log = Join-Path $root "phases/$Phase/outputs/godot.log"
    if ([System.IO.File]::Exists($log)) { Stop-Run 64 'owned engine log already exists' }
    $godotArgs = Get-GodotArguments $manifest $ready $token $log
    $godot = $ManifestRecord.Value.artifacts.godot
    $godotOwned = Start-OwnedProcess $State 'godot' $godot.path $godotArgs $childEnvironment.Values $root $manifest
    Assert-OwnedIdentity $nativeOwned; Assert-OwnedIdentity $godotOwned
    return [pscustomobject]@{ Phase=$Phase; Native=$nativeOwned; Godot=$godotOwned; Token=$token;
        ReadyPath=$ready; ReadyProof=$readyProof; Session=$session; Saves=$saves; Profile=$childEnvironment.ExpectedUserDir; StartedUtc=[DateTimeOffset]::UtcNow }
}
function Get-PipeText {
    param([WxBoundedPipe]$Pipe)
    try { return [System.Text.UTF8Encoding]::new($false,$true).GetString($Pipe.Raw) }
    catch { Stop-Run 42 'owned child output is not strict UTF-8' }
}
function Read-PhaseEvents {
    param([object]$Pair, [object]$Manifest)
    $i = Get-Interface
    $relative = "phases/$($Pair.Phase)/outputs/$($i.eventFileName)"
    $path = Join-Path $Manifest.namespace.runRoot ($relative.Replace('/','\'))
    if (-not [System.IO.File]::Exists($path)) { Stop-Run 30 'phase event file is absent' }
    $info = [System.IO.FileInfo]::new($path)
    if (($info.Attributes -band [System.IO.FileAttributes]::ReparsePoint) -ne 0 -or $info.Length -le 0 -or $info.Length -gt $i.maxEventFileBytes) { Stop-Run 30 'phase event file is outside its owned byte bound' }
    if ($Pair.Phase -ceq 'new' -and $info.Length -gt 67108864) { Stop-Run 30 'phase-A events exceed the existing Continue reader bound' }
    $bytes = [System.IO.File]::ReadAllBytes($path)
    try { $text = [System.Text.UTF8Encoding]::new($false,$true).GetString($bytes) }
    catch { Stop-Run 42 'phase event file is not strict UTF-8' }
    if (($text.Length -gt 0 -and $text[0] -eq [char]0xFEFF) -or -not $text.EndsWith("`n")) { Stop-Run 42 'phase event file framing is invalid' }
    $lines = $text.Split([char[]]@([char]10))
    $count = $lines.Length - 1
    if ($count -lt 1 -or $count -gt $i.maxEventCount -or $lines[$count] -cne '') { Stop-Run 30 'phase event count or final line is invalid' }
    $events = [System.Collections.Generic.List[object]]::new()
    $expectedSequence = 1
    for ($index=0; $index -lt $count; $index++) {
        $line = $lines[$index]
        if ([string]::IsNullOrEmpty($line) -or [System.Text.Encoding]::UTF8.GetByteCount($line) -gt $i.maxEventLineBytes) { Stop-Run 42 'phase event line is empty or oversized' }
        try {
            $doc = [System.Text.Json.JsonDocument]::Parse($line)
            try { Test-DuplicateJsonKeys $doc.RootElement } finally { $doc.Dispose() }
            $event = ConvertFrom-Json -InputObject $line -AsHashtable -Depth 32 -DateKind String
            Assert-ExactKeys $event $i.eventKeys 'phase event'
        } catch { Stop-Run 42 'phase event record is malformed or has an unsupported key set' }
        if ($event.schema -cne $i.evidenceSchema -or $event.runId -cne $Manifest.runId -or $event.phase -cne $Pair.Phase -or
            $event.manifestSha256 -cne $script:RunState.ManifestRecord.Sha256 -or
            $event.sourceBaselineReference -cne $Manifest.authority.sourceBaseline.reference -or
            $event.payload -isnot [System.Collections.IDictionary]) {
            Stop-Run 30 'phase event does not bind the accepted run, source, or payload shape'
        }
        Assert-Integer $event.sequence 'event sequence' $expectedSequence $expectedSequence
        Assert-Integer $event.monotonicMs 'event monotonic time' 0 2147483647
        Assert-String $event.kind 'event kind' 1 128; Assert-String $event.outcome 'event outcome' 1 128
        if ($event.utc -isnot [string] -or $event.utc -cnotmatch '^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d{1,7})?Z$') { Stop-Run 30 'phase event timestamp is not canonical UTC' }
        try { if ([DateTimeOffset]::Parse($event.utc,[Globalization.CultureInfo]::InvariantCulture,[Globalization.DateTimeStyles]::RoundtripKind).Offset -ne [TimeSpan]::Zero) { Stop-Run 30 'phase event timestamp is not UTC' } }
        catch { Stop-Run 30 'phase event timestamp is invalid' }
        $events.Add($event)
        $expectedSequence++
    }
    return $events.ToArray()
}
function Assert-SessionClose {
    param([object]$Pair, [object]$Manifest)
    $i = Get-Interface
    foreach ($pipe in @($Pair.Godot.Stdout,$Pair.Godot.Stderr,$Pair.Native.Stdout,$Pair.Native.Stderr)) { [void](Get-PipeText $pipe) }
    $phaseEvents = @(Read-PhaseEvents $Pair $Manifest)
    foreach ($requirement in $Manifest.acceptance.evidence | Where-Object { $_.phase -ceq $Pair.Phase }) {
        if (@($phaseEvents | Where-Object { $_.kind -ceq $requirement.kind -and $_.outcome -ceq $requirement.outcome }).Count -lt 1) { Stop-Run 20 'manifest-required acceptance evidence is absent' }
    }
    $closeEvents = @($phaseEvents | Where-Object { $_.kind -ceq $i.terminalKind })
    $terminal = @($closeEvents | Where-Object { $_.outcome -ceq $i.terminalOutcome })
    $accepted = @($phaseEvents | Where-Object { $_.kind -ceq $i.acceptedKind -and $_.outcome -ceq $i.acceptedOutcome })
    if ($closeEvents.Count -ne 1 -or $terminal.Count -ne 1 -or $accepted.Count -ne 1 -or
        $terminal[0].payload -isnot [System.Collections.IDictionary] -or
        $terminal[0].payload.association -isnot [System.Collections.IDictionary] -or
        $accepted[0].sequence -le $terminal[0].sequence) {
        Stop-Run 30 'authenticated normal session_close evidence or accepted terminal event is absent or ambiguous'
    }
    Assert-Boolean $terminal[0].payload.authenticated 'session_close.authenticated'
    Assert-Boolean $terminal[0].payload.closing 'session_close.closing'
    Assert-Integer $terminal[0].payload.connectionGeneration 'session_close.connectionGeneration' 0 2147483647
    Assert-ExactKeys $terminal[0].payload $i.terminalPayloadKeys 'session_close payload'
    if (-not $terminal[0].payload.authenticated -or -not $terminal[0].payload.closing -or
        $terminal[0].payload.association.purpose -cne 'close') { Stop-Run 30 'session_close authentication, association, or state is invalid' }
    if ($Pair.Phase -eq 'new') {
        $newReceipt = @($phaseEvents | Where-Object { $_.kind -ceq 'native_receipt' -and $_.outcome -ceq 'CONFIRMED' -and $_.payload.method -ceq 'formal_new' })
        $saveReceipt = @($phaseEvents | Where-Object { $_.kind -ceq 'native_receipt' -and $_.outcome -ceq 'CONFIRMED' -and $_.payload.method -ceq 'formal_save' })
        $routeVisits = @($phaseEvents | Where-Object { $_.kind -ceq 'route_visit' -and $_.outcome -ceq 'FIRST_SCENE_READY' } | Sort-Object sequence)
        $route = @($phaseEvents | Where-Object { $_.kind -ceq 'route_complete' -and $_.outcome -ceq 'CONFIRMED' })
        $choice = @($phaseEvents | Where-Object { $_.kind -ceq 'baizhi_choice' -and $_.outcome -ceq 'CONFIRMED' })
        $extraction = @($phaseEvents | Where-Object { $_.kind -ceq 'beacon_extraction' -and $_.outcome -ceq 'AUTHORITATIVE_RECEIPT_AND_STATE_CONFIRMED' })
        $returnGate = @($phaseEvents | Where-Object { $_.kind -ceq 'return_gate' -and $_.outcome -ceq 'AUTHORITATIVE_RECEIPT_CONFIRMED' })
        $summary = @($phaseEvents | Where-Object { $_.kind -ceq 'phase_summary' -and $_.outcome -ceq 'SAVED' })
        if ($newReceipt.Count -ne 1 -or $saveReceipt.Count -ne 1 -or $routeVisits.Count -ne $Manifest.acceptance.route.Count -or
            $route.Count -ne 1 -or $choice.Count -ne 1 -or $extraction.Count -ne 1 -or $returnGate.Count -ne 1 -or $summary.Count -ne 1) {
            Stop-Run 20 'phase-A journey, route, progression, choice, extraction, return, or save evidence is incomplete'
        }
        for ($routeIndex=0; $routeIndex -lt $Manifest.acceptance.route.Count; $routeIndex++) {
            if ($routeVisits[$routeIndex].payload.routeIndex -ne $routeIndex -or $routeVisits[$routeIndex].payload.sceneId -cne $Manifest.acceptance.route[$routeIndex]) { Stop-Run 20 'phase-A route visits are not the exact ordered route' }
        }
        if ((ConvertTo-Json -InputObject $route[0].payload.visits -Compress) -cne (ConvertTo-Json -InputObject $Manifest.acceptance.route -Compress) -or
            $route[0].payload.routeIndex -ne $Manifest.acceptance.route.Count -or $route[0].payload.snapshot.worldId -cne 'return_station' -or
            $route[0].payload.snapshot.sceneId -cne $i.returnScene -or $choice[0].payload.choice -cne $Manifest.acceptance.baizhiChoice -or
            $returnGate[0].payload.gateId -cne $i.returnGate -or $returnGate[0].payload.after.sceneId -cne $i.returnScene -or
            $summary[0].payload.routeComplete -ne $true -or $summary[0].payload.extractionConfirmed -ne $true) {
            Stop-Run 20 'phase-A terminal route or operator choice does not match the qualified manifest'
        }
        $progress = $extraction[0].payload.afterProgression
        $progressEvents = @($progress.completedEvents)
        if ($progress.completed -ne $true -or $progress.firstCompletion -ne $true -or $progress.revisitCount -ne 0 -or
            $progress.visitId -ne 1 -or $progressEvents.Count -ne $i.requiredEvents.Count -or
            @($i.requiredEvents | Where-Object { $_ -notin $progressEvents }).Count -ne 0 -or
            @($Manifest.acceptance.route | Where-Object { $_ -notin $route[0].payload.visits }).Count -ne 0) {
            Stop-Run 20 'authoritative completed hive progression or route evidence is incomplete'
        }
    } else {
        $continueReceipt = @($phaseEvents | Where-Object { $_.kind -ceq 'native_receipt' -and $_.outcome -ceq 'CONFIRMED' -and $_.payload.method -ceq 'formal_continue' })
        $comparison = @($phaseEvents | Where-Object { $_.kind -ceq 'continue_comparison' -and $_.outcome -ceq 'MATCHED' })
        if ($continueReceipt.Count -ne 1 -or $comparison.Count -ne 1) { Stop-Run 20 'phase-B Continue receipt or preserved-field comparison is absent' }
    }
    $expectedCaptures = @($Manifest.limits.captureList | Where-Object { $_.phase -ceq $Pair.Phase })
    $captureEvents = @($phaseEvents | Where-Object { $_.kind -ceq 'completed_frame_capture' -and $_.outcome -ceq 'SAVED' })
    if ($captureEvents.Count -ne $expectedCaptures.Count) { Stop-Run 20 'capture evidence does not match the allocated list' }
    foreach ($capture in $expectedCaptures) {
        $eventMatches = @($captureEvents | Where-Object { $_.payload.label -ceq $capture.id })
        $locator = "phases/$($Pair.Phase)/outputs/capture-$($capture.id).png"
        if ($eventMatches.Count -ne 1 -or $eventMatches[0].payload.locator -cne $locator -or
            $eventMatches[0].payload.binding.scene.sceneId -cne $capture.sceneId) { Stop-Run 20 'capture event does not bind its allocated scene and owned path' }
        $sequence = [long]$eventMatches[0].sequence
        if ($sequence -ge [long]$terminal[0].sequence) { Stop-Run 20 'capture must precede authenticated close' }
        if ($capture.id -ceq 'returned-rs-saved' -and ($sequence -le [long]$summary[0].sequence -or $saveReceipt[0].payload.association.exitAfter -ne $false)) { Stop-Run 20 'returned frame must follow the confirmed non-exiting save' }
        if ($capture.id -ceq 'gh-exit-extraction' -and $sequence -le [long]$extraction[0].sequence) { Stop-Run 20 'exit frame must follow authoritative extraction' }
        $capturePath = Join-Path $Manifest.namespace.runRoot ($locator.Replace('/','\'))
        $captureInfo = if ([System.IO.File]::Exists($capturePath)) { [System.IO.FileInfo]::new($capturePath) } else { $null }
        if ($null -eq $captureInfo -or ($captureInfo.Attributes -band [System.IO.FileAttributes]::ReparsePoint) -ne 0 -or $captureInfo.Length -le 0) { Stop-Run 20 'capture output is not an allocated owned file' }
    }
}
function Assert-Drained {
    param([object[]]$Processes, [int]$TimeoutSeconds)
    $watch = [System.Diagnostics.Stopwatch]::StartNew()
    foreach ($owned in $Processes) {
        foreach ($pipe in @($owned.Stdout,$owned.Stderr)) {
            if (-not $pipe.Wait([math]::Max(1,($TimeoutSeconds*1000)-[int]$watch.ElapsedMilliseconds))) { Stop-Run 42 'owned output pipe did not reach EOF within drain bound' }
            if (-not $pipe.Eof -or $pipe.Overflow -or $null -ne $pipe.Fault) { Stop-Run 42 'owned output pipe ended with EOF, overflow, or read fault failure' }
        }
    }
}
function Write-PipeEvidence {
    param([object]$Pair,[object]$Manifest)
    $files=@(); $root=Join-Path $Manifest.namespace.runRoot 'outputs'
    foreach($owned in @($Pair.Native,$Pair.Godot)) { foreach($kind in @('Stdout','Stderr')) {
        $pipe=$owned.$kind; $bytes=$pipe.Raw; $path=Join-Path $root "$($Pair.Phase)-$($owned.Role)-$($kind.ToLowerInvariant()).raw"
        $stream=[System.IO.File]::Open($path,[System.IO.FileMode]::CreateNew,[System.IO.FileAccess]::Write,[System.IO.FileShare]::None)
        try { $stream.Write($bytes,0,$bytes.Length); $stream.Flush($true) } finally { $stream.Dispose() }
        $files += [pscustomobject]@{path=$path;rawPipeBytes=$pipe.RawPipeBytes;storedBytes=$bytes.Length;eof=$pipe.Eof;overflow=$pipe.Overflow;fault=$pipe.Fault}
    } }
    return $files
}
function Wait-OwnedPairClosed {
    param([object]$Pair, [object]$Manifest)
    $processes = @($Pair.Native,$Pair.Godot)
    $deadline = [DateTimeOffset]::UtcNow.AddSeconds([double]$Manifest.limits.phaseSeconds + [double]$Manifest.limits.closeSeconds); if ($script:RunDeadlineUtc -lt $deadline) { $deadline = $script:RunDeadlineUtc }
    while (@($processes | Where-Object { -not $_.Process.HasExited }).Count -gt 0) {
        if ([DateTimeOffset]::UtcNow -ge $deadline) { Stop-Run 30 'operator phase did not close naturally within its bound' }
        foreach ($owned in $processes) { if (-not $owned.Process.HasExited) { Assert-OwnedIdentity $owned } }
        Start-Sleep -Milliseconds 100
    }
    foreach ($owned in $processes) { $owned.Process.WaitForExit() }
    Assert-Drained $processes ([int]$Manifest.limits.drainSeconds)
    foreach ($owned in $processes) { foreach ($pipe in @($owned.Stdout,$owned.Stderr)) {
        if (-not $pipe.Eof -or $pipe.Overflow -or $null -ne $pipe.Fault) { Stop-Run 42 'pipe EOF, overflow, or fault state failed final closure review' }
    } }
    $pipeEvidence=Write-PipeEvidence $Pair $Manifest
    foreach ($owned in $processes) {
        $owned.ExitCode = $owned.Process.ExitCode
        if ($owned.ExitCode -ne 0) { Stop-Run 20 "$($owned.Role) process exited unsuccessfully" }
    }
    Assert-SessionClose $Pair $Manifest
    foreach ($owned in $processes) { $owned.Process.Dispose() }
    return [pscustomobject]@{ Natural=$true; Forced=$false; ExitCodes=@($Pair.Native.ExitCode,$Pair.Godot.ExitCode); PipeEvidence=$pipeEvidence }
}
function Invoke-OwnedFallback {
    param([object]$State, [object]$Manifest, [object]$Pair = $null)
    Assert-FallbackAuthorization $State
    if ($null -ne $Pair) { $targets = @($Pair.Godot,$Pair.Native) }
    else { $targets = @($State.Processes | Where-Object { $_.Started }) }
    $targets = @($targets | Where-Object { $_.Started -and $null -eq $_.ExitCode })
    $activeTargets = [System.Collections.Generic.List[object]]::new()
    foreach ($owned in $targets) {
        if (-not $owned.Started -or $null -ne $owned.ExitCode) { continue }
        try { $active = -not $owned.Process.HasExited } catch { Stop-Run 40 'retained process exit state is unknown' }
        if ($active) {
            Assert-OwnedIdentity $owned
            $activeTargets.Add($owned)
        }
    }
    $killFault = $false
    foreach ($owned in $activeTargets) {
        try { $owned.Process.Kill() } catch { $killFault = $true }
    }
    $hard = [DateTimeOffset]::Parse($Manifest.deadlineUtc,[Globalization.CultureInfo]::InvariantCulture,[Globalization.DateTimeStyles]::RoundtripKind)
    $closeWatch = [System.Diagnostics.Stopwatch]::StartNew()
    $unresolved = $killFault
    foreach ($owned in $activeTargets) {
        $remainingHard = [int]($hard-[DateTimeOffset]::UtcNow).TotalMilliseconds
        $remainingBound = [int]([double]$Manifest.limits.closeSeconds*1000)-[int]$closeWatch.ElapsedMilliseconds
        $budget = [math]::Min($remainingHard,$remainingBound)
        if ($budget -le 0 -or -not $owned.Process.WaitForExit($budget)) { $unresolved = $true }
    }
    $drainWatch = [System.Diagnostics.Stopwatch]::StartNew()
    foreach ($owned in $targets) {
        try {
            if (-not $owned.Process.HasExited) { $unresolved = $true; continue }
            $owned.Process.WaitForExit(); $owned.ExitCode = $owned.Process.ExitCode
        } catch { $unresolved = $true }
        foreach ($pipe in @($owned.Stdout,$owned.Stderr)) {
            if ($null -ne $pipe) {
                $remainingHard = [int]($hard-[DateTimeOffset]::UtcNow).TotalMilliseconds
                $remainingDrain = [int]([double]$Manifest.limits.drainSeconds*1000)-[int]$drainWatch.ElapsedMilliseconds
                $left = [math]::Min($remainingHard,$remainingDrain)
                if ($left -le 0 -or -not $pipe.Wait($left)) { $unresolved = $true }
            }
        }
        try { $owned.Process.Dispose() } catch { $unresolved = $true }
    }
    if ($unresolved -or [DateTimeOffset]::UtcNow -gt $hard) { Stop-Run 40 'owned fallback closure or bounded stream drain remains unresolved' }
    Stop-Run 41 'owned fallback forced closure; observation is not accepted'
}
function Confirm-OwnedSave {
    param([object]$Pair, [object]$Manifest)
    $path = Join-Path $Pair.Saves 'formal-save-v6.json'
    if (-not [System.IO.File]::Exists($path)) { Stop-Run 20 'phase A did not produce the actual owned save' }
    $read = Read-StrictJson $path 16777216
    if ($read.Value -isnot [System.Collections.IDictionary]) { Stop-Run 30 'owned save identity is unavailable' }
    return [pscustomobject]@{ Path=$path; Bytes=$read.Bytes.Length; Sha256=(Get-ByteSha256 $read.Bytes); RunId=$Manifest.runId; Data=$read.Value }
}
function Archive-OwnedReady {
    param([object]$Pair, [object]$Manifest)
    $destination = Join-Path $Manifest.namespace.runRoot 'closed/new-ready.json'
    Assert-ReadyRecordSnapshot $Pair $Pair.ReadyPath
    if ([System.IO.File]::Exists($destination) -or [System.IO.Directory]::Exists($destination)) { Stop-Run 30 'ready archive destination is not absent' }
    [System.IO.File]::Move($Pair.ReadyPath,$destination)
    Assert-ReadyRecordSnapshot $Pair $destination
}
function Assert-ReadyRecordSnapshot {
    param([object]$Pair, [string]$Path)
    if ($null -eq $Pair.ReadyProof -or -not [System.IO.File]::Exists($Path)) { Stop-Run 30 'authenticated ready snapshot is absent' }
    $bytes = [System.IO.File]::ReadAllBytes($Path)
    if ((Get-ByteSha256 $bytes) -cne $Pair.ReadyProof.Sha256 -or
        -not [System.Collections.StructuralComparisons]::StructuralEqualityComparer.Equals($bytes,$Pair.ReadyProof.Bytes)) {
        Stop-Run 30 'ready record bytes differ from the authenticated pre-exit record'
    }
}
function Invoke-OwnedRun {
    param()
    $manifestRecord = Read-Manifest $ManifestPath $ManifestSha256
    Assert-RunAuthorization $manifestRecord
    $manifest = $manifestRecord.Value
    Assert-PurposeCoverage $manifest $script:ReceiptRecord.Value
    Assert-SourcePins $manifest
    Assert-ArtifactPins $manifest
    Assert-ResourcePins $manifest
    Assert-FreshNamespace $manifest
    if (-not ('WxBoundedPipe' -as [type])) { Add-Type -TypeDefinition $script:OwnedRuntimeSource -Language CSharp -ErrorAction Stop }
    $script:RunState = [pscustomobject]@{ ManifestRecord=$manifestRecord; Processes=[System.Collections.Generic.List[object]]::new() }
    $script:RunDeadlineUtc = [DateTimeOffset]::UtcNow.AddSeconds([double]$manifest.limits.totalSeconds)
    $hardDeadline = [DateTimeOffset]::Parse($manifest.deadlineUtc,[Globalization.CultureInfo]::InvariantCulture,[Globalization.DateTimeStyles]::RoundtripKind)
    $runCeiling = $hardDeadline.AddSeconds(-([double]$manifest.limits.closeSeconds+[double]$manifest.limits.drainSeconds))
    if ($runCeiling -lt $script:RunDeadlineUtc) { $script:RunDeadlineUtc = $runCeiling }
    if ($script:RunDeadlineUtc -le [DateTimeOffset]::UtcNow) { Stop-Run 64 'no bounded process-close and pipe-drain window remains' }
    try {
        New-OwnedNamespace $manifest
        $phaseA = Start-OwnedPair $script:RunState $manifestRecord 'new'
        $closeA = Wait-OwnedPairClosed $phaseA $manifest
        if (-not $closeA.Natural -or $closeA.Forced) { Stop-Run 41 'phase A did not close naturally' }
        $save = Confirm-OwnedSave $phaseA $manifest
        Archive-OwnedReady $phaseA $manifest
        Assert-SourcePins $manifest
        Assert-ArtifactPins $manifest
        Assert-ResourcePins $manifest
        $phaseB = Start-OwnedPair $script:RunState $manifestRecord 'continue'
        $closeB = Wait-OwnedPairClosed $phaseB $manifest
        if (-not $closeB.Natural -or $closeB.Forced) { Stop-Run 41 'phase B did not close naturally' }
        $saveAfter = Confirm-OwnedSave ([pscustomobject]@{ Saves=$phaseB.Saves }) $manifest
        foreach ($field in $manifest.acceptance.saveFields) {
            if (-not $save.Data.Contains($field) -or -not $saveAfter.Data.Contains($field)) { Stop-Run 20 'a selected save field is absent after Continue' }
            $before = ConvertTo-Json -InputObject $save.Data[$field] -Depth 32 -Compress
            $after = ConvertTo-Json -InputObject $saveAfter.Data[$field] -Depth 32 -Compress
            if ($before -cne $after) { Stop-Run 20 'Continue did not preserve a selected completion field' }
        }
        Write-RunSummary $manifestRecord $save $closeA $closeB
    } catch {
        if ($_.Exception.Message -match '^WXSTATUS:(40|41):') { throw }
        $active = @($script:RunState.Processes | Where-Object {
            if (-not $_.Started -or $null -ne $_.ExitCode) { return $false }
            try { return (-not $_.Process.HasExited) } catch { return $true }
        })
        if ($active.Count -gt 0) { Invoke-OwnedFallback $script:RunState $manifest }
        throw
    }
}
function Write-RunSummary {
    param([object]$ManifestRecord, [object]$Save, [object]$CloseA, [object]$CloseB)
    $path = Join-Path $ManifestRecord.Value.namespace.runRoot 'process.json'
    $summary = [ordered]@{ schema='wuxian.gh-first-playable.process-summary/v1'; runId=$ManifestRecord.Value.runId;
        manifestSha256=$ManifestRecord.Sha256; saveBytes=$Save.Bytes; saveSha256=$Save.Sha256;
        phaseAExitCodes=$CloseA.ExitCodes; phaseBPipeEvidence=$CloseB.PipeEvidence; phaseAPipeEvidence=$CloseA.PipeEvidence;
        phaseBExitCodes=$CloseB.ExitCodes; outcome='TWO_PHASE_OPERATOR_ACCEPTANCE' }
    $bytes = [System.Text.UTF8Encoding]::new($false).GetBytes(($summary | ConvertTo-Json -Depth 16 -Compress))
    $stream = [System.IO.File]::Open($path,[System.IO.FileMode]::CreateNew,[System.IO.FileAccess]::Write,[System.IO.FileShare]::None)
    try { $stream.Write($bytes,0,$bytes.Length); $stream.Flush($true) } finally { $stream.Dispose() }
}
function Invoke-Observation {
    param([string]$SelectedMode)
    if ($SelectedMode -eq 'Plan') {
        $plan = Get-Plan $ManifestPath $ManifestSha256
        $plan | ConvertTo-Json -Depth 24 -Compress | Write-Output
        exit 0
    }
    if ($SelectedMode -ne 'Run') { Stop-Run 64 'unsupported mode' }
    return
}
# BEGIN GH_DISPATCH
try {
    if ($Mode -eq 'Plan') {
        $plan = Get-Plan $ManifestPath $ManifestSha256
        $plan | ConvertTo-Json -Depth 24 -Compress | Write-Output
        exit 0
    }
    Invoke-Observation -SelectedMode $Mode
    Invoke-OwnedRun
} catch {
    $message = [string]$_.Exception.Message
    $code = 42
    if ($message -match '^WXSTATUS:(\d+):') { $code = [int]$Matches[1]; $message = $message.Substring($Matches[0].Length) }
    [Console]::Error.WriteLine("GH_FIRST_PLAYABLE status=$code detail=$message")
    exit $code
}
# END GH_DISPATCH
