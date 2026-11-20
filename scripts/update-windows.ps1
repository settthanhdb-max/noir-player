# NOIR Player - one-command update for Windows.
# Run from the repo root in PowerShell:  .\scripts\update-windows.ps1
#
# Stops running instances, pulls the latest code, builds the release exe,
# copies it over the installed copy (used by the Start menu and file
# associations), makes sure mpv's DLL is next to BOTH exes, and reports
# what to look for to confirm the update took effect.

$ErrorActionPreference = "Stop"

Write-Host "==> NOIR Player update" -ForegroundColor Cyan

# 0) Must run from the repo root.
if (-not (Test-Path "Cargo.toml")) {
    Write-Error "Run this script from the noir-player repo root (the folder containing Cargo.toml)."
}

# 1) Close any running instance (a leftover old one would also catch files
#    opened by double-click via the single-instance hand-off).
Get-Process -Name "noir-player" -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue

# 2) Pull the latest code.
$before = git rev-parse --short HEAD
git pull
$after = git rev-parse --short HEAD
if ($before -eq $after) {
    Write-Host "--> No new commits: already on $after" -ForegroundColor Yellow
} else {
    Write-Host "--> Updated: $before -> $after" -ForegroundColor Green
}

# 3) Build.
cargo build --release
if ($LASTEXITCODE -ne 0) {
    Write-Error "cargo build --release failed (see output above)."
}
$exe = "target\release\noir-player.exe"
if (-not (Test-Path $exe)) { Write-Error "Build produced no exe at $exe." }

# 4) Install libmpv next to the freshly built exe if we already have it in
#    the installed copy (so direct-running the repo exe plays real media too).
$instDir = "$env:LOCALAPPDATA\NOIR Player"
if (Test-Path $instDir) {
    Get-ChildItem $instDir -Filter "*mpv*.dll" -ErrorAction SilentlyContinue | ForEach-Object {
        $dst = "target\release\$($_.Name)"
        if (-not (Test-Path $dst)) {
            Copy-Item $_.FullName $dst
            Write-Host "--> Copied $($_.Name) next to the repo exe" -ForegroundColor Green
        }
    }
} else {
    Write-Host "--> No installed copy found at $instDir (installer not run yet)" -ForegroundColor Yellow
}

# 5) Replace the installed exe (Start-menu shortcut + file associations use it).
if (Test-Path $instDir) {
    Copy-Item $exe "$instDir\noir-player.exe" -Force
    Write-Host "--> Installed copy updated: $instDir\noir-player.exe" -ForegroundColor Green
}

# 6) Report what changed and what to check.
Write-Host ""
Write-Host "== Update complete ==" -ForegroundColor Cyan
Write-Host ("Repo exe   : {0}  ({1})" -f $exe, (Get-Item $exe).LastWriteTime)
if (Test-Path $instDir) {
    Write-Host ("Installed  : {0}  ({1})" -f "$instDir\noir-player.exe", (Get-Item "$instDir\noir-player.exe").LastWriteTime)
}
Write-Host ""
Write-Host "How you know it worked:" -ForegroundColor White
Write-Host "  1. Launch NOIR Player. The title bar reads  NOIR Player <version from Cargo.toml>."
Write-Host "  2. Play a video. A version banner shows briefly at start."
Write-Host "  3. Move the mouse over the video: the control bar appears immediately;"
Write-Host "     leave it still for 10 s: the bar hides."
Write-Host "  4. Right-click on the video: nothing pops up (by design)."
