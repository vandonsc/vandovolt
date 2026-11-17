Add-Type -AssemblyName System.Drawing

function Rounded-Path([float]$x, [float]$y, [float]$w, [float]$h, [float]$r) {
    $path = New-Object Drawing.Drawing2D.GraphicsPath
    $d = $r * 2
    $path.AddArc($x, $y, $d, $d, 180, 90); $path.AddArc($x + $w - $d, $y, $d, $d, 270, 90)
    $path.AddArc($x + $w - $d, $y + $h - $d, $d, $d, 0, 90); $path.AddArc($x, $y + $h - $d, $d, $d, 90, 90)
    $path.CloseFigure(); return $path
}

function Draw-Headset($g, [float]$x, [float]$y, [float]$size, $color, [float]$stroke) {
    $pen = New-Object Drawing.Pen $color, $stroke
    $pen.StartCap = $pen.EndCap = [Drawing.Drawing2D.LineCap]::Round
    $pen.LineJoin = [Drawing.Drawing2D.LineJoin]::Round
    $g.DrawArc($pen, $x + $size * .12, $y + $size * .10, $size * .76, $size * .72, 180, 180)
    $g.DrawArc($pen, $x + $size * .08, $y + $size * .48, $size * .28, $size * .40, 75, 210)
    $g.DrawArc($pen, $x + $size * .64, $y + $size * .48, $size * .28, $size * .40, -105, 210)
    $pen.Dispose()
}

function Draw-Mouse($g, [float]$x, [float]$y, [float]$size, $color, [float]$stroke) {
    $pen = New-Object Drawing.Pen $color, $stroke
    $pen.StartCap = $pen.EndCap = [Drawing.Drawing2D.LineCap]::Round
    $path = Rounded-Path ($x + $size * .25) ($y + $size * .05) ($size * .50) ($size * .88) ($size * .25)
    $g.DrawPath($pen, $path); $g.DrawLine($pen, $x + $size * .50, $y + $size * .06, $x + $size * .50, $y + $size * .36)
    $path.Dispose(); $pen.Dispose()
}

function Draw-BatteryPill($g, [float]$x, [float]$y, [float]$w, [float]$h, [float]$value, $track, $accent) {
    $trackPath = Rounded-Path $x $y $w $h ($h / 2)
    $g.FillPath((New-Object Drawing.SolidBrush $track), $trackPath)
    $fillPath = Rounded-Path $x $y ([Math]::Max($h, $w * $value / 100)) $h ($h / 2)
    $g.FillPath((New-Object Drawing.SolidBrush $accent), $fillPath)
    $trackPath.Dispose(); $fillPath.Dispose()
}

function Draw-TrayGlyph($g, [string]$kind, [float]$x, [float]$y, [float]$size, $color, [float]$percent, [string]$variant) {
    if ($variant -eq 'percent') {
        $font = New-Object Drawing.Font 'Segoe UI', ($size * .43), ([Drawing.FontStyle]::Bold), ([Drawing.GraphicsUnit]::Pixel)
        $format = New-Object Drawing.StringFormat; $format.Alignment = 'Center'; $format.LineAlignment = 'Center'
        $g.DrawString("$([int]$percent)%", $font, (New-Object Drawing.SolidBrush $color), (New-Object Drawing.RectangleF $x,$y,$size,$size), $format)
        $font.Dispose(); $format.Dispose(); return
    }
    $stroke = [Math]::Max(1.5, $size * .095)
    if ($kind -eq 'headset') { Draw-Headset $g $x $y $size $color $stroke } else { Draw-Mouse $g $x $y $size $color $stroke }
    if ($variant -eq 'compact') {
        $font = New-Object Drawing.Font 'Segoe UI', ($size * .26), ([Drawing.FontStyle]::Bold), ([Drawing.GraphicsUnit]::Pixel)
        $g.FillRectangle((New-Object Drawing.SolidBrush ([Drawing.Color]::FromArgb(220, 28, 28, 28))), $x + $size * .43, $y + $size * .63, $size * .57, $size * .35)
        $g.DrawString([string][int]$percent, $font, [Drawing.Brushes]::White, $x + $size * .46, $y + $size * .65); $font.Dispose()
    } elseif ($variant -eq 'indicator') {
        Draw-BatteryPill $g ($x + $size * .14) ($y + $size * .86) ($size * .72) ($size * .08) $percent ([Drawing.Color]::FromArgb(90, $color)) $color
    }
}

