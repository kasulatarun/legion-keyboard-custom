$ErrorActionPreference = "Stop"

Write-Host "=========================================="
Write-Host " Legion Keyboard Custom - Windows Release"
Write-Host "=========================================="

Write-Host "`n[1/3] Building the 'app' project in release mode..."
cargo build --release -p legion-kb-rgb

Write-Host "`n[2/3] Ensuring output directory exists..."
$DistDir = ".\dist\windows-release"
if (-not (Test-Path -Path $DistDir)) {
    New-Item -ItemType Directory -Path $DistDir | Out-Null
}

Write-Host "`n[3/3] Copying executable to $DistDir..."
$ExePath = ".\target\release\legion-kb-rgb.exe"

if (Test-Path -Path $ExePath) {
    Copy-Item -Path $ExePath -Destination "$DistDir\legion-kb-rgb.exe" -Force
    Write-Host "Successfully packaged legion-kb-rgb.exe to $DistDir"
    
    if (Test-Path -Path ".\README_RUN.txt") {
        Copy-Item -Path ".\README_RUN.txt" -Destination "$DistDir\README_RUN.txt" -Force
        Write-Host "Successfully packaged README_RUN.txt to $DistDir"
    } else {
        Write-Host "Warning: README_RUN.txt not found!" -ForegroundColor Yellow
    }
} else {
    Write-Host "Error: Could not find the compiled executable at $ExePath!" -ForegroundColor Red
    exit 1
}

Write-Host "`n=========================================="
Write-Host " Packaging Complete!"
Write-Host " You can find the standalone files in the 'dist\windows-release\' folder."
Write-Host "=========================================="
