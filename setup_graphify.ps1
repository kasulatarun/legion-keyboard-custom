$ErrorActionPreference = "Stop"

Write-Host "Cloning Graphify Repository..."
if (-not (Test-Path "graphify")) {
    git clone https://github.com/safishamsi/graphify.git
} else {
    Write-Host "Directory 'graphify' already exists, skipping clone."
}

Write-Host "Installing graphifyy from PyPI..."
pip install graphifyy

Write-Host "Configuring Graphify for Google Antigravity..."
graphify antigravity install

Write-Host "Graphify installed successfully! You can now run 'graphify .' in your workspace to generate the knowledge graph."