function Palette([string]$concept, [bool]$light) {
    if ($concept -eq 'A') {
        if ($light) { return @{ Canvas='#F3F3F3'; Panel='#FAFAFA'; Text='#1B1B1B'; Muted='#616161'; Border='#E1E1E1'; Surface='#F0F0F0'; Track='#E4E4E4'; Accent='#397FC3' } }
        return @{ Canvas='#171717'; Panel='#202020'; Text='#F5F5F5'; Muted='#B6B6B6'; Border='#3E3E3E'; Surface='#2C2C2C'; Track='#404040'; Accent='#5CA4E6' }
    }
    if ($concept -eq 'B') {
        if ($light) { return @{ Canvas='#F7F7F8'; Panel='#FFFFFF'; Text='#17171A'; Muted='#77777F'; Border='#EBEBED'; Surface='#FFFFFF'; Track='#ECECEF'; Accent='#7164E8' } }
        return @{ Canvas='#111113'; Panel='#19191C'; Text='#F7F7F8'; Muted='#92929B'; Border='#2B2B30'; Surface='#19191C'; Track='#303036'; Accent='#8B80F9' }
    }
    if ($light) { return @{ Canvas='#F5F5F5'; Panel='#FCFCFC'; Text='#202020'; Muted='#737373'; Border='#E5E5E5'; Surface='#FCFCFC'; Track='#E7E7E7'; Accent='#383838' } }
    return @{ Canvas='#151515'; Panel='#1C1C1C'; Text='#EEEEEE'; Muted='#999999'; Border='#303030'; Surface='#1C1C1C'; Track='#383838'; Accent='#D6D6D6' }
}

function C([string]$hex) { return [Drawing.ColorTranslator]::FromHtml($hex) }

