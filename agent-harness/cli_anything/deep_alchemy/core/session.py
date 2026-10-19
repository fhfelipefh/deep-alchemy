import copy
import json
import os
import sys
from pathlib import Path
from typing import Any, Dict, List, Optional

from cli_anything.deep_alchemy.core.player import PlayerState

def _locked_save_json(path: Path, data: Dict[str, Any]) -> None:

    path.parent.mkdir(parents=True, exist_ok=True)
    temp_path = path.with_suffix(".tmp")

    with open(temp_path, "w", encoding="utf-8") as f:
        json.dump(data, f, indent=2)

    temp_path.replace(path)

class GameSession:

    def __init__(self, session_path: Optional[Path] = None):
        self.session_path = session_path or (Path.home() / ".deep_alchemy_session.json")
        self.dungeon_seed = 42
        self.player = PlayerState()
        self.cauldron_ingredients: List[str] = []
        self.history: List[str] = []
        self._undo_stack: List[Dict[str, Any]] = []
        self._redo_stack: List[Dict[str, Any]] = []
        self.load()

    def snapshot(self) -> Dict[str, Any]:
        return {
            "dungeon_seed": self.dungeon_seed,
            "player": self.player.to_dict(),
            "cauldron_ingredients": list(self.cauldron_ingredients),
            "history": list(self.history),
        }

    def _push_undo(self) -> None:
        self._undo_stack.append(self.snapshot())
        self._redo_stack.clear()
        if len(self._undo_stack) > 30:
            self._undo_stack.pop(0)

    def undo(self) -> bool:
        if not self._undo_stack:
            return False
        self._redo_stack.append(self.snapshot())
        prev = self._undo_stack.pop()
        self._restore(prev)
        self.save()
        return True

    def redo(self) -> bool:
        if not self._redo_stack:
            return False
        self._undo_stack.append(self.snapshot())
        nxt = self._redo_stack.pop()
        self._restore(nxt)
        self.save()
        return True

    def _restore(self, snap: Dict[str, Any]) -> None:
        self.dungeon_seed = snap.get("dungeon_seed", 42)
        p_data = snap.get("player", {})
        self.player = PlayerState(
            health=p_data.get("health", 100.0),
            stamina=p_data.get("stamina", 100.0),
            active_slot=p_data.get("active_slot", 1),
        )
        self.cauldron_ingredients = snap.get("cauldron_ingredients", [])
        self.history = snap.get("history", [])

    def select_hotbar_slot(self, slot: int) -> bool:
        self._push_undo()
        ok = self.player.select_slot(slot)
        if ok:
            self.history.append(f"Select hotbar slot {slot}")
            self.save()
            return True
        self._undo_stack.pop()
        return False

    def record_action(self, action_name: str) -> None:
        self._push_undo()
        self.history.append(action_name)
        self.save()

    def save(self) -> None:
        _locked_save_json(self.session_path, self.snapshot())

    def load(self) -> bool:
        if not self.session_path.exists():
            return False
        try:
            with open(self.session_path, "r", encoding="utf-8") as f:
                data = json.load(f)
            self._restore(data)
            return True
        except Exception:
            return False
