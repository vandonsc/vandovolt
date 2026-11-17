Add-Type -AssemblyName System.Drawing

function Rounded-Path([float]$x, [float]$y, [float]$w, [float]$h, [float]$r) {
    $path = New-Object Drawing.Drawing2D.GraphicsPath; $d = $r * 2
    $path.AddArc($x,$y,$d,$d,180,90); $path.AddArc($x+$w-$d,$y,$d,$d,270,90)
    $path.AddArc($x+$w-$d,$y+$h-$d,$d,$d,0,90); $path.AddArc($x,$y+$h-$d,$d,$d,90,90)
    $path.CloseFigure(); return $path
}

function Draw-MouseState($g,[float]$x,[float]$y,[float]$size,[int]$level,[bool]$charging) {
    $outline=[Drawing.Color]::FromArgb(242,242,242)
    $fill=if($charging){[Drawing.Color]::FromArgb(105,173,134)}elseif($level -le 10){[Drawing.Color]::FromArgb(207,104,104)}else{[Drawing.Color]::FromArgb(198,198,198)}
    $stroke=[Math]::Max(1.7,$size*.105); $w=$size*.44; $h=$size*.92; $left=$x+($size-$w)/2; $top=$y+$size*.03
    $inner=Rounded-Path ($left+$size*.09) ($top+$size*.08) ($w-$size*.18) ($h-$size*.16) ($w*.4)
    $state=$g.Save(); $g.SetClip($inner); $innerTop=$top+$size*.08; $innerH=$h-$size*.16
    $g.FillRectangle((New-Object Drawing.SolidBrush $fill),$left,$innerTop+$innerH*(1-$level/100),$w,$innerH*$level/100)
    $g.Restore($state); $inner.Dispose()
    $pen=New-Object Drawing.Pen $outline,$stroke; $pen.StartCap=$pen.EndCap='Round'; $pen.LineJoin='Round'
    $body=Rounded-Path $left $top $w $h ($w/2); $g.DrawPath($pen,$body)
    $g.DrawLine($pen,$x+$size/2,$top+$stroke/2,$x+$size/2,$top+$size*.34)
    $body.Dispose(); $pen.Dispose()
}

function Draw-HeadsetState($g,[float]$x,[float]$y,[float]$size,[int]$level,[bool]$charging) {
    $outline=[Drawing.Color]::FromArgb(242,242,242)
    $fill=if($charging){[Drawing.Color]::FromArgb(105,173,134)}elseif($level -le 10){[Drawing.Color]::FromArgb(207,104,104)}else{[Drawing.Color]::FromArgb(198,198,198)}
    $stroke=[Math]::Max(1.7,$size*.105); $left=$x+$size*.06; $top=$y+$size*.04; $width=$size*.88; $height=$size*.92
    $cupTop=$top+$height*.52; $cupH=$height*.43; $cupW=$width*7/24; $gap=$size*.055
    foreach($cupX in @($left,($left+$width-$cupW))) {
        $inner=Rounded-Path ($cupX+$gap) ($cupTop+$gap) ($cupW-2*$gap) ($cupH-2*$gap) ($cupW*.25)
        $state=$g.Save(); $g.SetClip($inner); $ih=$cupH-2*$gap
        $g.FillRectangle((New-Object Drawing.SolidBrush $fill),$cupX,$cupTop+$gap+$ih*(1-$level/100),$cupW,$ih*$level/100)
        $g.Restore($state); $inner.Dispose()
    }
    $pen=New-Object Drawing.Pen $outline,$stroke; $pen.StartCap=$pen.EndCap='Round'; $pen.LineJoin='Round'
    $g.DrawArc($pen,$left,$top,$width,$height,180,180)
    $g.DrawArc($pen,$left-$stroke/4,$cupTop,$cupW+$stroke/2,$cupH,75,210)
    $g.DrawArc($pen,$left+$width-$cupW-$stroke/4,$cupTop,$cupW+$stroke/2,$cupH,-105,210)
    $pen.Dispose()
}

function New-StateSheet([string]$device) {
    $cases=@(@('Normal 10%',10,$false),@('Normal 50%',50,$false),@('Normal 90%',90,$false),
        @('Charging 25%',25,$true),@('Charging 50%',50,$true),@('Charging 84%',84,$true),@('Charging 100%',100,$true))
    $bitmap=New-Object Drawing.Bitmap 1190,250,([Drawing.Imaging.PixelFormat]::Format32bppArgb)
    $g=[Drawing.Graphics]::FromImage($bitmap); $g.Clear([Drawing.Color]::FromArgb(32,32,32)); $g.SmoothingMode='AntiAlias'
    $labelFont=New-Object Drawing.Font 'Segoe UI',12,([Drawing.FontStyle]::Regular),([Drawing.GraphicsUnit]::Pixel)
    for($i=0;$i -lt $cases.Count;$i++) {
        $x=25+$i*165
        if($device -eq 'mouse'){Draw-MouseState $g ($x+25) 22 112 $cases[$i][1] $cases[$i][2]}else{Draw-HeadsetState $g ($x+25) 22 112 $cases[$i][1] $cases[$i][2]}
        $g.DrawString($cases[$i][0],$labelFont,[Drawing.Brushes]::Gainsboro,$x+20,145)
        if($device -eq 'mouse'){Draw-MouseState $g ($x+68) 184 16 $cases[$i][1] $cases[$i][2]}else{Draw-HeadsetState $g ($x+68) 184 16 $cases[$i][1] $cases[$i][2]}
        $g.DrawString('16 px',$labelFont,[Drawing.Brushes]::DarkGray,$x+56,207)
    }
    $g.Dispose(); $dir=Join-Path $PSScriptRoot 'previews'; New-Item -ItemType Directory -Force $dir|Out-Null
    $bitmap.Save((Join-Path $dir "tray-$device-states.png"),[Drawing.Imaging.ImageFormat]::Png); $bitmap.Dispose()
}

New-StateSheet 'mouse'
New-StateSheet 'headset'