function New-Concept([string]$concept, [bool]$light) {
    $p = Palette $concept $light; $canvas = C $p.Canvas; $panelColor = C $p.Panel; $text = C $p.Text
    $muted = C $p.Muted; $border = C $p.Border; $surface = C $p.Surface; $track = C $p.Track; $accent = C $p.Accent
    $bitmap = New-Object Drawing.Bitmap 1200,760,([Drawing.Imaging.PixelFormat]::Format32bppArgb)
    $g = [Drawing.Graphics]::FromImage($bitmap); $g.Clear($canvas)
    $g.SmoothingMode = [Drawing.Drawing2D.SmoothingMode]::AntiAlias; $g.PixelOffsetMode = 'HighQuality'
    $g.TextRenderingHint = [Drawing.Text.TextRenderingHint]::ClearTypeGridFit
    $titleFont = New-Object Drawing.Font 'Segoe UI',22,([Drawing.FontStyle]::Bold),([Drawing.GraphicsUnit]::Pixel)
    $labelFont = New-Object Drawing.Font 'Segoe UI',12,([Drawing.FontStyle]::Regular),([Drawing.GraphicsUnit]::Pixel)
    $nameFont = New-Object Drawing.Font 'Segoe UI',15,([Drawing.FontStyle]::Bold),([Drawing.GraphicsUnit]::Pixel)
    $statusFont = New-Object Drawing.Font 'Segoe UI',11,([Drawing.FontStyle]::Regular),([Drawing.GraphicsUnit]::Pixel)
    $percentFont = New-Object Drawing.Font 'Segoe UI',38,([Drawing.FontStyle]::Bold),([Drawing.GraphicsUnit]::Pixel)
    $conceptName = @{ A='A. Fluent Native'; B='B. Premium Minimal'; C='C. Monochrome' }[$concept]
    $g.DrawString($conceptName, $titleFont, (New-Object Drawing.SolidBrush $text), 64, 42)
    $g.DrawString($(if ($light) {'Tema claro'} else {'Tema escuro'}), $labelFont, (New-Object Drawing.SolidBrush $muted), 66, 78)

    $px=64; $py=128; $pw=520
    $ph = if ($concept -eq 'B') { 220 } elseif ($concept -eq 'C') { 190 } else { 250 }
    $radius = if ($concept -eq 'B') { 18 } elseif ($concept -eq 'C') { 10 } else { 14 }
    $panel = Rounded-Path $px $py $pw $ph $radius
    $g.FillPath((New-Object Drawing.SolidBrush $panelColor), $panel); $g.DrawPath((New-Object Drawing.Pen $border,1), $panel)
    if ($concept -eq 'A') {
        $tile=Rounded-Path 88 158 72 72 14; $g.FillPath((New-Object Drawing.SolidBrush $surface),$tile); Draw-Headset $g 103 171 42 $accent 3
        $nameX=184; $nameY=154
    } elseif ($concept -eq 'B') {
        Draw-Headset $g 92 161 58 $accent 3; $nameX=176; $nameY=153
    } else {
        Draw-Headset $g 94 166 48 $text 2.5; $nameX=168; $nameY=153
    }
    $g.DrawString('HyperX Cloud III Wireless',$nameFont,(New-Object Drawing.SolidBrush $text),$nameX,$nameY)
    $g.FillEllipse((New-Object Drawing.SolidBrush $accent),$nameX,$nameY+34,8,8)
    $g.DrawString('Conectado · descarregando',$statusFont,(New-Object Drawing.SolidBrush $muted),$nameX+14,$nameY+29)
    if ($concept -eq 'A') {
        $g.DrawString('68%',$percentFont,(New-Object Drawing.SolidBrush $text),$nameX,$nameY+66)
        Draw-BatteryPill $g $nameX ($nameY+132) 330 7 68 $track $accent
        $g.DrawString('Atualizado agora',$statusFont,(New-Object Drawing.SolidBrush $muted),$nameX,$nameY+158)
    } elseif ($concept -eq 'B') {
        $g.DrawString('68%',$percentFont,(New-Object Drawing.SolidBrush $text),455,$nameY+4)
        Draw-BatteryPill $g $nameX ($nameY+105) 350 5 68 $track $accent
        $g.DrawString('Atualizado agora',$statusFont,(New-Object Drawing.SolidBrush $muted),$nameX,$nameY+128)
    } else {
        $g.DrawString('68%',$percentFont,(New-Object Drawing.SolidBrush $text),455,$nameY+4)
        Draw-BatteryPill $g $nameX ($nameY+79) 350 4 68 $track $accent
        $g.DrawString('agora',$statusFont,(New-Object Drawing.SolidBrush $muted),$nameX,$nameY+98)
    }
    $panel.Dispose()

    $g.DrawString('Ícones do System Tray — tamanho real (16 px)',$labelFont,(New-Object Drawing.SolidBrush $muted),64,430)
    $taskbar = Rounded-Path 64 458 520 56 10; $g.FillPath((New-Object Drawing.SolidBrush $panelColor),$taskbar)
    Draw-TrayGlyph $g 'headset' 390 478 16 $text 68 'compact'; Draw-TrayGlyph $g 'mouse' 420 478 16 $text 68 'compact'
    Draw-TrayGlyph $g 'headset' 455 478 16 $text 68 'percent'; Draw-TrayGlyph $g 'mouse' 490 478 16 $text 68 'indicator'
    $taskbar.Dispose()

    $g.DrawString('Inspeção ampliada 8×',$labelFont,(New-Object Drawing.SolidBrush $muted),650,128)
    $cards=@(@('Headset',660,170,'headset','compact'),@('Mouse',890,170,'mouse','indicator'),@('Porcentagem',660,410,'headset','percent'))
    foreach($card in $cards) {
        $cardPath=Rounded-Path $card[1] $card[2] 190 190 16; $g.FillPath((New-Object Drawing.SolidBrush $panelColor),$cardPath)
        Draw-TrayGlyph $g $card[3] ($card[1]+31) ($card[2]+25) 128 $text 68 $card[4]
        $g.DrawString($card[0],$labelFont,(New-Object Drawing.SolidBrush $muted),$card[1]+16,$card[2]+158); $cardPath.Dispose()
    }
    $g.Dispose(); $directory=Join-Path $PSScriptRoot 'concepts'; New-Item -ItemType Directory -Force $directory | Out-Null
    $theme=if($light){'light'}else{'dark'}; $bitmap.Save((Join-Path $directory "concept-$($concept.ToLower())-$theme.png"),[Drawing.Imaging.ImageFormat]::Png); $bitmap.Dispose()
}

foreach($concept in 'A','B','C') { New-Concept $concept $true; New-Concept $concept $false }
