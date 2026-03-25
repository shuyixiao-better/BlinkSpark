param(
    [string]$OutputDir = "assets/branding/generated"
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

Add-Type -AssemblyName System.Drawing

function New-RoundedRectPath {
    param(
        [float]$X,
        [float]$Y,
        [float]$Width,
        [float]$Height,
        [float]$Radius
    )

    $path = New-Object System.Drawing.Drawing2D.GraphicsPath
    $d = $Radius * 2

    $path.AddArc($X, $Y, $d, $d, 180, 90)
    $path.AddArc($X + $Width - $d, $Y, $d, $d, 270, 90)
    $path.AddArc($X + $Width - $d, $Y + $Height - $d, $d, $d, 0, 90)
    $path.AddArc($X, $Y + $Height - $d, $d, $d, 90, 90)
    $path.CloseFigure()

    return $path
}

function New-StarPoints {
    param(
        [float]$CenterX,
        [float]$CenterY,
        [float]$OuterRadius,
        [float]$InnerRadius,
        [int]$Points = 5
    )

    $result = New-Object 'System.Collections.Generic.List[System.Drawing.PointF]'
    $step = [Math]::PI / $Points
    $angle = -[Math]::PI / 2

    for ($i = 0; $i -lt $Points * 2; $i++) {
        $radius = if ($i % 2 -eq 0) { $OuterRadius } else { $InnerRadius }
        $x = $CenterX + [Math]::Cos($angle) * $radius
        $y = $CenterY + [Math]::Sin($angle) * $radius
        $result.Add([System.Drawing.PointF]::new([float]$x, [float]$y))
        $angle += $step
    }

    return $result.ToArray()
}

function Draw-BlinkSparkIcon {
    param(
        [int]$Size,
        [string]$OutputPath
    )

    $scale = $Size / 1024.0
    $bitmap = New-Object System.Drawing.Bitmap($Size, $Size, [System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
    $graphics = [System.Drawing.Graphics]::FromImage($bitmap)

    try {
        $graphics.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
        $graphics.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality
        $graphics.CompositingQuality = [System.Drawing.Drawing2D.CompositingQuality]::HighQuality
        $graphics.Clear([System.Drawing.Color]::Transparent)

        $bgRect = [System.Drawing.RectangleF]::new(56 * $scale, 56 * $scale, 912 * $scale, 912 * $scale)
        $bgPath = New-RoundedRectPath -X $bgRect.X -Y $bgRect.Y -Width $bgRect.Width -Height $bgRect.Height -Radius (220 * $scale)
        $bgBrush = New-Object System.Drawing.Drawing2D.LinearGradientBrush(
            [System.Drawing.PointF]::new(120 * $scale, 120 * $scale),
            [System.Drawing.PointF]::new(904 * $scale, 904 * $scale),
            [System.Drawing.Color]::FromArgb(11, 50, 88),
            [System.Drawing.Color]::FromArgb(14, 121, 178)
        )
        try {
            $graphics.FillPath($bgBrush, $bgPath)
        }
        finally {
            $bgBrush.Dispose()
            $bgPath.Dispose()
        }

        $ringRect = [System.Drawing.RectangleF]::new((512 - 336) * $scale, (512 - 336) * $scale, 672 * $scale, 672 * $scale)
        $ringPen = [System.Drawing.Pen]::new([System.Drawing.Color]::FromArgb(230, 214, 248, 255), [float](38 * $scale))
        $ringPen.StartCap = [System.Drawing.Drawing2D.LineCap]::Round
        $ringPen.EndCap = [System.Drawing.Drawing2D.LineCap]::Round
        try {
            $graphics.DrawArc($ringPen, $ringRect, -40, 150)
            $graphics.DrawArc($ringPen, $ringRect, 150, 110)
            $graphics.DrawArc($ringPen, $ringRect, 300, 50)
        }
        finally {
            $ringPen.Dispose()
        }

        $softRingPen = [System.Drawing.Pen]::new([System.Drawing.Color]::FromArgb(36, 232, 252, 255), [float](18 * $scale))
        try {
            $graphics.DrawEllipse($softRingPen, $ringRect)
        }
        finally {
            $softRingPen.Dispose()
        }

        $eyeBrush = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb(245, 234, 248, 255))
        try {
            $graphics.FillEllipse($eyeBrush, 224 * $scale, 364 * $scale, 576 * $scale, 296 * $scale)
        }
        finally {
            $eyeBrush.Dispose()
        }

        $upperPen = [System.Drawing.Pen]::new([System.Drawing.Color]::FromArgb(170, 255, 255, 255), [float](18 * $scale))
        $lowerPen = [System.Drawing.Pen]::new([System.Drawing.Color]::FromArgb(140, 207, 239, 255), [float](18 * $scale))
        $upperPen.StartCap = [System.Drawing.Drawing2D.LineCap]::Round
        $upperPen.EndCap = [System.Drawing.Drawing2D.LineCap]::Round
        $lowerPen.StartCap = [System.Drawing.Drawing2D.LineCap]::Round
        $lowerPen.EndCap = [System.Drawing.Drawing2D.LineCap]::Round
        try {
            $graphics.DrawBezier(
                $upperPen,
                [System.Drawing.PointF]::new(224 * $scale, 512 * $scale),
                [System.Drawing.PointF]::new(294 * $scale, 408 * $scale),
                [System.Drawing.PointF]::new(730 * $scale, 408 * $scale),
                [System.Drawing.PointF]::new(800 * $scale, 512 * $scale)
            )
            $graphics.DrawBezier(
                $lowerPen,
                [System.Drawing.PointF]::new(224 * $scale, 512 * $scale),
                [System.Drawing.PointF]::new(294 * $scale, 616 * $scale),
                [System.Drawing.PointF]::new(730 * $scale, 616 * $scale),
                [System.Drawing.PointF]::new(800 * $scale, 512 * $scale)
            )
        }
        finally {
            $upperPen.Dispose()
            $lowerPen.Dispose()
        }

        $irisRect = [System.Drawing.RectangleF]::new((512 - 128) * $scale, (512 - 128) * $scale, 256 * $scale, 256 * $scale)
        $irisPath = New-Object System.Drawing.Drawing2D.GraphicsPath
        $irisPath.AddEllipse($irisRect)
        $irisBrush = New-Object System.Drawing.Drawing2D.PathGradientBrush($irisPath)
        $irisBrush.CenterColor = [System.Drawing.Color]::FromArgb(197, 247, 255)
        $irisBrush.SurroundColors = @([System.Drawing.Color]::FromArgb(38, 183, 232))
        try {
            $graphics.FillPath($irisBrush, $irisPath)
        }
        finally {
            $irisBrush.Dispose()
            $irisPath.Dispose()
        }

        $pupilBrush = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb(8, 35, 61))
        try {
            $graphics.FillEllipse($pupilBrush, (512 - 58) * $scale, (512 - 58) * $scale, 116 * $scale, 116 * $scale)
        }
        finally {
            $pupilBrush.Dispose()
        }

        $highlightBrush = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb(241, 253, 255))
        try {
            $graphics.FillEllipse($highlightBrush, (477 - 16) * $scale, (476 - 16) * $scale, 32 * $scale, 32 * $scale)
        }
        finally {
            $highlightBrush.Dispose()
        }

        $starPoints = New-StarPoints -CenterX (650 * $scale) -CenterY (502 * $scale) -OuterRadius (76 * $scale) -InnerRadius (36 * $scale)
        $starBrush = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb(255, 255, 255))
        try {
            $graphics.FillPolygon($starBrush, $starPoints)
        }
        finally {
            $starBrush.Dispose()
        }

        $bitmap.Save($OutputPath, [System.Drawing.Imaging.ImageFormat]::Png)
    }
    finally {
        $graphics.Dispose()
        $bitmap.Dispose()
    }
}

