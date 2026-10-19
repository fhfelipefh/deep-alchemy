import unittest
from pathlib import Path
from cli_anything.deep_alchemy.core.alchemy import parse_ron_recipes, brew_ingredients

class TestAlchemy(unittest.TestCase):
    def test_parse_ron_recipes(self):
        db = parse_ron_recipes()
        self.assertIn("ingredients", db)
        self.assertIn("potions", db)
        self.assertGreaterEqual(len(db["ingredients"]), 3)
        self.assertGreaterEqual(len(db["potions"]), 2)

        ing_ids = [ing["id"] for ing in db["ingredients"]]
        self.assertIn("ing_cyan_spore", ing_ids)
        self.assertIn("ing_thermal_brimstone", ing_ids)
        self.assertIn("ing_cave_lichen", ing_ids)

    def test_brew_ingredients_invalid(self):
        res = brew_ingredients(["ing_cyan_spore"])
        self.assertFalse(res["success"])
        self.assertIn("requires at least 2", res["error"])

    def test_brew_ingredients_unknown(self):
        res = brew_ingredients(["unknown_herb", "ing_cyan_spore"])
        self.assertFalse(res["success"])
        self.assertIn("Unknown ingredient", res["error"])

    def test_brew_ingredients_slag(self):

        res = brew_ingredients(["ing_cyan_spore", "ing_thermal_brimstone"])
        self.assertTrue(res["success"])
        self.assertEqual(res["result"], "Alchemical Slag")

if __name__ == "__main__":
    unittest.main()
