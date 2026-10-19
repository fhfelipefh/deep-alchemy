import re
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple

def get_default_recipe_file() -> Path:
    from cli_anything.deep_alchemy.core.engine import find_workspace_root

    root = find_workspace_root()
    return root / "assets" / "data" / "alchemy_recipes.ron"

def parse_ron_recipes(ron_path: Optional[Path] = None) -> Dict[str, Any]:

    path = ron_path or get_default_recipe_file()
    if not path.exists():
        return {"ingredients": [], "potions": [], "error": f"File not found: {path}"}

    content = path.read_text(encoding="utf-8")

    ingredients: List[Dict[str, Any]] = []
    potions: List[Dict[str, Any]] = []

    ing_matches = re.findall(
        r'id:\s*"([^"]+)",\s*name:\s*"([^"]+)",\s*description:\s*"([^"]+)",\s*weight:\s*([0-9.]+),\s*value:\s*([0-9]+),\s*primary_aspect:\s*([A-Za-z0-9_]+),\s*secondary_aspect:\s*([A-Za-z0-9_]+)',
        content,
    )
    for m in ing_matches:
        ingredients.append(
            {
                "id": m[0],
                "name": m[1],
                "description": m[2],
                "weight": float(m[3]),
                "value": int(m[4]),
                "primary_aspect": m[5],
                "secondary_aspect": m[6],
            }
        )

    pot_matches = re.findall(
        r'id:\s*"([^"]+)",\s*name:\s*"([^"]+)",\s*liquid_color:\s*\(([0-9.,\s]+)\)',
        content,
    )
    for p_id, p_name, color_str in pot_matches:
        colors = [float(c.strip()) for c in color_str.split(",") if c.strip()]

        pot_block_regex = rf'id:\s*"{re.escape(p_id)}".*?effects:\s*\[(.*?)\]'
        pot_block = re.search(pot_block_regex, content, re.DOTALL)
        effects: List[Dict[str, Any]] = []
        if pot_block:
            effects_str = pot_block.group(1)
            eff_matches = re.findall(
                r'kind:\s*([A-Za-z0-9_]+),\s*duration:\s*([0-9.]+),\s*magnitude:\s*([0-9.]+)',
                effects_str,
            )
            for e_kind, e_dur, e_mag in eff_matches:
                effects.append(
                    {
                        "kind": e_kind,
                        "duration": float(e_dur),
                        "magnitude": float(e_mag),
                    }
                )

        potions.append(
            {
                "id": p_id,
                "name": p_name,
                "liquid_color": colors,
                "effects": effects,
            }
        )

    return {
        "ingredients": ingredients,
        "potions": potions,
        "source": str(path),
    }

def brew_ingredients(
    ingredient_ids: List[str],
    ron_path: Optional[Path] = None,
) -> Dict[str, Any]:

    db = parse_ron_recipes(ron_path)
    ing_map = {ing["id"]: ing for ing in db["ingredients"]}

    selected_ings = []
    unknown = []
    for i_id in ingredient_ids:
        if i_id in ing_map:
            selected_ings.append(ing_map[i_id])
        else:
            unknown.append(i_id)

    if unknown:
        return {
            "success": False,
            "error": f"Unknown ingredient IDs: {', '.join(unknown)}",
            "available_ids": list(ing_map.keys()),
        }

    if len(selected_ings) < 2:
        return {
            "success": False,
            "error": "Brewing requires at least 2 ingredients",
        }

    aspect_counts: Dict[str, int] = {}
    for ing in selected_ings:
        p = ing["primary_aspect"]
        s = ing["secondary_aspect"]
        aspect_counts[p] = aspect_counts.get(p, 0) + 1
        aspect_counts[s] = aspect_counts.get(s, 0) + 1

    shared_aspects = [asp for asp, count in aspect_counts.items() if count >= 2]

    if not shared_aspects:
        return {
            "success": True,
            "result": "Alchemical Slag",
            "potion_id": "slag",
            "description": "Incompatible reagents produced toxic inert residue",
            "effects": [],
            "shared_aspects": [],
            "used_ingredients": [ing["name"] for ing in selected_ings],
        }

    matched_potion = None
    for pot in db["potions"]:
        pot_kinds = {eff["kind"] for eff in pot["effects"]}
        if pot_kinds.issubset(set(shared_aspects)):
            matched_potion = pot
            break

    if matched_potion:
        return {
            "success": True,
            "result": matched_potion["name"],
            "potion_id": matched_potion["id"],
            "effects": matched_potion["effects"],
            "shared_aspects": shared_aspects,
            "liquid_color": matched_potion.get("liquid_color", [0.5, 0.5, 0.5, 1.0]),
            "used_ingredients": [ing["name"] for ing in selected_ings],
        }

    return {
        "success": True,
        "result": "Unstable Concoction",
        "potion_id": "pot_custom",
        "effects": [{"kind": asp, "duration": 10.0, "magnitude": 1.0} for asp in shared_aspects],
        "shared_aspects": shared_aspects,
        "liquid_color": [0.2, 0.8, 0.6, 0.8],
        "used_ingredients": [ing["name"] for ing in selected_ings],
    }
