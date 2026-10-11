# Automated QA Runtime Window Screenshot Capture for MINE//SHIFT: BEYOND
# Generates true GPU/DirectX frame captures directly from the application runtime.

param(
    [string]$TargetExe = "target\debug\mineshift.exe",
    [string]$OutputDir = "docs\qa"
)

if (-not (Test-Path $OutputDir)) {
    New-Item -ItemType Directory -Path $OutputDir -Force | Out-Null
}

Add-Type -AssemblyName System.Drawing

function Capture-Scenario {
    param(
        [string]$Name,
        [string]$ExtraArgs
    )

    $bmpPath = Join-Path $OutputDir "$Name.bmp"
    $pngPath = Join-Path $OutputDir "$Name.png"

    if (Test-Path $bmpPath) { Remove-Item $bmpPath -Force }
    if (Test-Path $pngPath) { Remove-Item $pngPath -Force }

    $argList = "$ExtraArgs --screenshot $bmpPath"
    Write-Host "Capturing scenario: $Name ($argList)..."

    $proc = Start-Process -FilePath $TargetExe -ArgumentList $argList -Wait -PassThru

    if (Test-Path $bmpPath) {
        $img = [System.Drawing.Image]::FromFile((Resolve-Path $bmpPath))
        $img.Save($pngPath, [System.Drawing.Imaging.ImageFormat]::Png)
        $img.Dispose()
        Remove-Item $bmpPath -Force
        $len = (Get-Item $pngPath).Length
        Write-Host "Success: $pngPath ($len bytes)"
    } else {
        Write-Error "Failed to generate screenshot for $Name"
    }
}

# Stop any running mineshift instances first
Get-Process mineshift -ErrorAction SilentlyContinue | Stop-Process -Force

# 1. P0: Settings -> Cursor (Cursor Size slider with full track and handle)
Capture-Scenario -Name "01_settings_cursor_sliders" -ExtraArgs "--screen settings-cursor"

# 2. P0: Settings -> Game Feel (Reveal Animation Speed & Screen Shake sliders)
Capture-Scenario -Name "02_settings_gamefeel_sliders" -ExtraArgs "--screen settings-gamefeel"

# 3. P1: Settings -> Game Feel with active Screen Shake impulse & simulation widget
Capture-Scenario -Name "03_settings_shake_simulation" -ExtraArgs "--screen settings-gamefeel --trigger-shake"

# 4. P0: Statistics screen (unclipped Career Overview, Records, Mode distribution)
Capture-Scenario -Name "04_statistics_full_unclipped" -ExtraArgs "--screen statistics"

# 5. P0: Achievements screen (unclipped achievement cards and category tabs)
Capture-Scenario -Name "05_achievements_full_unclipped" -ExtraArgs "--screen achievements"

# 6. P1: Settings Saved Feedback with clean vector checkmark icon
Capture-Scenario -Name "06_settings_saved_checkmark" -ExtraArgs "--screen settings-general --saved-feedback"

# 7a. Theme verification: Obsidian theme applied to UI & controls
Capture-Scenario -Name "07_theme_obsidian" -ExtraArgs "--screen settings-cursor --theme obsidian"

# 7b. Theme verification: Soft Circuit theme applied to UI & controls
Capture-Scenario -Name "07b_theme_soft_circuit" -ExtraArgs "--screen settings-cursor --theme soft_circuit"

# 8a. Gameplay verification: Hexagonal board (Obsidian)
Capture-Scenario -Name "08_gameplay_hex_board_obsidian" -ExtraArgs "--screen gameplay-hex --theme obsidian"

# 8b. Gameplay verification: Hexagonal board (Soft Circuit)
Capture-Scenario -Name "08b_gameplay_hex_board_soft_circuit" -ExtraArgs "--screen gameplay-hex --theme soft_circuit"

# 9a. Gameplay verification: Square board (Obsidian)
Capture-Scenario -Name "09_gameplay_square_board_obsidian" -ExtraArgs "--screen gameplay-square --theme obsidian"

# 9b. Gameplay verification: Square board (Soft Circuit)
Capture-Scenario -Name "09b_gameplay_square_board_soft_circuit" -ExtraArgs "--screen gameplay-square --theme soft_circuit"

Write-Host "All QA verification screenshots captured and converted to PNG in $OutputDir."
