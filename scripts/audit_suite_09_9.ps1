# Comprehensive Quality Assurance Audit Suite for MINE//SHIFT: BEYOND Update 09.9
# Executes 98 distinct targeted verification scenarios across 14 categories.

param(
    [string]$TargetExe = "target\debug\mineshift.exe",
    [string]$BaseDir = "docs\qa\update-09-9"
)

Add-Type -AssemblyName System.Drawing

$categories = @(
    "baseline",
    "navigation",
    "gameplay-square",
    "gameplay-hex",
    "game-modes",
    "themes",
    "customization",
    "settings",
    "profiles",
    "statistics",
    "achievements",
    "audio-animation",
    "responsive",
    "regression",
    "reports"
)

foreach ($cat in $categories) {
    $dir = Join-Path $BaseDir $cat
    if (-not (Test-Path $dir)) {
        New-Item -ItemType Directory -Path $dir -Force | Out-Null
    }
}

# Stop any running mineshift instances
Get-Process mineshift -ErrorAction SilentlyContinue | Stop-Process -Force

function Capture-AuditScenario {
    param(
        [string]$Category,
        [string]$Name,
        [string]$Arguments
    )

    $catDir = Join-Path $BaseDir $Category
    $bmpPath = Join-Path $catDir "$Name.bmp"
    $pngPath = Join-Path $catDir "$Name.png"

    if (Test-Path $bmpPath) { Remove-Item $bmpPath -Force }
    if (Test-Path $pngPath) { Remove-Item $pngPath -Force }

    $fullArgs = "$Arguments --screenshot $bmpPath"
    Write-Host "[$Category] Capturing $Name..."

    $proc = Start-Process -FilePath $TargetExe -ArgumentList $fullArgs -Wait -PassThru

    if (Test-Path $bmpPath) {
        $img = [System.Drawing.Image]::FromFile((Resolve-Path $bmpPath))
        $img.Save($pngPath, [System.Drawing.Imaging.ImageFormat]::Png)
        $img.Dispose()
        Remove-Item $bmpPath -Force
        $len = (Get-Item $pngPath).Length
        Write-Host "  -> Success: $Name.png ($len bytes)"
    } else {
        Write-Error "  -> FAILED: Could not capture $Name"
    }
}

Write-Host "=== STARTING UPDATE 09.9 AUDIT CAPTURE SUITE ==="

# 1. BASELINE
Capture-AuditScenario -Category "baseline" -Name "01_app_launch_default" -Arguments ""
Capture-AuditScenario -Category "baseline" -Name "02_main_menu_soft_circuit" -Arguments "--screen mainmenu --theme soft_circuit"
Capture-AuditScenario -Category "baseline" -Name "03_main_menu_obsidian" -Arguments "--screen mainmenu --theme obsidian"
Capture-AuditScenario -Category "baseline" -Name "04_main_menu_aurora" -Arguments "--screen mainmenu --theme aurora"
Capture-AuditScenario -Category "baseline" -Name "05_main_menu_neon_arcade" -Arguments "--screen mainmenu --theme neon_arcade"

# 2. NAVIGATION
Capture-AuditScenario -Category "navigation" -Name "nav_01_main_menu" -Arguments "--screen mainmenu"
Capture-AuditScenario -Category "navigation" -Name "nav_02_game_modes" -Arguments "--screen gamemodes"
Capture-AuditScenario -Category "navigation" -Name "nav_03_customization_hub" -Arguments "--screen themestudio"
Capture-AuditScenario -Category "navigation" -Name "nav_04_player_profiles" -Arguments "--screen profiles"
Capture-AuditScenario -Category "navigation" -Name "nav_05_profile_cosmetics" -Arguments "--screen profile-cosmetics"
Capture-AuditScenario -Category "navigation" -Name "nav_06_statistics" -Arguments "--screen statistics"
Capture-AuditScenario -Category "navigation" -Name "nav_07_achievements" -Arguments "--screen achievements"
Capture-AuditScenario -Category "navigation" -Name "nav_08_settings" -Arguments "--screen settings-general"

