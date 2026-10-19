---
name: cli-anything-deep-alchemy
description: Control, inspect, and test Deep Alchemy (Bevy 0.15 Rust Game Engine) via CLI. Simulate systemic cauldron brewing, inspect procedural WFC dungeon layouts, query player stats, and execute automated visual regression screenshot gates.
---

# Deep Alchemy CLI Skill

Use this skill when interacting with the **Deep Alchemy** game engine, testing alchemy combinations, inspecting procedural dungeons, or running visual regression tests.

## Available Commands

| Command | Description | Example |
|---------|-------------|---------|
| `cli-anything-deep-alchemy status` | Engine status, binary paths, and asset integrity | `cli-anything-deep-alchemy status --json` |
| `cli-anything-deep-alchemy alchemy recipes` | List ingredients and known potion recipes | `cli-anything-deep-alchemy alchemy recipes --json` |
| `cli-anything-deep-alchemy alchemy brew <ings...>` | Simulate cauldron brewing | `cli-anything-deep-alchemy alchemy brew ing_cyan_spore ing_cave_lichen --json` |
| `cli-anything-deep-alchemy dungeon inspect` | Inspect procedural dungeon layout and dual cells | `cli-anything-deep-alchemy dungeon inspect --seed 42 --json` |
| `cli-anything-deep-alchemy dungeon map` | Render ASCII map of dungeon | `cli-anything-deep-alchemy dungeon map --seed 42` |
| `cli-anything-deep-alchemy player stats` | Query player HP (5 hearts), stamina, and hotbar | `cli-anything-deep-alchemy player stats --json` |
| `cli-anything-deep-alchemy player select-slot <1-5>` | Change active hotbar slot | `cli-anything-deep-alchemy player select-slot 3 --json` |
| `cli-anything-deep-alchemy engine screenshot` | Run `--screenshot-test` and validate Sobel edge gate | `cli-anything-deep-alchemy engine screenshot --json` |
| `cli-anything-deep-alchemy engine telemetry` | Dump camera frustum JSON telemetry | `cli-anything-deep-alchemy engine telemetry --json` |
| `cli-anything-deep-alchemy repl` | Interactive REPL mode | `cli-anything-deep-alchemy repl` |

All commands support `--json` for machine-readable agent consumption.
