# MINE//SHIFT: BEYOND — POLYGLOT LAB
## Multi-Language Engineering Ecosystem & Verification Matrix

### 1. Mission & Scope
The Polyglot Laboratory is an independent engineering environment operating alongside the primary **MINE//SHIFT: BEYOND** native Rust application. It provides:
- **Independent Algorithmic Oracles:** Cross-checking MINECORE deduction rules, flood reveals, and topology math.
- **Contract Standardization:** Versioned puzzle schemas (`puzzle_contract_v1.schema.json`) ensuring interoperability.
- **Cross-Language Differential Testing:** Comparing deductions and topology across diverse compiler/runtime backends.
- **Fair Benchmarking:** Head-to-head performance measurements across equivalent workloads.
- **Strict Production Isolation:** Zero mandatory production runtime dependencies on secondary toolchains. The main Rust desktop game builds and ships completely independently.

---

### 2. Verified Language Matrix (12 Verified Ecosystems)

| Language | Verified Toolchain / Version | Role in Ecosystem | Deliverable Path |
| :--- | :--- | :--- | :--- |
| **Rust** | `rustc 1.90.0`, `cargo 1.90.0` | Canonical MINECORE engine & fixture generator | `engine/`, `src/` |
| **Python** | Python 3.15.0 (`uv run python`) | Constraint solver oracle & property verifier | `polyglot/python/` |
| **C++** | MSVC `cl.exe 14.44` (C++20) | High-performance deduction solver & benchmark | `polyglot/cpp/` |
| **C** | MSVC `cl.exe 14.44` (C11) | Minimal reference solver & memory oracle | `polyglot/c/` |
| **Zig** | `zig 0.17.0` | Bitset topology solver & zero-alloc bounds | `polyglot/zig/` |
| **Go** | `go 1.27.0` | Concurrent worker-pool batch verifier | `polyglot/go/` |
| **C#** | `csc.exe` (.NET Framework 4.8) | Matrix deduction engine & Windows diagnostics | `polyglot/csharp/` |
| **TypeScript / JS** | Node.js `v24.20.0` | Schema validator & contract interoperability | `polyglot/typescript/` |
| **Java** | Oracle JDK 17 (`javac` + `java`) | Puzzle serialization & constraint solver | `polyglot/java/` |
| **Lua** | Lua `5.4.6` | Scripted modifier sandbox & rules prototype | `polyglot/lua/` |
| **PowerShell** | PowerShell 5.1 Native | Build automation, test runner & release QA | `polyglot/powershell/` |
| **SQL** | SQLite Engine | Schema integrity, progression isolation checks | `polyglot/sql/` |

---

### 3. Shared Puzzle Contract (`puzzle_contract_v1`)
All polyglot tools communicate using versioned JSON fixtures adhering to `polyglot/contracts/puzzle_contract_v1.schema.json`.
- **Topologies Supported:** `square`, `hex` (axial coordinate system).
- **Modifier Specifications:** `standard`, `no_flags`, `limited_flags`, `extended_ring2`.
- **Truth Representation:** `mines` (secret ground truth for validation) vs `revealed` / `clues` (player-visible).
