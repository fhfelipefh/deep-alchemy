import tempfile
import unittest
from pathlib import Path

from cli_anything.deep_alchemy.core.engine import find_game_binary, find_workspace_root
from cli_anything.deep_alchemy.core.player import PlayerState
from cli_anything.deep_alchemy.core.session import GameSession

class TestCore(unittest.TestCase):
    def test_player_invariants(self):
        player = PlayerState()
        self.assertEqual(player.health, 100.0)
        self.assertEqual(player.stamina, 100.0)
        self.assertEqual(player.active_slot, 1)
        self.assertEqual(player.get_heart_count(), 5.0)

        player.take_damage(30.0)
        self.assertEqual(player.health, 70.0)
        self.assertEqual(player.get_heart_count(), 3.5)

        player.heal(15.0)
        self.assertEqual(player.health, 85.0)

    def test_player_hotbar_slots(self):
        player = PlayerState()
        self.assertTrue(player.select_slot(3))
        self.assertEqual(player.active_slot, 3)
        self.assertFalse(player.select_slot(9))
        self.assertEqual(player.active_slot, 3)

    def test_session_undo_redo(self):
        with tempfile.NamedTemporaryFile(suffix=".json", delete=False) as tf:
            temp_path = Path(tf.name)

        try:
            session = GameSession(session_path=temp_path)
            session.select_hotbar_slot(2)
            session.select_hotbar_slot(4)

            self.assertEqual(session.player.active_slot, 4)

            self.assertTrue(session.undo())
            self.assertEqual(session.player.active_slot, 2)

            self.assertTrue(session.redo())
            self.assertEqual(session.player.active_slot, 4)
        finally:
            if temp_path.exists():
                temp_path.unlink()

    def test_workspace_and_binary(self):
        root = find_workspace_root()
        self.assertTrue((root / "Cargo.toml").exists())
        bin_path = find_game_binary(root)
        self.assertIsNotNone(bin_path)
        self.assertTrue(bin_path.exists())

if __name__ == "__main__":
    unittest.main()
