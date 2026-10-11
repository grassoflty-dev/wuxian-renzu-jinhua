param(
    [string]$OutputPath = (Join-Path $PSScriptRoot '..\server-rs\icons\yujie.ico')
)

$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing

$resolvedOutput = [System.IO.Path]::GetFullPath($OutputPath)
$outputDirectory = [System.IO.Path]::GetDirectoryName($resolvedOutput)
[System.IO.Directory]::CreateDirectory($outputDirectory) | Out-Null

$amber = [System.Drawing.ColorTranslator]::FromHtml('#d8a64c')
$amberSoft = [System.Drawing.ColorTranslator]::FromHtml('#f1c976')
$background = [System.Drawing.ColorTranslator]::FromHtml('#07111b')
$sizes = @(256, 128, 64, 48, 32, 16)

function New-PngFrame([int]$Size) {
    $bitmap = $null
    $graphics = $null
    $stream = $null
    $outline = $null
    $signal = $null
    $dot = $null
    try {
        $bitmap = New-Object System.Drawing.Bitmap -ArgumentList @(
            $Size,
            $Size,
            [System.Drawing.Imaging.PixelFormat]::Format32bppArgb
        )
        $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
        $graphics.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
        $graphics.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality
        $graphics.CompositingQuality = [System.Drawing.Drawing2D.CompositingQuality]::HighQuality
        $graphics.Clear($background)

        $center = [single]($Size / 2.0)
        $edge = [single]($Size * 0.16)
        $stroke = [single][Math]::Max(1.0, $Size * 0.035)
        $outline = [System.Drawing.Pen]::new($amber, $stroke)
        $outline.LineJoin = [System.Drawing.Drawing2D.LineJoin]::Miter
        $signal = [System.Drawing.Pen]::new($amber, $stroke)

        # The four-point outline and crossed signal lines mirror the release UI
        # brand mark: a rotated square, two rays, and no lettering.
        $diamond = [System.Drawing.PointF[]]@(
            [System.Drawing.PointF]::new($center, $edge),
            [System.Drawing.PointF]::new($Size - $edge, $center),
            [System.Drawing.PointF]::new($center, $Size - $edge),
            [System.Drawing.PointF]::new($edge, $center)
        )
        $ray = [single]($Size * 0.105)
        $graphics.DrawLine(
            $signal,
            [System.Drawing.PointF]::new($center, $ray),
            [System.Drawing.PointF]::new($center, $Size - $ray)
        )
        $graphics.DrawLine(
            $signal,
            [System.Drawing.PointF]::new($ray, $center),
            [System.Drawing.PointF]::new($Size - $ray, $center)
        )
        $graphics.DrawPolygon($outline, $diamond)

        $dotRadius = [single]($Size * 0.075)
        $dot = [System.Drawing.SolidBrush]::new($amberSoft)
        $graphics.FillEllipse(
            $dot,
            $center - $dotRadius,
            $center - $dotRadius,
            $dotRadius * 2,
            $dotRadius * 2
        )

        $marker = [System.Drawing.SolidBrush]::new($amberSoft)
        try {
            $markerSide = [single][Math]::Max(1.0, $Size * 0.055)
            $markerOffset = [single]($Size * 0.265)
            $markerYs = @(
                [single]($center - $markerOffset)
                [single]($center + $markerOffset)
            )
            foreach ($markerY in $markerYs) {
                $graphics.FillRectangle(
                    $marker,
                    $center - ($markerSide / 2),
                    $markerY - ($markerSide / 2),
                    $markerSide,
                    $markerSide
                )
            }
        } finally {
            $marker.Dispose()
        }

        $stream = [System.IO.MemoryStream]::new()
        $bitmap.Save($stream, [System.Drawing.Imaging.ImageFormat]::Png)
        return ,$stream.ToArray()
    } finally {
        if ($stream) { $stream.Dispose() }
        if ($dot) { $dot.Dispose() }
        if ($signal) { $signal.Dispose() }
        if ($outline) { $outline.Dispose() }
        if ($graphics) { $graphics.Dispose() }
        if ($bitmap) { $bitmap.Dispose() }
    }
}

$frames = @()
foreach ($size in $sizes) {
    $frames += ,(New-PngFrame $size)
}

$stream = $null
$writer = $null
try {
    $stream = [System.IO.File]::Open(
        $resolvedOutput,
        [System.IO.FileMode]::Create,
        [System.IO.FileAccess]::Write,
        [System.IO.FileShare]::None
    )
    $writer = [System.IO.BinaryWriter]::new($stream)
    $writer.Write([uint16]0)
    $writer.Write([uint16]1)
    $writer.Write([uint16]$frames.Count)

    $offset = 6 + (16 * $frames.Count)
    for ($index = 0; $index -lt $frames.Count; $index++) {
        $size = $sizes[$index]
        $widthByte = if ($size -eq 256) { 0 } else { $size }
        $writer.Write([byte]$widthByte)
        $writer.Write([byte]$widthByte)
        $writer.Write([byte]0)
        $writer.Write([byte]0)
        $writer.Write([uint16]1)
        $writer.Write([uint16]32)
        $writer.Write([uint32]$frames[$index].Length)
        $writer.Write([uint32]$offset)
        $offset += $frames[$index].Length
    }
    foreach ($frame in $frames) {
        $writer.Write($frame)
    }
    $writer.Flush()
} finally {
    if ($writer) { $writer.Dispose() }
    elseif ($stream) { $stream.Dispose() }
}

Write-Output ("Wrote original Yujie beacon icon: {0} ({1} bytes, {2} frames)" -f $resolvedOutput, (Get-Item -LiteralPath $resolvedOutput).Length, $frames.Count)
