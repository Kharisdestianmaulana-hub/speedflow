# SpeedFlow Release Packaging Script
# Generates two distribution options in dist/:
# 1. SpeedFlow-Setup-v0.1.0.exe (Standard Windows Setup Wizard with Desktop / Start Menu shortcut)
# 2. SpeedFlow-v0.1.0-Portable.zip (Standalone portable package for direct run without installation)

param(
    [string]$Version = ""
)

$ErrorActionPreference = "Stop"
$rootDir = Split-Path -Parent $PSScriptRoot
Set-Location $rootDir

if (-not $Version) {
    # Automatically extract version from Cargo.toml
    $cargoToml = Get-Content (Join-Path $rootDir "Cargo.toml") -Raw
    if ($cargoToml -match 'version\s*=\s*"([^"]+)"') {
        $Version = $matches[1]
    } else {
        $Version = "0.1.0"
    }
}

Write-Host "1. Building SpeedFlow release binary..."
cargo build --release

# --- A. Portable Package Generation ---
Write-Host "2. Packaging Portable version..."
$portableDir = Join-Path $rootDir "dist\SpeedFlow-v$Version-Portable"
if (Test-Path $portableDir) {
    Remove-Item -Path $portableDir -Recurse -Force
}
New-Item -ItemType Directory -Path $portableDir -Force | Out-Null

Copy-Item (Join-Path $rootDir "target\release\speedflow.exe") -Destination $portableDir -Force
Copy-Item (Join-Path $rootDir "README.md") -Destination $portableDir -Force
Copy-Item (Join-Path $rootDir "LICENSE") -Destination $portableDir -Force

$extDest = Join-Path $portableDir "extension"
New-Item -ItemType Directory -Path $extDest -Force | Out-Null
Copy-Item (Join-Path $rootDir "extension\manifest.json") -Destination $extDest -Force
Copy-Item (Join-Path $rootDir "extension\background.js") -Destination $extDest -Force
Copy-Item (Join-Path $rootDir "extension\popup.html") -Destination $extDest -Force
Copy-Item (Join-Path $rootDir "extension\popup.js") -Destination $extDest -Force
Copy-Item (Join-Path $rootDir "extension\README.md") -Destination $extDest -Force
Copy-Item (Join-Path $rootDir "extension\icons") -Destination $extDest -Recurse -Force

$portableZip = Join-Path $rootDir "dist\SpeedFlow-v$Version-Portable.zip"
if (Test-Path $portableZip) {
    Remove-Item $portableZip -Force
}
Compress-Archive -Path "$portableDir\*" -DestinationPath $portableZip -Force
Write-Host "Portable package created at: $portableZip"

# --- B. Standalone Extension Package ---
$extZip = Join-Path $rootDir "dist\speedflow-extension-v1.0.0.zip"
if (Test-Path $extZip) {
    Remove-Item $extZip -Force
}
Compress-Archive -Path "$extDest\*" -DestinationPath $extZip -Force
Write-Host "Standalone extension package created at: $extZip"

# --- C. Installer Setup Wizard Generation (Inno Setup) ---
Write-Host "3. Compiling Windows Installer Setup Wizard (Inno Setup)..."
$isccCandidates = @(
    "$env:LOCALAPPDATA\Programs\Inno Setup 6\ISCC.exe",
    "C:\Program Files (x86)\Inno Setup 6\ISCC.exe",
    "C:\Program Files\Inno Setup 6\ISCC.exe"
)

$isccPath = $null
foreach ($cand in $isccCandidates) {
    if (Test-Path $cand) {
        $isccPath = $cand
        break
    }
}

if ($isccPath) {
    $issScript = Join-Path $rootDir "installer\SpeedFlow.iss"
    & "$isccPath" "/DMyAppVersion=$Version" "$issScript"
    $setupExe = Join-Path $rootDir "dist\SpeedFlow-Setup-v$Version.exe"
    Write-Host "Installer package created at: $setupExe"
} else {
    Write-Warning "ISCC.exe not found. Install Inno Setup 6 to compile the installer."
}

Write-Host "All distribution packages ready in dist/ folder!"
