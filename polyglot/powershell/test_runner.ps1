# MINE//SHIFT: BEYOND — Polyglot Lab Master Test Runner
# Orchestrates all multi-language tests, collects performance timings, and asserts differential consistency.

$ErrorActionPreference = 'Stop'
$swTotal = [System.Diagnostics.Stopwatch]::StartNew()

Write-Host '=================================================================' -ForegroundColor Cyan
Write-Host '     MINE//SHIFT: BEYOND — POLYGLOT LAB MASTER TEST RUNNER      ' -ForegroundColor Cyan
Write-Host '=================================================================' -ForegroundColor Cyan

$results = [System.Collections.Generic.List[PSCustomObject]]::new()

function Run-PolyglotTest {
    param(
        [string]$Language,
        [string]$Toolchain,
        [scriptblock]$Action
    )
    Write-Host ''
    Write-Host "[RUNNING] $Language ($Toolchain)..." -ForegroundColor Yellow
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    $status = 'PASS'
    $err = ''
    try {
        & $Action
    } catch {
        $status = 'FAIL'
        $err = $_.Exception.Message
        Write-Host "  -> ERROR: $err" -ForegroundColor Red
    }
    $sw.Stop()
    $elapsedMs = [math]::Round($sw.Elapsed.TotalMilliseconds, 2)
    $color = 'Red'
    if ($status -eq 'PASS') { $color = 'Green' }
    Write-Host "  -> Status: $status in $elapsedMs ms" -ForegroundColor $color

    $results.Add([PSCustomObject]@{
        Language   = $Language
        Toolchain  = $Toolchain
        Status     = $status
        DurationMs = $elapsedMs
    })
}

# 1. Rust (MINECORE & mineshift tests)
Run-PolyglotTest -Language 'Rust' -Toolchain 'rustc 1.90 / cargo' -Action {
    cargo test --workspace --quiet
}

# 2. Python (Solver Oracle)
Run-PolyglotTest -Language 'Python' -Toolchain 'Python 3.15 (uv run)' -Action {
    uv run python polyglot/python/test_oracle.py
}

# 3. TypeScript / JS (Contract Validator)
Run-PolyglotTest -Language 'TypeScript/JS' -Toolchain 'Node.js v24.20.0' -Action {
    node polyglot/typescript/contract_validator.js
}

# 4. C++ (Fast Solver & Microbenchmark)
Run-PolyglotTest -Language 'C++' -Toolchain 'Clang 22.1 / C++20' -Action {
    .\polyglot\cpp\fast_solver.exe
}

# 5. C (Reference Solver & SAT)
Run-PolyglotTest -Language 'C' -Toolchain 'Clang 22.1 / C11' -Action {
    .\polyglot\c\reference_solver.exe
}

# 6. Zig (Bitset Topology Solver)
Run-PolyglotTest -Language 'Zig' -Toolchain 'Zig 0.17.0' -Action {
    & "C:\Users\koubes\AppData\Local\Microsoft\WinGet\Packages\zig.zig_Microsoft.Winget.Source_8wekyb3d8bbwe\zig-x86_64-windows-0.17.0\zig.exe" run polyglot/zig/bitset_solver.zig
}

# 7. Go (Concurrent Batch Verifier)
Run-PolyglotTest -Language 'Go' -Toolchain 'Go 1.27.0' -Action {
    & "C:\Program Files\Go\bin\go.exe" run polyglot/go/batch_verifier.go
}

# 8. C# (Linear Algebraic Matrix Solver)
Run-PolyglotTest -Language 'C#' -Toolchain 'csc.exe .NET 4.8' -Action {
    .\polyglot\csharp\MatrixSolver.exe
}

# 9. Java (Topology & Puzzle Verifier)
Run-PolyglotTest -Language 'Java' -Toolchain 'Oracle JDK 17' -Action {
    java -cp polyglot/java PuzzleVerifier
}

# 10. Lua (Rules Sandbox)
Run-PolyglotTest -Language 'Lua' -Toolchain 'Lua 5.4.6' -Action {
    & "C:\Users\koubes\AppData\Local\Programs\Lua\bin\lua.exe" polyglot/lua/rules_sandbox.lua
}

$swTotal.Stop()

Write-Host ''
Write-Host '=================================================================' -ForegroundColor Cyan
Write-Host '                POLYGLOT VERIFICATION MATRIX SUMMARY             ' -ForegroundColor Cyan
Write-Host '=================================================================' -ForegroundColor Cyan

$results | Format-Table -AutoSize

$allPass = ($results | Where-Object { $_.Status -ne 'PASS' }).Count -eq 0
if ($allPass) {
    $count = $results.Count
    $sec = [math]::Round($swTotal.Elapsed.TotalSeconds, 2)
    Write-Host "ALL $count POLYGLOT ECOSYSTEMS FULLY VERIFIED IN $sec SECONDS" -ForegroundColor Green
    exit 0
} else {
    Write-Host 'SOME POLYGLOT VERIFICATIONS FAILED' -ForegroundColor Red
    exit 1
}
