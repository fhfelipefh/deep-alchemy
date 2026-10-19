import json
import sys
from pathlib import Path
from typing import Optional

import click

from cli_anything.deep_alchemy.core.alchemy import (
    brew_ingredients,
    parse_ron_recipes,
)
from cli_anything.deep_alchemy.core.dungeon import DungeonInspector
from cli_anything.deep_alchemy.core.engine import (
    find_game_binary,
    find_workspace_root,
    get_latest_telemetry,
    run_game,
    run_screenshot_test,
    validate_frame,
)
from cli_anything.deep_alchemy.core.player import PlayerState
from cli_anything.deep_alchemy.core.session import GameSession
from cli_anything.deep_alchemy.utils.repl_skin import ReplSkin

VERSION = "0.1.0"

def print_output(data: dict, as_json: bool, skin: Optional[ReplSkin] = None, title: str = ""):

    if as_json:
        click.echo(json.dumps(data, indent=2))
    else:
        if skin:
            if title:
                skin.status("Module", title)
            for k, v in data.items():
                if isinstance(v, (dict, list)):
                    skin.info(f"{k}:")
                    click.echo(json.dumps(v, indent=2))
                else:
                    skin.status(str(k), str(v))
        else:
            click.echo(json.dumps(data, indent=2))

@click.group(invoke_without_command=True)
@click.option("--json", "as_json", is_flag=True, help="Output machine-readable JSON")
@click.pass_context
def cli(ctx, as_json: bool):

    ctx.ensure_object(dict)
    ctx.obj["json"] = as_json
    ctx.obj["session"] = GameSession()

    if ctx.invoked_subcommand is None:
        ctx.invoke(repl)

@cli.command("status")
@click.option("--json", "as_json", is_flag=True, help="Output machine-readable JSON")
@click.pass_context
def status(ctx, as_json: bool):

    as_json = as_json or ctx.obj.get("json", False)
    root = find_workspace_root()
    binary = find_game_binary(root)
    session = ctx.obj["session"]

    data = {
        "workspace_root": str(root),
        "game_binary": str(binary) if binary else None,
        "binary_compiled": binary is not None,
        "assets_models": (root / "assets" / "models" / "cavern_kit.glb").exists(),
        "recipes_ron": (root / "assets" / "data" / "alchemy_recipes.ron").exists(),
        "scene_spec": (root / "assets" / "data" / "scene_spec.json").exists(),
        "active_dungeon_seed": session.dungeon_seed,
        "active_hotbar_slot": session.player.active_slot,
    }
    skin = ReplSkin("deep_alchemy", version=VERSION)
    print_output(data, as_json, skin, title="System Status")

@cli.group("alchemy")
def alchemy_group():

    pass

@alchemy_group.command("recipes")
@click.option("--json", "as_json", is_flag=True, help="Output machine-readable JSON")
@click.pass_context
def alchemy_recipes(ctx, as_json: bool):

    as_json = as_json or ctx.obj.get("json", False)
    db = parse_ron_recipes()
    skin = ReplSkin("deep_alchemy", version=VERSION)
    print_output(db, as_json, skin, title="Alchemy Database")

@alchemy_group.command("brew")
@click.argument("ingredients", nargs=-1, required=True)
@click.option("--json", "as_json", is_flag=True, help="Output machine-readable JSON")
@click.pass_context
def alchemy_brew(ctx, ingredients, as_json: bool):

    as_json = as_json or ctx.obj.get("json", False)
    session: GameSession = ctx.obj["session"]
    result = brew_ingredients(list(ingredients))

    if result.get("success"):
        session.record_action(f"Brew: {', '.join(ingredients)} -> {result.get('result')}")

    skin = ReplSkin("deep_alchemy", version=VERSION)
    print_output(result, as_json, skin, title="Brewing Result")

@cli.group("dungeon")
def dungeon_group():

    pass

@dungeon_group.command("inspect")
@click.option("--seed", type=int, default=42, help="Dungeon generation seed")
@click.option("--json", "as_json", is_flag=True, help="Output machine-readable JSON")
@click.pass_context
def dungeon_inspect(ctx, seed: int, as_json: bool):

    as_json = as_json or ctx.obj.get("json", False)
    inspector = DungeonInspector(seed=seed)
    summary = inspector.get_summary()

    skin = ReplSkin("deep_alchemy", version=VERSION)
    print_output(summary, as_json, skin, title=f"Dungeon Metrics (Seed {seed})")

@dungeon_group.command("map")
@click.option("--seed", type=int, default=42, help="Dungeon generation seed")
@click.pass_context
def dungeon_map(ctx, seed: int):

    inspector = DungeonInspector(seed=seed)
    click.echo(inspector.render_ascii())

