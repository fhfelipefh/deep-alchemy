from typing import Any, Dict, List, Optional

class PlayerState:

    DEFAULT_HOTBAR = [
        {"slot": 1, "item": "potion_green", "name": "Antidote Flask", "active": True},
        {"slot": 2, "item": "potion_red", "name": "Health Draught", "active": False},
        {"slot": 3, "item": "potion_yellow", "name": "Luminescent Elixir", "active": False},
        {"slot": 4, "item": "potion_blue", "name": "Stamina Philter", "active": False},
        {"slot": 5, "item": "potion_white", "name": "Pure Solvent", "active": False},
    ]

    def __init__(
        self,
        health: float = 100.0,
        max_health: float = 100.0,
        stamina: float = 100.0,
        max_stamina: float = 100.0,
        active_slot: int = 1,
    ):
        self.health = health
        self.max_health = max_health
        self.stamina = stamina
        self.max_stamina = max_stamina
        self.active_slot = active_slot
        self.hotbar = [dict(s) for s in self.DEFAULT_HOTBAR]
        self._sync_active_slot()

    def _sync_active_slot(self) -> None:
        for item in self.hotbar:
            item["active"] = item["slot"] == self.active_slot

    def select_slot(self, slot: int) -> bool:

        if 1 <= slot <= 5:
            self.active_slot = slot
            self._sync_active_slot()
            return True
        return False

    def take_damage(self, amount: float) -> float:
        self.health = max(0.0, self.health - amount)
        return self.health

    def heal(self, amount: float) -> float:
        self.health = min(self.max_health, self.health + amount)
        return self.health

    def get_heart_count(self) -> float:

        return round(self.health / 20.0, 1)

    def to_dict(self) -> Dict[str, Any]:
        return {
            "health": self.health,
            "max_health": self.max_health,
            "hearts_display": f"{self.get_heart_count()}/5.0",
            "stamina": self.stamina,
            "max_stamina": self.max_stamina,
            "active_slot": self.active_slot,
            "active_item": next(
                (s["name"] for s in self.hotbar if s["slot"] == self.active_slot),
                None,
            ),
            "hotbar": self.hotbar,
            "locomotion_constants": {
                "walk_speed_mps": 4.0,
                "sprint_speed_mps": 7.5,
                "crouch_speed_mps": 2.0,
                "stamina_drain_per_sec": 24.0,
                "stamina_regen_delay_sec": 1.25,
                "stamina_regen_rate_per_sec": 28.0,
            },
        }
