import unittest
from cli_anything.deep_alchemy.core.dungeon import DungeonInspector

class TestDungeon(unittest.TestCase):
    def test_dungeon_dimensions_and_cells(self):
        dungeon = DungeonInspector(width=24, depth=24, seed=42)
        summary = dungeon.get_summary()

        self.assertEqual(summary["dimensions"], [24, 24])
        self.assertEqual(summary["total_dual_cells"], 576)
        self.assertGreater(summary["walkable_cells"], 50)
        self.assertEqual(summary["starting_grotto_cells"], 16)
        self.assertTrue(summary["grotto_anchored_at_center"])
        self.assertEqual(summary["cardinal_exits"], 4)

    def test_dungeon_seed_determinism(self):
        d1 = DungeonInspector(seed=123)
        d2 = DungeonInspector(seed=123)
        self.assertEqual(d1.grid, d2.grid)

        d3 = DungeonInspector(seed=999)
        self.assertNotEqual(d1.grid, d3.grid)

    def test_dungeon_ascii_render(self):
        dungeon = DungeonInspector(width=10, depth=10, seed=42)
        ascii_out = dungeon.render_ascii()
        self.assertIn("Legend:", ascii_out)
        self.assertIn("G", ascii_out)

if __name__ == "__main__":
    unittest.main()
