Add-Type -AssemblyName System.Drawing

function Add-RoundRect($path, [float]$x, [float]$y, [float]$w, [float]$h, [float]$r) {
    $d = $r * 2
    $path.AddArc($x, $y, $d, $d, 180, 90)
    $path.AddArc($x + $w - $d, $y, $d, $d, 270, 90)
    $path.AddArc($x + $w - $d, $y + $h - $d, $d, $d, 0, 90)
    $path.AddArc($x, $y + $h - $d, $d, $d, 90, 90)
    $path.CloseFigure()
}

function New-Preview([string]$name, [bool]$light, [bool]$mouse, [int]$percent, [bool]$charging) {
    $scale = 2
    $bitmap = New-Object Drawing.Bitmap (348 * $scale), (178 * $scale), ([Drawing.Imaging.PixelFormat]::Format32bppArgb)
    $g = [Drawing.Graphics]::FromImage($bitmap)
    $g.SmoothingMode = [Drawing.Drawing2D.SmoothingMode]::AntiAlias
    $g.TextRenderingHint = [Drawing.Text.TextRenderingHint]::ClearTypeGridFit
    $g.ScaleTransform($scale, $scale)

    if ($light) {
        $background = [Drawing.Color]::FromArgb(250, 250, 250); $foreground = [Drawing.Color]::FromArgb(32, 32, 32)
        $muted = [Drawing.Color]::FromArgb(97, 97, 97); $surface = [Drawing.Color]::FromArgb(238, 238, 238)
        $border = [Drawing.Color]::FromArgb(222, 222, 222); $track = [Drawing.Color]::FromArgb(227, 227, 227)
    } else {
        $background = [Drawing.Color]::FromArgb(32, 32, 32); $foreground = [Drawing.Color]::FromArgb(245, 245, 245)
        $muted = [Drawing.Color]::FromArgb(181, 181, 181); $surface = [Drawing.Color]::FromArgb(42, 42, 42)
        $border = [Drawing.Color]::FromArgb(69, 69, 69); $track = [Drawing.Color]::FromArgb(64, 64, 64)
    }
    $accent = if ($charging) { [Drawing.Color]::FromArgb(105, 173, 134) } elseif ($percent -le 10) { [Drawing.Color]::FromArgb(207, 104, 104) } else { [Drawing.Color]::FromArgb(72, 138, 197) }

    $panel = New-Object Drawing.Drawing2D.GraphicsPath; Add-RoundRect $panel 1 1 346 174 14
    $g.FillPath((New-Object Drawing.SolidBrush $background), $panel); $g.DrawPath((New-Object Drawing.Pen $border, 1), $panel)
    $tile = New-Object Drawing.Drawing2D.GraphicsPath; Add-RoundRect $tile 16 18 58 58 13
    $g.FillPath((New-Object Drawing.SolidBrush $surface), $tile)
    $pen = New-Object Drawing.Pen $accent, 3.4; $pen.StartCap = $pen.EndCap = [Drawing.Drawing2D.LineCap]::Round
    if ($mouse) {
        $body = New-Object Drawing.Drawing2D.GraphicsPath; Add-RoundRect $body 31 24 28 46 14
        $g.DrawPath($pen, $body); $g.DrawLine($pen, 45, 26, 45, 40)
        $device = 'NZXT Lift Elite Wireless'
    } else {
        $g.DrawArc($pen, 24, 25, 42, 34, 180, 180)
        $g.DrawArc($pen, 23, 44, 14, 22, 75, 210); $g.DrawArc($pen, 53, 44, 14, 22, -105, 210)
        $device = 'HyperX Cloud III Wireless'
    }
    $nameFont = New-Object Drawing.Font 'Segoe UI', 15.5, ([Drawing.FontStyle]::Bold), ([Drawing.GraphicsUnit]::Pixel)
    $statusFont = New-Object Drawing.Font 'Segoe UI', 12.5, ([Drawing.FontStyle]::Regular), ([Drawing.GraphicsUnit]::Pixel)
    $percentFont = New-Object Drawing.Font 'Segoe UI', 32, ([Drawing.FontStyle]::Bold), ([Drawing.GraphicsUnit]::Pixel)
    $smallFont = New-Object Drawing.Font 'Segoe UI', 12, ([Drawing.FontStyle]::Regular), ([Drawing.GraphicsUnit]::Pixel)
    $g.DrawString($device, $nameFont, (New-Object Drawing.SolidBrush $foreground), 88, 15)
    $g.FillEllipse((New-Object Drawing.SolidBrush $accent), 88, 46, 7, 7)
    $state = if ($charging) { 'Carregando' } elseif ($percent -le 10) { 'Bateria baixa' } else { 'Descarregando' }
    $g.DrawString($state, $statusFont, (New-Object Drawing.SolidBrush $muted), 101, 39)
    $g.DrawString("$percent%", $percentFont, (New-Object Drawing.SolidBrush $foreground), 88, 66)
    $bar = New-Object Drawing.Drawing2D.GraphicsPath; Add-RoundRect $bar 88 122 242 7 3.5
    $g.FillPath((New-Object Drawing.SolidBrush $track), $bar)
    $fill = New-Object Drawing.Drawing2D.GraphicsPath; Add-RoundRect $fill 88 122 (242 * $percent / 100) 7 3.5
    $g.FillPath((New-Object Drawing.SolidBrush $accent), $fill)
    $g.DrawString('Atualizado agora', $smallFont, (New-Object Drawing.SolidBrush $muted), 88, 145)

    $g.ResetTransform(); $g.Dispose()
    $path = Join-Path $PSScriptRoot "previews\$name.png"
    $bitmap.Save($path, [Drawing.Imaging.ImageFormat]::Png); $bitmap.Dispose()
}

$previewDirectory = Join-Path $PSScriptRoot 'previews'
New-Item -ItemType Directory -Force -Path $previewDirectory | Out-Null
New-Preview 'panel-light-headset-low' $true $false 8 $false
New-Preview 'panel-light-mouse-charging' $true $true 74 $true
New-Preview 'panel-dark-headset-charging' $false $false 86 $true
New-Preview 'panel-dark-mouse-normal' $false $true 63 $false