function Write-IcoFromPng {
    param(
        [string]$PngPath,
        [string]$IcoPath
    )

    $pngBytes = [System.IO.File]::ReadAllBytes($PngPath)
    $stream = [System.IO.File]::Open($IcoPath, [System.IO.FileMode]::Create, [System.IO.FileAccess]::Write)

    try {
        $writer = New-Object System.IO.BinaryWriter($stream)
        try {
            # ICONDIR
            $writer.Write([UInt16]0)   # reserved
            $writer.Write([UInt16]1)   # type = icon
            $writer.Write([UInt16]1)   # image count

            # ICONDIRENTRY
            $writer.Write([Byte]0)     # width 256
            $writer.Write([Byte]0)     # height 256
            $writer.Write([Byte]0)     # palette colors
            $writer.Write([Byte]0)     # reserved
            $writer.Write([UInt16]1)   # color planes
            $writer.Write([UInt16]32)  # bits per pixel
            $writer.Write([UInt32]$pngBytes.Length)
            $writer.Write([UInt32]22)  # data offset

            $writer.Write($pngBytes)
        }
        finally {
            $writer.Dispose()
        }
    }
    finally {
        $stream.Dispose()
    }
}

$resolvedOutputDir = Join-Path (Get-Location) $OutputDir
[System.IO.Directory]::CreateDirectory($resolvedOutputDir) | Out-Null

$sizes = @(1024, 512, 256, 128, 64, 48, 32)
foreach ($size in $sizes) {
    $path = Join-Path $resolvedOutputDir ("blinkspark-icon-{0}.png" -f $size)
    Draw-BlinkSparkIcon -Size $size -OutputPath $path
    Write-Host "Generated $path"
}

$iconPng = Join-Path $resolvedOutputDir "blinkspark-icon-256.png"
$icoPath = Join-Path $resolvedOutputDir "blinkspark.ico"
Write-IcoFromPng -PngPath $iconPng -IcoPath $icoPath
Write-Host "Generated $icoPath"

$rootIcoPath = Join-Path (Get-Location) "assets/branding/blinkspark.ico"
Copy-Item $icoPath $rootIcoPath -Force
Write-Host "Generated $rootIcoPath"
