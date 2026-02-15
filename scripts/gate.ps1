$ErrorActionPreference = "Stop"

Write-Host "[gate] repo:" (Get-Location)
git status --porcelain | Out-Host

if (!(Test-Path "project.godot")) {
  throw "[gate] project.godot not found. Are you in the Godot project root?"
}

$godot = $env:GODOT

if ([string]::IsNullOrWhiteSpace($godot)) {
  $candidates = @(
    "C:\Program Files\Godot\Godot_v4.3-stable_win64.exe",
    "C:\Program Files\Godot\Godot_v4.2-stable_win64.exe",
    "C:\Program Files\Godot\Godot_v4.1-stable_win64.exe",
    "C:\Program Files\Godot\Godot_v4.0-stable_win64.exe",
    "C:\Program Files\Godot\Godot.exe",
    "$env:LOCALAPPDATA\Programs\Godot\Godot.exe"
  )
  foreach ($c in $candidates) {
    if (Test-Path $c) { $godot = $c; break }
  }
}

if ([string]::IsNullOrWhiteSpace($godot)) {
  $cmd = Get-Command godot -ErrorAction SilentlyContinue
  if ($cmd) { $godot = $cmd.Source }
}

if ([string]::IsNullOrWhiteSpace($godot) -or !(Test-Path $godot)) {
  throw "[gate] Godot executable not found. Set env var GODOT, e.g.
  `$env:GODOT='C:\Path\To\Godot_v4.3-stable_win64.exe'"
}

Write-Host "[gate] GODOT:" $godot
Write-Host "[gate] Godot headless smoke (import + quit)"
& $godot --headless --path . --quit | Out-Host

Write-Host "[gate] PASS"