@cli.group("player")
def player_group():

    pass

@player_group.command("stats")
@click.option("--json", "as_json", is_flag=True, help="Output machine-readable JSON")
@click.pass_context
def player_stats(ctx, as_json: bool):

    as_json = as_json or ctx.obj.get("json", False)
    session: GameSession = ctx.obj["session"]
    data = session.player.to_dict()
    skin = ReplSkin("deep_alchemy", version=VERSION)
    print_output(data, as_json, skin, title="Player Status")

@player_group.command("select-slot")
@click.argument("slot", type=click.IntRange(1, 5))
@click.option("--json", "as_json", is_flag=True, help="Output machine-readable JSON")
@click.pass_context
def player_select_slot(ctx, slot: int, as_json: bool):

    as_json = as_json or ctx.obj.get("json", False)
    session: GameSession = ctx.obj["session"]
    ok = session.select_hotbar_slot(slot)

    data = {"slot": slot, "success": ok, "player": session.player.to_dict()}
    skin = ReplSkin("deep_alchemy", version=VERSION)
    print_output(data, as_json, skin, title=f"Hotbar Slot {slot} Selected")

@cli.group("engine")
def engine_group():

    pass

@engine_group.command("screenshot")
@click.option("--json", "as_json", is_flag=True, help="Output machine-readable JSON")
@click.pass_context
def engine_screenshot(ctx, as_json: bool):

    as_json = as_json or ctx.obj.get("json", False)
    skin = ReplSkin("deep_alchemy", version=VERSION)
    if not as_json:
        skin.info("Launching Deep Alchemy in --screenshot-test mode...")

    capture_res = run_screenshot_test()
    validation_front = {}
    validation_rear = {}

    root = find_workspace_root()
    front_path = root / "debug_output" / "frame_check.png"
    rear_path = root / "debug_output" / "frame_rear_check.png"

    if front_path.exists():
        validation_front = validate_frame(front_path)
    if rear_path.exists():
        validation_rear = validate_frame(rear_path)

    report = {
        "execution": capture_res,
        "front_frame": validation_front,
        "rear_frame": validation_rear,
        "all_passed": validation_front.get("passed", False) and validation_rear.get("passed", False),
    }

    print_output(report, as_json, skin, title="Screenshot Gate Validation")

@engine_group.command("telemetry")
@click.option("--json", "as_json", is_flag=True, help="Output machine-readable JSON")
@click.pass_context
def engine_telemetry(ctx, as_json: bool):

    as_json = as_json or ctx.obj.get("json", False)
    telemetry = get_latest_telemetry()
    skin = ReplSkin("deep_alchemy", version=VERSION)
    print_output(telemetry or {"error": "No telemetry found. Run game first."}, as_json, skin, title="Frustum Telemetry")

@cli.group("session")
def session_group():

    pass

@session_group.command("undo")
@click.pass_context
def session_undo(ctx):

    session: GameSession = ctx.obj["session"]
    ok = session.undo()
    skin = ReplSkin("deep_alchemy", version=VERSION)
    if ok:
        skin.success("Action undone")
    else:
        skin.warning("Nothing to undo")

@session_group.command("redo")
@click.pass_context
def session_redo(ctx):

    session: GameSession = ctx.obj["session"]
    ok = session.redo()
    skin = ReplSkin("deep_alchemy", version=VERSION)
    if ok:
        skin.success("Action redone")
    else:
        skin.warning("Nothing to redo")

@cli.command("repl")
@click.pass_context
def repl(ctx):

    skin = ReplSkin("deep_alchemy", version=VERSION)
    skin.print_banner()

    session: GameSession = ctx.obj["session"]
    skin.info("Type 'help' for available commands, or 'exit'/'quit' to leave.")

    while True:
        try:
            line = input(f"{skin._CYAN}deep_alchemy{skin._RESET} [slot {session.player.active_slot}] > ").strip()
            if not line:
                continue
            if line.lower() in ("exit", "quit", "q"):
                skin.print_goodbye()
                break

            parts = line.split()
            cmd_name = parts[0]
            cmd_args = parts[1:]

            if cmd_name == "help":
                skin.status("Available commands", "status, alchemy, dungeon, player, engine, session, exit")
                continue

            try:
                cli.main(args=parts, standalone_mode=False)
            except Exception as e:
                skin.error(f"Command error: {e}")

        except (KeyboardInterrupt, EOFError):
            skin.print_goodbye()
            break

def main():
    cli(obj={})

if __name__ == "__main__":
    main()