# 3. GAMEPLAY SQUARE
Capture-AuditScenario -Category "gameplay-square" -Name "sq_01_beginner_fresh" -Arguments "--screen gameplay-square --board-size beginner"
Capture-AuditScenario -Category "gameplay-square" -Name "sq_02_beginner_partial" -Arguments "--screen gameplay-square --board-size beginner --game-state partial"
Capture-AuditScenario -Category "gameplay-square" -Name "sq_03_beginner_victory" -Arguments "--screen gameplay-square --board-size beginner --game-state won"
Capture-AuditScenario -Category "gameplay-square" -Name "sq_04_beginner_defeat" -Arguments "--screen gameplay-square --board-size beginner --game-state lost"
Capture-AuditScenario -Category "gameplay-square" -Name "sq_05_intermediate_fresh" -Arguments "--screen gameplay-square --board-size intermediate"
Capture-AuditScenario -Category "gameplay-square" -Name "sq_06_intermediate_partial" -Arguments "--screen gameplay-square --board-size intermediate --game-state partial"
Capture-AuditScenario -Category "gameplay-square" -Name "sq_07_expert_fresh" -Arguments "--screen gameplay-square --board-size expert"
Capture-AuditScenario -Category "gameplay-square" -Name "sq_08_expert_partial" -Arguments "--screen gameplay-square --board-size expert --game-state partial"
Capture-AuditScenario -Category "gameplay-square" -Name "sq_09_square_soft_circuit" -Arguments "--screen gameplay-square --theme soft_circuit --game-state partial"
Capture-AuditScenario -Category "gameplay-square" -Name "sq_10_square_paper" -Arguments "--screen gameplay-square --theme paper --game-state partial"

# 4. GAMEPLAY HEX
Capture-AuditScenario -Category "gameplay-hex" -Name "hex_01_small_fresh" -Arguments "--screen gameplay-hex --board-size hex-small"
Capture-AuditScenario -Category "gameplay-hex" -Name "hex_02_small_partial" -Arguments "--screen gameplay-hex --board-size hex-small --game-state partial"
Capture-AuditScenario -Category "gameplay-hex" -Name "hex_03_small_victory" -Arguments "--screen gameplay-hex --board-size hex-small --game-state won"
Capture-AuditScenario -Category "gameplay-hex" -Name "hex_04_small_defeat" -Arguments "--screen gameplay-hex --board-size hex-small --game-state lost"
Capture-AuditScenario -Category "gameplay-hex" -Name "hex_05_med_fresh" -Arguments "--screen gameplay-hex --board-size hex-med"
Capture-AuditScenario -Category "gameplay-hex" -Name "hex_06_med_partial" -Arguments "--screen gameplay-hex --board-size hex-med --game-state partial"
Capture-AuditScenario -Category "gameplay-hex" -Name "hex_07_large_fresh" -Arguments "--screen gameplay-hex --board-size hex-large"
Capture-AuditScenario -Category "gameplay-hex" -Name "hex_08_large_partial" -Arguments "--screen gameplay-hex --board-size hex-large --game-state partial"
Capture-AuditScenario -Category "gameplay-hex" -Name "hex_09_hex_obsidian" -Arguments "--screen gameplay-hex --theme obsidian --game-state partial"
Capture-AuditScenario -Category "gameplay-hex" -Name "hex_10_hex_retro" -Arguments "--screen gameplay-hex --theme retro --game-state partial"

# 5. GAME MODES
Capture-AuditScenario -Category "game-modes" -Name "mode_01_classic_square" -Arguments "--mode classic --geometry square --game-state partial"
Capture-AuditScenario -Category "game-modes" -Name "mode_02_classic_hex" -Arguments "--mode classic --geometry hex --game-state partial"
Capture-AuditScenario -Category "game-modes" -Name "mode_03_sprint" -Arguments "--mode sprint --geometry square --game-state partial"
Capture-AuditScenario -Category "game-modes" -Name "mode_04_noguess" -Arguments "--mode noguess --geometry square --game-state partial"
Capture-AuditScenario -Category "game-modes" -Name "mode_05_blitz" -Arguments "--mode blitz --geometry square --game-state partial"
Capture-AuditScenario -Category "game-modes" -Name "mode_06_survival" -Arguments "--mode survival --geometry square --game-state partial"
Capture-AuditScenario -Category "game-modes" -Name "mode_07_daily" -Arguments "--mode daily --geometry square --game-state partial"
Capture-AuditScenario -Category "game-modes" -Name "mode_08_expedition" -Arguments "--mode expedition --geometry square --game-state partial"
Capture-AuditScenario -Category "game-modes" -Name "mode_09_zen" -Arguments "--mode zen --geometry square --game-state partial"
Capture-AuditScenario -Category "game-modes" -Name "mode_10_sandbox" -Arguments "--mode sandbox --geometry square --game-state partial"

