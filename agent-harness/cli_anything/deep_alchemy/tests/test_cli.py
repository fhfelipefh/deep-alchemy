import json
import unittest
from click.testing import CliRunner
from cli_anything.deep_alchemy.deep_alchemy_cli import cli

class TestCLI(unittest.TestCase):
    def setUp(self):
        self.runner = CliRunner()

    def test_status_json(self):
        result = self.runner.invoke(cli, ["status", "--json"])
        self.assertEqual(result.exit_code, 0)
        data = json.loads(result.output)
        self.assertIn("workspace_root", data)
        self.assertIn("binary_compiled", data)

    def test_alchemy_recipes_json(self):
        result = self.runner.invoke(cli, ["alchemy", "recipes", "--json"])
        self.assertEqual(result.exit_code, 0)
        data = json.loads(result.output)
        self.assertIn("ingredients", data)
        self.assertIn("potions", data)

    def test_alchemy_brew_json(self):
        result = self.runner.invoke(cli, ["alchemy", "brew", "ing_cyan_spore", "ing_thermal_brimstone", "--json"])
        self.assertEqual(result.exit_code, 0)
        data = json.loads(result.output)
        self.assertTrue(data.get("success"))

    def test_dungeon_inspect_json(self):
        result = self.runner.invoke(cli, ["dungeon", "inspect", "--seed", "100", "--json"])
        self.assertEqual(result.exit_code, 0)
        data = json.loads(result.output)
        self.assertEqual(data["seed"], 100)
        self.assertEqual(data["total_dual_cells"], 576)

    def test_dungeon_map(self):
        result = self.runner.invoke(cli, ["dungeon", "map"])
        self.assertEqual(result.exit_code, 0)
        self.assertIn("Legend:", result.output)

    def test_player_stats_json(self):
        result = self.runner.invoke(cli, ["player", "stats", "--json"])
        self.assertEqual(result.exit_code, 0)
        data = json.loads(result.output)
        self.assertIn("health", data)
        self.assertIn("hotbar", data)

    def test_player_select_slot_json(self):
        result = self.runner.invoke(cli, ["player", "select-slot", "4", "--json"])
        self.assertEqual(result.exit_code, 0)
        data = json.loads(result.output)
        self.assertEqual(data["slot"], 4)
        self.assertTrue(data["success"])

if __name__ == "__main__":
    unittest.main()
