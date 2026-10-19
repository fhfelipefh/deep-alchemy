# Deep Alchemy CLI Harness

Agent-Native command-line interface for the **Deep Alchemy** desktop game engine (Bevy 0.15, Rust).

## Features

- **Engine Automation**: Run `--screenshot-test` headless tests and validate Sobel edge density and color vibrancy.
- **Systemic Alchemy**: Query ingredients, known potion formulas, and simulate cauldron brewing.
- **Procedural Dungeon**: Inspect WFC and dual-grid layout, seed metrics, and render ASCII mini-maps.
- **Player & HUD**: Query player stats (hearts, stamina) and select active hotbar inventory slots.
- **Agent First**: Full `--json` support across all subcommands.
- **Interactive REPL**: Rich terminal interface with `ReplSkin`, prompt completion, and history.

## Installation

```bash
cd agent-harness
pip install -e .
```

## Quick Start

```bash
# Check status
cli-anything-deep-alchemy status --json

# Inspect dungeon
cli-anything-deep-alchemy dungeon inspect --seed 42 --json

# Simulate alchemy brewing
cli-anything-deep-alchemy alchemy brew ing_cyan_spore ing_cave_lichen --json

# Run visual validation
cli-anything-deep-alchemy engine screenshot --json

# Launch interactive REPL
cli-anything-deep-alchemy
```