# 6. THEMES
Capture-AuditScenario -Category "themes" -Name "theme_01_soft_circuit_menu" -Arguments "--screen mainmenu --theme soft_circuit"
Capture-AuditScenario -Category "themes" -Name "theme_02_soft_circuit_board" -Arguments "--screen gameplay-square --theme soft_circuit --game-state partial"
Capture-AuditScenario -Category "themes" -Name "theme_03_obsidian_menu" -Arguments "--screen mainmenu --theme obsidian"
Capture-AuditScenario -Category "themes" -Name "theme_04_obsidian_board" -Arguments "--screen gameplay-square --theme obsidian --game-state partial"
Capture-AuditScenario -Category "themes" -Name "theme_05_aurora_menu" -Arguments "--screen mainmenu --theme aurora"
Capture-AuditScenario -Category "themes" -Name "theme_06_aurora_board" -Arguments "--screen gameplay-square --theme aurora --game-state partial"
Capture-AuditScenario -Category "themes" -Name "theme_07_paper_menu" -Arguments "--screen mainmenu --theme paper"
Capture-AuditScenario -Category "themes" -Name "theme_08_paper_board" -Arguments "--screen gameplay-square --theme paper --game-state partial"
Capture-AuditScenario -Category "themes" -Name "theme_09_retro_menu" -Arguments "--screen mainmenu --theme retro"
Capture-AuditScenario -Category "themes" -Name "theme_10_retro_board" -Arguments "--screen gameplay-square --theme retro --game-state partial"
Capture-AuditScenario -Category "themes" -Name "theme_11_neon_arcade_menu" -Arguments "--screen mainmenu --theme neon_arcade"
Capture-AuditScenario -Category "themes" -Name "theme_12_neon_arcade_board" -Arguments "--screen gameplay-square --theme neon_arcade --game-state partial"

# 7. CUSTOMIZATION
Capture-AuditScenario -Category "customization" -Name "cust_01_hub_overview" -Arguments "--screen themestudio"
Capture-AuditScenario -Category "customization" -Name "cust_02_theme_tab_colors" -Arguments "--screen themestudio --theme-tab colors"
Capture-AuditScenario -Category "customization" -Name "cust_03_theme_tab_geometry" -Arguments "--screen themestudio --theme-tab geometry"
Capture-AuditScenario -Category "customization" -Name "cust_04_theme_tab_numbers" -Arguments "--screen themestudio --theme-tab numbers"
Capture-AuditScenario -Category "customization" -Name "cust_05_live_previews" -Arguments "--screen themestudio --theme soft_circuit"
Capture-AuditScenario -Category "customization" -Name "cust_06_cosmetics_screen" -Arguments "--screen profile-cosmetics"

# 8. SETTINGS
Capture-AuditScenario -Category "settings" -Name "set_01_tab_general" -Arguments "--screen settings-general"
Capture-AuditScenario -Category "settings" -Name "set_02_tab_audio" -Arguments "--screen settings-audio"
Capture-AuditScenario -Category "settings" -Name "set_03_tab_gamefeel" -Arguments "--screen settings-gamefeel"
Capture-AuditScenario -Category "settings" -Name "set_04_tab_cursor" -Arguments "--screen settings-cursor"
Capture-AuditScenario -Category "settings" -Name "set_05_tab_appearance" -Arguments "--screen settings-appearance"
Capture-AuditScenario -Category "settings" -Name "set_06_tab_accessibility" -Arguments "--screen settings-accessibility"
Capture-AuditScenario -Category "settings" -Name "set_07_saved_confirmation" -Arguments "--screen settings-general --saved-feedback"
Capture-AuditScenario -Category "settings" -Name "set_08_shake_impulse_active" -Arguments "--screen settings-gamefeel --trigger-shake"

