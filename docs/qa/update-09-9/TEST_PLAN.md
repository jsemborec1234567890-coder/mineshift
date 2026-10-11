# MINE//SHIFT: BEYOND — Update 09.9 Quality Assurance Test Plan
## Project: Zero Surprises — Full-Game Visual Audit & Stabilization

**Date:** 2026-10-11  
**Project:** MINE//SHIFT: BEYOND  
**Target Architecture:** Native Windows x64 (Rust 1.85+, eframe 0.31, egui, SQLite)  
**Phase:** Final Quality Gate before Update 10 (Gameplay Evolution)  
**Status:** Approved & Active  

---

### 1. Executive Summary & Objectives
The purpose of Update 09.9 ("Project Zero Surprises") is to execute the most thorough, evidence-backed quality assurance operation in the history of MINE//SHIFT: BEYOND. This audit serves as the unconditional quality gate prior to Update 10.

Key objectives:
1. **Zero Visual Surprises:** Verify that every screen, modal, grid, slider, button, and HUD element renders cleanly without clipping, overlap, text truncation, or missing glyphs ("tofu").
2. **Comprehensive Coverage:** Validate all 9 game modes (Classic, Sprint, No Guess, Blitz, Survival, Daily Challenge, Expedition, Zen, Sandbox) in both Square and Hexagonal geometries.
3. **Theme & Contrast Integrity:** Confirm all 6 official theme presets (Soft Circuit, Obsidian, Aurora, Paper, Retro, Neon Arcade) maintain readable contrast and proper color hierarchy across both menu interfaces and active gameplay boards.
4. **Customization & Progression Verification:** Inspect the Theme Studio, Profile Cosmetics, Career Statistics, and Achievements systems to ensure all controls operate smoothly and statistics display without clipping.
5. **Responsive Scaling:** Test multi-resolution viewports from 900x600 (compact window) up to 1920x1080 (standard FHD desktop) to ensure responsive layout adaptation.
6. **Regression Prevention:** Confirm that all previous GUI defect repairs (slider thumb clipping, statistics panel height collapses, save confirmation checkmarks) remain stable and unregressed.

---

### 2. Scope & Boundaries
- **In Scope:**
  - Complete application UI inventory across 14 verification categories (98 targeted scenarios).
  - Headless and interactive native WGPU/DirectX surface framebuffer capture via application CLI flags.
  - Lossless visual asset capture (PNG format) and automated verification.
  - Visual analysis using Antigravity visual inspection tools.
  - Defect isolation and surgical repair of UI, styling, and layout logic.
  - Regression validation suite.
- **Out of Scope:**
  - Update 10 feature development (new modes, online services, modding SDK).
  - Modifications to MINECORE puzzle generation or solver algorithms (unless a critical bug is discovered).
  - Destructive modifications or resets of the production SQLite database (`mineshift.db`).

---

### 3. Verification Categories & Test Matrix (98 Scenarios)

| # | Category | Identifier Prefix | Count | Target Scope |
|---|---|---|---|---|
| 1 | Baseline Launch | `01_` - `05_` | 5 | Clean startup, default profile loading, main menu initial state across themes |
| 2 | Screen Navigation | `nav_01` - `nav_08` | 8 | All primary screens (Menu, Modes, Studio, Profiles, Cosmetics, Stats, Achievements, Settings) |
| 3 | Gameplay: Square | `sq_01` - `sq_10` | 10 | Square board states (Fresh, Partial, Won, Lost), Beginner/Intermediate/Expert, themes |
| 4 | Gameplay: Hex | `hex_01` - `hex_10` | 10 | Hexagonal board states (Fresh, Partial, Won, Lost), Small/Medium/Large, themes |
| 5 | Game Modes | `mode_01` - `mode_10` | 10 | All 9 game modes (Classic Square/Hex, Sprint, No Guess, Blitz, Survival, Daily, Expedition, Zen, Sandbox) |
| 6 | Theme Presets | `theme_01` - `theme_12` | 12 | Menus and active boards under Soft Circuit, Obsidian, Aurora, Paper, Retro, Neon Arcade |
| 7 | Customization Hub | `cust_01` - `cust_06` | 6 | Theme Studio tabs (Colors, Geometry, Numbers), live previews, profile cosmetics |
| 8 | Settings Subsystems | `set_01` - `set_08` | 8 | General, Audio, Game Feel, Cursor, Appearance, Accessibility tabs, feedback toasts, shake impulse |
| 9 | Profiles & Progression | `prof_01` - `prof_04` | 4 | Profile management list, creation UI, avatar selector, title selector |
| 10 | Statistics & Records | `stat_01` - `stat_04` | 4 | Career overview, records, milestones in Obsidian and Soft Circuit themes |
| 11 | Achievements System | `ach_01` - `ach_04` | 4 | All achievements filter, Unlocked filter, Locked filter, theme presentation |
| 12 | Audio & Animation | `anim_01` - `anim_04` | 4 | Screen shake displacement, victory confetti particles, custom cursor scaling, reduced motion mode |
| 13 | Responsive Viewports | `resp_01` - `resp_08` | 8 | Multi-resolution testing at 900x600, 1280x720, 1600x900, 1920x1080 |
| 14 | Regression Fixes | `regr_01` - `regr_05` | 5 | Slider tracks, unclipped statistics scroll container, achievement grid layout, checkmark glyphs |

---

### 4. Technical Test Methodology
1. **Automated Surface Capture:**
   The application executable supports the `--screenshot <path.bmp>` CLI argument. Upon initialization and layout calculation, the application renders 5 frames to ensure stable layout convergence and particle initialization, triggers `egui::ViewportCommand::Screenshot`, and flushes a 32-bit uncompressed BMP directly from the native graphics surface.
2. **Lossless Conversion & Archival:**
   The audit harness converts BMP captures to optimized PNGs using GDI+ System.Drawing and archives them under category-specific directories in `docs/qa/update-09-9/<category>/`.
3. **Visual Inspection Protocol:**
   Each category's artifacts are inspected using Antigravity image inspection tools. Inspections evaluate:
   - Margin and padding uniformity (minimum 8px touch clearance).
   - Text rendering and baseline alignment.
   - Contrast ratios (WCAG AA compliant wherever possible).
   - Slider rail and thumb visibility.
   - Card container boundary clipping and scrollbar behavior.
4. **Defect Tracking & Triage:**
   Any anomalies are documented in `DEFECTS.md` with severity ratings (Critical, Major, Minor, Cosmetic) and resolved iteratively.

---

### 5. Quality Gate Criteria for Update 10 Readiness
- [ ] 100% of the 98 targeted audit scenarios execute and produce valid screenshots.
- [ ] Zero Critical or Major visual defects remaining across any screen.
- [ ] Zero build warnings or clippy lints in workspace code.
- [ ] 100% unit and integration tests passing (`cargo test --workspace`).
- [ ] Production database integrity preserved and verified.
- [ ] Comprehensive documentation bundle completed in `docs/qa/update-09-9/`.
