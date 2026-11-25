# Creates 1,000 synthetic 300x200 JPEG thumbnails. The first four are magenta: the harness looks for that colour.
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing
$out = Join-Path $PSScriptRoot 'thumbs'
New-Item -ItemType Directory -Force $out | Out-Null
$rnd = [Random]::new(1)
$jpeg = [System.Drawing.Imaging.ImageFormat]::Jpeg
for ($i = 0; $i -lt 1000; $i++) {
    $bmp = [System.Drawing.Bitmap]::new(300, 200)
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    if ($i -lt 4) {
        $g.Clear([System.Drawing.Color]::Magenta)
    } else {
        $g.Clear([System.Drawing.Color]::FromArgb($rnd.Next(80), $rnd.Next(80), $rnd.Next(80)))
        for ($k = 0; $k -lt 60; $k++) {
            $brush = [System.Drawing.SolidBrush]::new([System.Drawing.Color]::FromArgb($rnd.Next(60, 200), $rnd.Next(256), $rnd.Next(256), $rnd.Next(256)))
            $g.FillEllipse($brush, $rnd.Next(-40, 300), $rnd.Next(-40, 200), $rnd.Next(10, 120), $rnd.Next(10, 120))
            $brush.Dispose()
        }
    }
    $g.Dispose()
    $bmp.Save((Join-Path $out "t$i.jpg"), $jpeg)
    $bmp.Dispose()
}
"Created $((Get-ChildItem $out -Filter *.jpg).Count) images in $out"
