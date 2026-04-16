$ErrorActionPreference = "Stop"

$workspaceDir = "d:\Projects\Experiments\legion-keyboard-custom\legion-stealth"
Set-Location $workspaceDir

Write-Host "Cleaning up old React rubbish..."
if (Test-Path "$workspaceDir\src\components") { Remove-Item -Recurse -Force "$workspaceDir\src\components" }
if (Test-Path "$workspaceDir\src\hooks") { Remove-Item -Recurse -Force "$workspaceDir\src\hooks" }
if (Test-Path "$workspaceDir\src\App.jsx") { Remove-Item -Force "$workspaceDir\src\App.jsx" }
if (Test-Path "$workspaceDir\src\main.jsx") { Remove-Item -Force "$workspaceDir\src\main.jsx" }
if (Test-Path "$workspaceDir\src\index.css") { Remove-Item -Force "$workspaceDir\src\index.css" }
if (Test-Path "$workspaceDir\src\App.css") { Remove-Item -Force "$workspaceDir\src\App.css" }

Write-Host "Installing fresh dependencies (Vanilla UI)..."
npm install

Write-Host "Building Tauri app into a lightweight EXE..."
npm run tauri build

Write-Host "Build complete! You can find your fresh EXE in legon-stealth/src-tauri/target/release/legion-stealth.exe"
