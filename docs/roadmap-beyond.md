# MINE//SHIFT Master Roadmap: Updates 07-26

This document outlines the massive 20-update roadmap charting MINE//SHIFT's evolution from a single offline Minesweeper application to a 1,000+ feature puzzle ecosystem.

## 07 — MINECORE 2.0 (CURRENT)
- **Goals:** Extricate the core puzzle rules into a standalone, headless library capable of open-source distribution.
- **Dependencies:** None (Baseline restructuring).
- **Features:** Clean architectural boundaries, engine-specific `Cargo.toml`, No Guess bounding, headless tests.
- **Status:** IMPLEMENTED.

## 08 — Ultimate Customization
- **Goals:** Fully decouple `Sandbox` from presets and give the user absolute control over colors, board geometry, rules, and logic subsets.
- **Dependencies:** MINECORE 2.0
- **Features:** RGB Soft Circuit palettes, Custom WxHxM menus, Rule toggles (No Chording, etc).

## 09 — Feel Everything
- **Goals:** Tactile visual UI polish, including particle effects, fluid animations, and screen-shake on defeat.
- **Features:** Custom egui painting shaders, sweeping reveal animations, polished explosion effects.

## 10 — Gameplay Evolution (CURRENT)
- **Goals:** Advanced Logic Engine, New Puzzle Mechanics, Mastery Challenges, and Evolved Game Modes across Square and Hex boards.
- **Features:**
  - Structured deduction engine (`DeductionType`, `Explanation`, Ring-2 constraints, Subset reductions, non-leaking player hints).
  - Extended Clues (Ring-2 Clue cells with distinctive visuals and dual-ring constraints).
  - Modifier rules (No Flags / Precision Mode, Limited Flags with pool exhaustion).
  - First-Click Opening policies (Standard Safe, Guaranteed 0 Opening, Validated Deduction Opening).
  - Mastery Challenge system (11-challenge tiered catalog across 5 categories, SQLite persistence & XP rewards).
  - 9 Evolved Game Modes (Blitz dynamic combo multipliers, Survival heart resilience, Zen logic hints, Expedition modifiers, Custom Sandbox Lab).
- **Status:** IMPLEMENTED & VERIFIED.

## 11 — Windows 1.0
- **Goals:** Exit beta. Polish SQLite error handling, build installers, add native OS notifications, freeze local save formats.

## 12 — Beyond the Grid
- **Goals:** Introduce Triangular and arbitrary Voronoi topologies to MINECORE.

## 13 — Logic Academy
- **Goals:** Interactive tutorials that use the Rust `Solver` engine to teach the player *how* to play No Guess by visualizing subset constraints on the board.

## 14 — Expedition Universe
- **Goals:** Expand Expedition from 30 linear stages to a branching roguelike overworld map.

## 15 — Creator Studio
- **Goals:** Allow users to paint custom mine patterns or pixel art and save them as seedable JSON strings for Daily Challenges.

## 16 — Infinite Puzzle Lab
- **Goals:** Continuous, endless scrolling boards generating procedural chunks vertically.

## 17 — Connected Profiles
- **Goals:** Cloudflare-backed user accounts seamlessly mapping the local SQLite progress to a remote DB.

## 18 — Ranked Universe
- **Goals:** ELO matchmaking and asynchronous leaderboards for Sprint and Blitz times.

## 19 — Ghost Protocol
- **Goals:** Record, replay, and race against ghost cursors of your past personal bests.

## 20 — Co-op Minefields
- **Goals:** Real-time WebSockets allowing two players to flag and chord a giant shared board concurrently.

## 21 — Minefield Arena
- **Goals:** Competitive Battle Royale Minesweeper. 100 players, clearing grids expands your territory; making a mistake incurs massive time penalties.

## 22 — Modding Revolution
- **Goals:** Lua/WASM scripting integration into MINECORE for custom game rules written by players.

## 23 — Community Workshop
- **Goals:** In-game browser to download custom themes, Lua mods, and user-generated levels.

## 24 — MINE//SHIFT Mobile
- **Goals:** Deploy the exact same Rust engine to Android/iOS with touch-native eframe gestures.

## 25 — Spatial Minefields
- **Goals:** 3D boards (spheres, cubes) rendered via WebGPU using a 3D extension of the topology API.

## 26 — MINECORE 1.0 Ecosystem
- **Goals:** Formal crate publication of the engine, opening the ecosystem up to third-party frontends.

---
*Machine readable feature registry exists in `docs/feature-registry/`.*
