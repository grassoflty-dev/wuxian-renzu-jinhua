param()

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..\..")).Path
$manifestPath = Join-Path $repoRoot "governance\dependencies\babylonjs.json"
$manifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json
if ($manifest.name -ne "babylonjs" -or $manifest.version -ne "9.26.2" -or $manifest.license -ne "Apache-2.0") {
    throw "E_BABYLON_MANIFEST_IDENTITY"
}

$registry = Invoke-RestMethod -Uri $manifest.registryMetadata -Method Get -TimeoutSec 60
if ($registry.name -ne $manifest.name -or $registry.version -ne $manifest.version -or
    $registry.license -ne $manifest.license -or $registry.dist.tarball -ne $manifest.tarball -or
    $registry.dist.integrity -ne $manifest.integrity -or
    $registry.repository.url -ne "git+https://github.com/BabylonJS/Babylon.js.git") {
    throw "E_BABYLON_REGISTRY_METADATA_MISMATCH"
}

$tempRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("babylon-vendor-" + [guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Path $tempRoot | Out-Null
try {
    $archive = Join-Path $tempRoot "babylonjs-$($manifest.version).tgz"
    Invoke-WebRequest -Uri $manifest.tarball -OutFile $archive -TimeoutSec 180

    $stream = [System.IO.File]::OpenRead($archive)
    try {
        $sha512 = [System.Security.Cryptography.SHA512]::Create()
        try { $integrity = "sha512-" + [Convert]::ToBase64String($sha512.ComputeHash($stream)) }
        finally { $sha512.Dispose() }
    }
    finally { $stream.Dispose() }
    if ($integrity -ne $manifest.integrity) { throw "E_BABYLON_TARBALL_INTEGRITY" }
    $tarballHash = (Get-FileHash -LiteralPath $archive -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($tarballHash -ne $manifest.tarballSha256) { throw "E_BABYLON_TARBALL_SHA256 actual=$tarballHash expected=$($manifest.tarballSha256)" }

    $extractRoot = Join-Path $tempRoot "extract"
    New-Item -ItemType Directory -Path $extractRoot | Out-Null
    & tar.exe -xzf $archive -C $extractRoot package/babylon.js package/license.md package/NOTICE.md
    if ($LASTEXITCODE -ne 0) { throw "E_BABYLON_TAR_EXTRACT" }

    $destinations = @{
        "server-rs/release-ui/vendor/babylon.js" = "package/babylon.js"
        "server-rs/release-ui/vendor/BABYLON-LICENSE.txt" = "package/license.md"
    }
    foreach ($relativeDestination in $destinations.Keys) {
        $expected = $manifest.files.$relativeDestination
        $source = Join-Path $extractRoot $destinations[$relativeDestination]
        $actual = (Get-FileHash -LiteralPath $source -Algorithm SHA256).Hash.ToLowerInvariant()
        if ($actual -ne $expected.sha256) { throw "E_BABYLON_FILE_SHA256:$relativeDestination" }
        $destination = Join-Path $repoRoot $relativeDestination
        New-Item -ItemType Directory -Force -Path (Split-Path -Parent $destination) | Out-Null
        Copy-Item -LiteralPath $source -Destination $destination -Force
    }

    $notice = Join-Path $extractRoot "package/NOTICE.md"
    $noticeHash = (Get-FileHash -LiteralPath $notice -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($noticeHash -ne $manifest.files.upstreamNotice.sha256) { throw "E_BABYLON_NOTICE_SHA256" }

    Write-Output "Babylon.js $($manifest.version) verified and rebuilt."
    Write-Output "npm integrity: $integrity"
    Write-Output "tarball SHA-256: $tarballHash"
    foreach ($relativeDestination in $destinations.Keys) {
        Write-Output "$relativeDestination SHA-256 $($manifest.files.$relativeDestination.sha256)"
    }
    Write-Output "upstream NOTICE SHA-256: $noticeHash"
}
finally {
    $tempParent = [System.IO.Path]::GetFullPath([System.IO.Path]::GetTempPath()).TrimEnd([System.IO.Path]::DirectorySeparatorChar) + [System.IO.Path]::DirectorySeparatorChar
    $resolvedTemp = [System.IO.Path]::GetFullPath($tempRoot)
    if ($resolvedTemp.StartsWith($tempParent, [System.StringComparison]::OrdinalIgnoreCase) -and
        [System.IO.Path]::GetFileName($resolvedTemp).StartsWith("babylon-vendor-", [System.StringComparison]::Ordinal)) {
        Remove-Item -LiteralPath $resolvedTemp -Recurse -Force
    }
}
