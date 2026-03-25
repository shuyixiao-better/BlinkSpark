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

function New-EyePath {
    param(
        [float]$CenterX,
        [float]$CenterY,
        [float]$Width,
        [float]$Height
    )

    $path = New-Object System.Drawing.Drawing2D.GraphicsPath
    $halfW = $Width / 2
    $halfH = $Height / 2

    $left = [System.Drawing.PointF]::new($CenterX - $halfW, $CenterY)
    $right = [System.Drawing.PointF]::new($CenterX + $halfW, $CenterY)

    $path.StartFigure()
    $path.AddBezier(
        $left,
        [System.Drawing.PointF]::new($CenterX - $halfW * 0.55, $CenterY - $halfH),
        [System.Drawing.PointF]::new($CenterX + $halfW * 0.55, $CenterY - $halfH),
        $right
    )
    $path.AddBezier(
        $right,
        [System.Drawing.PointF]::new($CenterX + $halfW * 0.55, $CenterY + $halfH),
        [System.Drawing.PointF]::new($CenterX - $halfW * 0.55, $CenterY + $halfH),
        $left
    )
    $path.CloseFigure()

    return $path
}

function New-StarPoints {
    param(
        [float]$CenterX,
        [float]$CenterY,
        [float]$OuterRadius,
        [float]$InnerRadius,
        [int]$Points
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

        $bgPath = New-RoundedRectPath -X (64 * $scale) -Y (64 * $scale) -Width (896 * $scale) -Height (896 * $scale) -Radius (228 * $scale)
        $bgBrush = New-Object System.Drawing.Drawing2D.LinearGradientBrush(
            [System.Drawing.PointF]::new(120 * $scale, 120 * $scale),
            [System.Drawing.PointF]::new(920 * $scale, 920 * $scale),
            [System.Drawing.Color]::FromArgb(7, 24, 42),
            [System.Drawing.Color]::FromArgb(18, 81, 122)
        )
        try {
            $graphics.FillPath($bgBrush, $bgPath)
        }
        finally {
            $bgBrush.Dispose()
            $bgPath.Dispose()
        }

        $glowPath = New-Object System.Drawing.Drawing2D.GraphicsPath
        $glowPath.AddEllipse(242 * $scale, 250 * $scale, 540 * $scale, 540 * $scale)
        $glowBrush = New-Object System.Drawing.Drawing2D.PathGradientBrush($glowPath)
        $glowBrush.CenterColor = [System.Drawing.Color]::FromArgb(76, 95, 196, 240)
        $glowBrush.SurroundColors = @([System.Drawing.Color]::FromArgb(0, 95, 196, 240))
        try {
            $graphics.FillEllipse($glowBrush, 242 * $scale, 250 * $scale, 540 * $scale, 540 * $scale)
        }
        finally {
            $glowBrush.Dispose()
            $glowPath.Dispose()
        }

        $ringPen = [System.Drawing.Pen]::new([System.Drawing.Color]::FromArgb(120, 168, 233, 255), [float](22 * $scale))
        $ringPen.StartCap = [System.Drawing.Drawing2D.LineCap]::Round
        $ringPen.EndCap = [System.Drawing.Drawing2D.LineCap]::Round
        try {
            $ringRect = [System.Drawing.RectangleF]::new((512 - 316) * $scale, (512 - 316) * $scale, 632 * $scale, 632 * $scale)
            $graphics.DrawArc($ringPen, $ringRect, -30, 120)
            $graphics.DrawArc($ringPen, $ringRect, 160, 120)
            $graphics.DrawArc($ringPen, $ringRect, 315, 36)
        }
        finally {
            $ringPen.Dispose()
        }

        $eyePath = New-EyePath -CenterX (512 * $scale) -CenterY (532 * $scale) -Width (590 * $scale) -Height (250 * $scale)
        $eyeBrush = New-Object System.Drawing.Drawing2D.LinearGradientBrush(
            [System.Drawing.PointF]::new(512 * $scale, 406 * $scale),
            [System.Drawing.PointF]::new(512 * $scale, 658 * $scale),
            [System.Drawing.Color]::FromArgb(250, 252, 255),
            [System.Drawing.Color]::FromArgb(212, 232, 246)
        )
        try {
            $graphics.FillPath($eyeBrush, $eyePath)
        }
        finally {
            $eyeBrush.Dispose()
        }

        $eyeStroke = [System.Drawing.Pen]::new([System.Drawing.Color]::FromArgb(120, 214, 238, 255), [float](8 * $scale))
        try {
            $graphics.DrawPath($eyeStroke, $eyePath)
        }
        finally {
            $eyeStroke.Dispose()
            $eyePath.Dispose()
        }

        $lidPen = [System.Drawing.Pen]::new([System.Drawing.Color]::FromArgb(220, 122, 222, 255), [float](18 * $scale))
        $lidPen.StartCap = [System.Drawing.Drawing2D.LineCap]::Round
        $lidPen.EndCap = [System.Drawing.Drawing2D.LineCap]::Round
        try {
            $graphics.DrawBezier(
                $lidPen,
                [System.Drawing.PointF]::new(258 * $scale, 532 * $scale),
                [System.Drawing.PointF]::new(362 * $scale, 450 * $scale),
                [System.Drawing.PointF]::new(662 * $scale, 450 * $scale),
                [System.Drawing.PointF]::new(766 * $scale, 532 * $scale)
            )
        }
        finally {
            $lidPen.Dispose()
        }

        $irisPath = New-Object System.Drawing.Drawing2D.GraphicsPath
        $irisPath.AddEllipse((512 - 112) * $scale, (534 - 112) * $scale, 224 * $scale, 224 * $scale)
        $irisBrush = New-Object System.Drawing.Drawing2D.PathGradientBrush($irisPath)
        $irisBrush.CenterColor = [System.Drawing.Color]::FromArgb(171, 248, 255)
        $irisBrush.SurroundColors = @([System.Drawing.Color]::FromArgb(20, 153, 213))
        try {
            $graphics.FillPath($irisBrush, $irisPath)
        }
        finally {
            $irisBrush.Dispose()
            $irisPath.Dispose()
        }

        $pupilBrush = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb(8, 31, 52))
        try {
            $graphics.FillEllipse($pupilBrush, (512 - 50) * $scale, (534 - 50) * $scale, 100 * $scale, 100 * $scale)
        }
        finally {
            $pupilBrush.Dispose()
        }

        $highlightBrush = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb(232, 251, 255))
        try {
            $graphics.FillEllipse($highlightBrush, (476 - 14) * $scale, (499 - 14) * $scale, 28 * $scale, 28 * $scale)
        }
        finally {
            $highlightBrush.Dispose()
        }

        $sparkPoints = New-StarPoints -CenterX (676 * $scale) -CenterY (442 * $scale) -OuterRadius (30 * $scale) -InnerRadius (12 * $scale) -Points 4
        $sparkBrush = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb(255, 255, 236, 184))
        try {
            $graphics.FillPolygon($sparkBrush, $sparkPoints)
        }
        finally {
            $sparkBrush.Dispose()
        }

        $sparkDot = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb(230, 255, 204, 143))
        try {
            $graphics.FillEllipse($sparkDot, (720 - 7) * $scale, (482 - 7) * $scale, 14 * $scale, 14 * $scale)
        }
        finally {
            $sparkDot.Dispose()
        }

        $framePath = New-RoundedRectPath -X (64 * $scale) -Y (64 * $scale) -Width (896 * $scale) -Height (896 * $scale) -Radius (228 * $scale)
        $framePen = [System.Drawing.Pen]::new([System.Drawing.Color]::FromArgb(46, 199, 234, 255), [float](5 * $scale))
        try {
            $graphics.DrawPath($framePen, $framePath)
        }
        finally {
            $framePen.Dispose()
            $framePath.Dispose()
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
            $writer.Write([UInt16]0)
            $writer.Write([UInt16]1)
            $writer.Write([UInt16]1)
            $writer.Write([Byte]0)
            $writer.Write([Byte]0)
            $writer.Write([Byte]0)
            $writer.Write([Byte]0)
            $writer.Write([UInt16]1)
            $writer.Write([UInt16]32)
            $writer.Write([UInt32]$pngBytes.Length)
            $writer.Write([UInt32]22)
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