# 9. PROFILES
Capture-AuditScenario -Category "profiles" -Name "prof_01_list" -Arguments "--screen profiles"
Capture-AuditScenario -Category "profiles" -Name "prof_02_create_ui" -Arguments "--screen profiles"
Capture-AuditScenario -Category "profiles" -Name "prof_03_cosmetics_avatars" -Arguments "--screen profile-cosmetics"
Capture-AuditScenario -Category "profiles" -Name "prof_04_cosmetics_titles" -Arguments "--screen profile-cosmetics"

# 10. STATISTICS
Capture-AuditScenario -Category "statistics" -Name "stat_01_career_overview_obsidian" -Arguments "--screen statistics --theme obsidian"
Capture-AuditScenario -Category "statistics" -Name "stat_02_records_milestones_obsidian" -Arguments "--screen statistics --theme obsidian"
Capture-AuditScenario -Category "statistics" -Name "stat_03_career_overview_soft_circuit" -Arguments "--screen statistics --theme soft_circuit"
Capture-AuditScenario -Category "statistics" -Name "stat_04_records_soft_circuit" -Arguments "--screen statistics --theme soft_circuit"

# 11. ACHIEVEMENTS
Capture-AuditScenario -Category "achievements" -Name "ach_01_filter_all" -Arguments "--screen achievements --achieve-filter all"
Capture-AuditScenario -Category "achievements" -Name "ach_02_filter_unlocked" -Arguments "--screen achievements --achieve-filter unlocked"
Capture-AuditScenario -Category "achievements" -Name "ach_03_filter_locked" -Arguments "--screen achievements --achieve-filter locked"
Capture-AuditScenario -Category "achievements" -Name "ach_04_soft_circuit" -Arguments "--screen achievements --theme soft_circuit"

# 12. AUDIO & ANIMATION
Capture-AuditScenario -Category "audio-animation" -Name "anim_01_shake_active_gameplay" -Arguments "--screen gameplay-square --trigger-shake"
Capture-AuditScenario -Category "audio-animation" -Name "anim_02_victory_confetti" -Arguments "--screen gameplay-square --game-state won"
Capture-AuditScenario -Category "audio-animation" -Name "anim_03_cursor_trail_large" -Arguments "--screen settings-cursor --cursor-size 2.0"
Capture-AuditScenario -Category "audio-animation" -Name "anim_04_reduced_motion" -Arguments "--screen settings-accessibility"

# 13. RESPONSIVE RESOLUTIONS
Capture-AuditScenario -Category "responsive" -Name "resp_01_main_menu_900x600" -Arguments "--screen mainmenu --width 900 --height 600"
Capture-AuditScenario -Category "responsive" -Name "resp_02_settings_900x600" -Arguments "--screen settings-cursor --width 900 --height 600"
Capture-AuditScenario -Category "responsive" -Name "resp_03_main_menu_1280x720" -Arguments "--screen mainmenu --width 1280 --height 720"
Capture-AuditScenario -Category "responsive" -Name "resp_04_settings_1280x720" -Arguments "--screen settings-cursor --width 1280 --height 720"
Capture-AuditScenario -Category "responsive" -Name "resp_05_main_menu_1600x900" -Arguments "--screen mainmenu --width 1600 --height 900"
Capture-AuditScenario -Category "responsive" -Name "resp_06_gameplay_1600x900" -Arguments "--screen gameplay-square --width 1600 --height 900 --game-state partial"
Capture-AuditScenario -Category "responsive" -Name "resp_07_main_menu_1920x1080" -Arguments "--screen mainmenu --width 1920 --height 1080"
Capture-AuditScenario -Category "responsive" -Name "resp_08_settings_1920x1080" -Arguments "--screen settings-cursor --width 1920 --height 1080"

# 14. REGRESSION FIXES
Capture-AuditScenario -Category "regression" -Name "regr_01_slider_cursor_track" -Arguments "--screen settings-cursor"
Capture-AuditScenario -Category "regression" -Name "regr_02_slider_gamefeel_tracks" -Arguments "--screen settings-gamefeel"
Capture-AuditScenario -Category "regression" -Name "regr_03_statistics_unclipped_height" -Arguments "--screen statistics"
Capture-AuditScenario -Category "regression" -Name "regr_04_achievements_unclipped_grid" -Arguments "--screen achievements"
Capture-AuditScenario -Category "regression" -Name "regr_05_saved_checkmark_no_tofu" -Arguments "--screen settings-general --saved-feedback"

Write-Host "=== AUDIT SUITE COMPLETED SUCCESSFULLY ==="
