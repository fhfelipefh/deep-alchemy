# Deep Alchemy SOP & Architecture Specification

## Overview

Deep Alchemy is a 100% code-first, cel-shaded, first-person dungeon exploration and systemic alchemy game built in Rust with Bevy 0.15, Avian3D, and custom WGSL shaders.

## Command Architecture

| Domain | Subcommand | Purpose |
|--------|------------|---------|
| Status | `status` | Checks binary, asset integrity, and active session |
| Alchemy | `alchemy recipes` | Lists ingredients, aspects, and potions from RON |
| Alchemy | `alchemy brew` | Simulates cauldron reaction with aspect matching |
| Dungeon | `dungeon inspect` | Inspects 24x24 dual-cell WFC dungeon layout |
| Dungeon | `dungeon map` | Renders ASCII visualization of dungeon |
| Player | `player stats` | Inspects 5 hearts (100 HP), stamina, and hotbar |
| Player | `player select-slot` | Changes active HUD hotbar slot (1-5) |
| Engine | `engine screenshot` | Runs `--screenshot-test` and validates frames |
| Engine | `engine telemetry` | Dumps zero-vision camera frustum JSON data |
| Session | `session undo/redo` | Reversible command history with file locking |
