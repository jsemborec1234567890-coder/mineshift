# MINE//SHIFT: BEYOND — Windows 1.0 Release Engineering Automation
# Builds optimized binary, produces portable ZIP, compiles native Inno Setup installer, and hashes artifacts.

$ErrorActionPreference = 'Stop'
$sw = [System.Diagnostics.Stopwatch]::StartNew()

Write-Host '=================================================================' -ForegroundColor Cyan
Write-Host '     MINE//SHIFT: BEYOND — WINDOWS 1.0 RELEASE BUILD PIPELINE   ' -ForegroundColor Cyan
Write-Host '=================================================================' -ForegroundColor Cyan

# 1. Workspace Test Verification
Write-Host ''
Write-Host '[1/6] Running full test verification...' -ForegroundColor Yellow
cargo test --workspace --quiet
Write-Host '  -> All workspace tests passed.' -ForegroundColor Green

# 2. Optimized Release Compilation
Write-Host ''
Write-Host '[2/6] Compiling optimized release binary...' -ForegroundColor Yellow
cargo build --release
$binPath = 'target\release\mineshift.exe'
if (-not (Test-Path $binPath)) {
    throw "Release binary not found at $binPath"
}
$binSizeMb = [math]::Round((Get-Item $binPath).Length / 1MB, 2)
Write-Host "  -> Built target\release\mineshift.exe ($binSizeMb" + " MB)" -ForegroundColor Green

# 3. Clean and Prepare Output Directory
Write-Host ''
Write-Host '[3/6] Setting up distribution structure...' -ForegroundColor Yellow
$distDir = 'dist'
$portableDir = "$distDir\portable\MineShift-v1.0.0-Portable"
$installerDir = "$distDir\installer"

if (Test-Path $distDir) {
    Remove-Item -Path $distDir -Recurse -Force
}
New-Item -ItemType Directory -Path $portableDir -Force | Out-Null
New-Item -ItemType Directory -Path $installerDir -Force | Out-Null

# 4. Package Portable Windows Edition
Write-Host ''
Write-Host '[4/6] Packaging Portable Windows Edition...' -ForegroundColor Yellow
Copy-Item $binPath "$portableDir\mineshift.exe" -Force
Copy-Item 'assets' "$portableDir\assets" -Recurse -Force
Copy-Item 'LICENSE.txt' "$portableDir\LICENSE.txt" -Force

$readmeText = @"
MINE//SHIFT: BEYOND — Windows 1.0 Portable Edition
==================================================
Quick Start:
Simply run mineshift.exe to play.

Requirements:
- Windows 10 or Windows 11 (64-bit)
- No external runtimes or dependencies required.

User Data:
All player profiles, progression, achievements, and settings are saved to:
%APPDATA%\Antigravity\MineShift\data\mineshift.db
"@
Set-Content -Path "$portableDir\README.txt" -Value $readmeText

$portableZip = "$distDir\MineShift-v1.0.0-Windows-Portable.zip"
Compress-Archive -Path "$portableDir\*" -DestinationPath $portableZip -Force
$zipSizeMb = [math]::Round((Get-Item $portableZip).Length / 1MB, 2)
Write-Host "  -> Generated $portableZip ($zipSizeMb" + " MB)" -ForegroundColor Green

# 5. Compile Windows Installer with Inno Setup
Write-Host ''
Write-Host '[5/6] Compiling Windows Installer (Inno Setup)...' -ForegroundColor Yellow
$isccPath = 'C:\Users\koubes\AppData\Local\Programs\Inno Setup 6\iscc.exe'
if (-not (Test-Path $isccPath)) {
    throw "Inno Setup compiler not found at $isccPath"
}
& $isccPath '/Q' 'installer\mineshift_installer.iss'
$setupExe = "$installerDir\MineShift-v1.0.0-Setup.exe"
if (-not (Test-Path $setupExe)) {
    throw "Installer binary not generated at $setupExe"
}
$setupSizeMb = [math]::Round((Get-Item $setupExe).Length / 1MB, 2)
Write-Host "  -> Generated $setupExe ($setupSizeMb" + " MB)" -ForegroundColor Green

# 6. Checksums and Release Manifest
Write-Host ''
Write-Host '[6/6] Generating SHA-256 Checksums and Release Manifest...' -ForegroundColor Yellow
$zipHash = (Get-FileHash -Path $portableZip -Algorithm SHA256).Hash
$setupHash = (Get-FileHash -Path $setupExe -Algorithm SHA256).Hash
$binHash = (Get-FileHash -Path $binPath -Algorithm SHA256).Hash

$checksums = @"
$binHash  target/release/mineshift.exe
$zipHash  MineShift-v1.0.0-Windows-Portable.zip
$setupHash  MineShift-v1.0.0-Setup.exe
"@
Set-Content -Path "$distDir\SHA256SUMS.txt" -Value $checksums

$gitCommit = (git rev-parse HEAD).Trim()
$manifest = [ordered]@{
    product_name      = 'MINE//SHIFT: BEYOND'
    version           = '1.0.0'
    target_platform   = 'Windows 10/11 x86_64'
    git_commit        = $gitCommit
    release_date      = (Get-Date -Format 'yyyy-MM-ddTHH:mm:sszzz')
    artifacts         = @(
        [ordered]@{
            type     = 'installer'
            file     = 'MineShift-v1.0.0-Setup.exe'
            size_mb  = $setupSizeMb
            sha256   = $setupHash
        },
        [ordered]@{
            type     = 'portable_zip'
            file     = 'MineShift-v1.0.0-Windows-Portable.zip'
            size_mb  = $zipSizeMb
            sha256   = $zipHash
        }
    )
}
$manifest | ConvertTo-Json -Depth 4 | Set-Content -Path "$distDir\release_manifest.json"

$sw.Stop()
$totalSec = [math]::Round($sw.Elapsed.TotalSeconds, 2)

Write-Host ''
Write-Host '=================================================================' -ForegroundColor Cyan
Write-Host '        WINDOWS 1.0 RELEASE DISTRIBUTION PIPELINE SUCCESS        ' -ForegroundColor Green
Write-Host '=================================================================' -ForegroundColor Cyan
Write-Host "  Installer: $setupExe ($setupSizeMb" + " MB)"
Write-Host "  Portable:  $portableZip ($zipSizeMb" + " MB)"
Write-Host "  Manifest:  $distDir\release_manifest.json"
Write-Host "  Total Duration: $totalSec seconds"
Write-Host ''
