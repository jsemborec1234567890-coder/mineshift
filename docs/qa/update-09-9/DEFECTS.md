# MINE//SHIFT: BEYOND — Update 09.9 Defect Log
## Discovered Defect Tracking, Triage, and Resolutions

**Date:** 2026-10-11  
**Project:** MINE//SHIFT: BEYOND  
**Version:** 0.9.9  
**Status:** All Discovered Defects Resolved  

---

### 1. Defect Classification Scheme
- **Critical (P0):** Application crash, memory corruption, unhandled panic, data corruption/loss, game engine desync.
- **Major (P1):** Core gameplay progression blocked, inaccessible UI screens, broken controls, complete visual failure.
- **Minor (P2):** Element clipping, text truncation, non-blocking layout overflows, contrast issues.
- **Cosmetic (P3):** Sub-pixel misalignment, styling inconsistencies, minor spacing/padding anomalies.

---

### 2. Defect Inventory & Resolution Summary

| ID | Severity | Category | Description | Root Cause | Status | Resolution / Commit |
|---|---|---|---|---|---|---|
| **DEF-01** | Major | QA Tooling | Headless/Agent GDI Screen Capture Failure | Windows desktop station restrictions prevent GDI `CopyFromScreen` in background agents. | **RESOLVED** | Implemented native WGPU/DirectX surface framebuffer capture via `--screenshot <path.bmp>` directly inside `eframe::App::ui`. |
| **DEF-02** | Minor | UI / Settings | Slider Track Low Contrast in Dark Themes | In Obsidian and Neon Arcade themes, slider inactive rail color matched card background. | **RESOLVED** | Enhanced slider visuals with explicit high-contrast rail fills (`theme.border_subtle`) and solid circular thumbs. |
| **DEF-03** | Minor | Responsive | Fixed Viewport CLI Configuration Missing | Automated testing could not adjust window resolution dynamically via CLI parameters. | **RESOLVED** | Added `--width <w>` and `--height <h>` arguments in `src/main.rs` that configure `NativeOptions::viewport.inner_size`. |
| **DEF-04** | Minor | UI / HUD | Board State Simulation Flags Missing for QA | Automated tests could not trigger victory/defeat/partial board states directly from CLI. | **RESOLVED** | Added `--game-state <fresh\|partial\|won\|lost>` in `src/main.rs` to allow instant, reliable visual verification of game outcomes. |
| **DEF-05** | Cosmetic | UI / Settings | Unicode Checkmark Missing Glyph Risk | Standard unicode checkmark `✓` in Save Feedback toasts risked tofu rendering without system font fallbacks. | **RESOLVED** | Replaced with vector-styled `[SAVED]` accent pill badge in `src/ui/settings.rs`. |

---

### 3. Open Defects
**None.** All identified defects have been patched, verified, and integrated into the master branch.
