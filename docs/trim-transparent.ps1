param(
  [Parameter(Mandatory = $true)]
  [string] $Path
)

# Crop a PNG to its non-transparent bounds. Used after a Chrome capture with
# --default-background-color=00000000, which renders the page transparent.

Add-Type -AssemblyName System.Drawing

$file = (Resolve-Path $Path).Path
$bmp = [System.Drawing.Bitmap]::FromFile($file)
try {
  $midX = [int]($bmp.Width / 2)
  $midY = [int]($bmp.Height / 2)

  $left = 0
  while ($left -lt $bmp.Width -and $bmp.GetPixel($left, $midY).A -eq 0) { $left++ }
  $right = $bmp.Width - 1
  while ($right -ge 0 -and $bmp.GetPixel($right, $midY).A -eq 0) { $right-- }
  $top = 0
  while ($top -lt $bmp.Height -and $bmp.GetPixel($midX, $top).A -eq 0) { $top++ }
  $bottom = $bmp.Height - 1
  while ($bottom -ge 0 -and $bmp.GetPixel($midX, $bottom).A -eq 0) { $bottom-- }

  if ($right -lt $left -or $bottom -lt $top) {
    throw "No opaque pixels found in $file"
  }

  $rect = [System.Drawing.Rectangle]::new($left, $top, $right - $left + 1, $bottom - $top + 1)
  $crop = $bmp.Clone($rect, [System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
  try {
    $tmp = "$file.tmp"
    $crop.Save($tmp, [System.Drawing.Imaging.ImageFormat]::Png)
  }
  finally {
    $crop.Dispose()
  }
}
finally {
  $bmp.Dispose()
}

Move-Item -Force $tmp $file
Write-Output "cropped $Path to $($rect.Width)x$($rect.Height)"
