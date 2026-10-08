param(
  [string]$OutputDir = "visual-evidence\p1\contact-sheets"
)

$ErrorActionPreference = "Stop"
if (-not $IsWindows -and $PSVersionTable.PSEdition -eq "Core") {
  throw "Character contact sheets require Windows System.Drawing."
}
Add-Type -AssemblyName System.Drawing

$repo = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$destination = [IO.Path]::GetFullPath((Join-Path $repo $OutputDir))
New-Item -ItemType Directory -Path $destination -Force | Out-Null

$font = [Drawing.Font]::new("Segoe UI", 10, [Drawing.FontStyle]::Regular)
$heading = [Drawing.Font]::new("Segoe UI", 14, [Drawing.FontStyle]::Bold)
$ink = [Drawing.SolidBrush]::new([Drawing.Color]::FromArgb(40, 44, 56))
$paper = [Drawing.SolidBrush]::new([Drawing.Color]::FromArgb(246, 247, 250))
$checkerA = [Drawing.SolidBrush]::new([Drawing.Color]::FromArgb(232, 234, 239))
$checkerB = [Drawing.SolidBrush]::new([Drawing.Color]::White)

try {
  foreach ($id in @("byte", "mochi", "pip", "kiwi")) {
    $base = Join-Path $repo ("public\assets\characters\" + $id)
    $manifest = Get-Content (Join-Path $base "manifest.json") -Raw | ConvertFrom-Json
    $atlas = [Drawing.Bitmap]::new((Join-Path $base "atlas.png"))
    $sheet = [Drawing.Bitmap]::new(8 * 160, 4 * 169 + 42)
    $graphics = [Drawing.Graphics]::FromImage($sheet)
    try {
      $graphics.InterpolationMode = [Drawing.Drawing2D.InterpolationMode]::NearestNeighbor
      $graphics.PixelOffsetMode = [Drawing.Drawing2D.PixelOffsetMode]::Half
      $graphics.TextRenderingHint = [Drawing.Text.TextRenderingHint]::AntiAlias
      $graphics.FillRectangle($paper, 0, 0, $sheet.Width, $sheet.Height)
      $graphics.DrawString(($manifest.name + " — actual 64px production frames, 2× magnification"), $heading, $ink, 12, 9)
      $frames = @($manifest.frames.PSObject.Properties | Sort-Object { [int]$_.Value.index })
      foreach ($frame in $frames) {
        $index = [int]$frame.Value.index
        $col = $index % 8
        $row = [math]::Floor($index / 8)
        $x = $col * 160 + 14
        $y = $row * 169 + 45
        for ($cy = 0; $cy -lt 128; $cy += 16) {
          for ($cx = 0; $cx -lt 128; $cx += 16) {
            $brush = if ((($cx + $cy) / 16) % 2 -eq 0) { $checkerA } else { $checkerB }
            $graphics.FillRectangle($brush, $x + $cx, $y + $cy, 16, 16)
          }
        }
        $source = [Drawing.Rectangle]::new(
          ($index % [int]$manifest.atlas.columns) * [int]$manifest.atlas.frameWidth,
          [math]::Floor($index / [int]$manifest.atlas.columns) * [int]$manifest.atlas.frameHeight,
          [int]$manifest.atlas.frameWidth, [int]$manifest.atlas.frameHeight
        )
        $target = [Drawing.Rectangle]::new($x, $y, 128, 128)
        $graphics.DrawImage($atlas, $target, $source, [Drawing.GraphicsUnit]::Pixel)
        $graphics.DrawString(("{0:D2} {1}" -f $index, [string]$frame.Name), $font, $ink, $x, ($y + 132))
      }
      $file = Join-Path $destination ("p1-character-" + $id + ".png")
      $sheet.Save($file, [Drawing.Imaging.ImageFormat]::Png)
      Write-Host "Created $file"
    } finally {
      $graphics.Dispose()
      $sheet.Dispose()
      $atlas.Dispose()
    }
  }
} finally {
  foreach ($resource in @($font, $heading, $ink, $paper, $checkerA, $checkerB)) {
    $resource.Dispose()
  }
}
