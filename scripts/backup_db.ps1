$paths = @(
    "$env:APPDATA\Antigravity\MineShift\data\mineshift.db",
    "$env:LOCALAPPDATA\Antigravity\MineShift\data\mineshift.db",
    "$env:APPDATA\Antigravity\MineShift\mineshift.db",
    "$env:LOCALAPPDATA\Antigravity\MineShift\mineshift.db",
    "$env:APPDATA\com.Antigravity.MineShift\data\mineshift.db",
    "mineshift.db"
)

$found = $false
foreach ($p in $paths) {
    if (Test-Path $p) {
        $backup = "$p.backup_update_09_9"
        Copy-Item $p $backup -Force
        Write-Host "Backed up production DB at '$p' to '$backup'"
        $found = $true
    }
}

if (-not $found) {
    # Search AppData for mineshift.db
    $foundFiles = Get-ChildItem -Path "$env:APPDATA" -Filter "mineshift.db" -Recurse -ErrorAction SilentlyContinue
    foreach ($f in $foundFiles) {
        $backup = "$($f.FullName).backup_update_09_9"
        Copy-Item $f.FullName $backup -Force
        Write-Host "Found and backed up: $($f.FullName) to $backup"
    }
}
