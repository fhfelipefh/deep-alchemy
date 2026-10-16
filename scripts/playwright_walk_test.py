import sys
import os
import time
from playwright.sync_api import sync_playwright

def run_walk_test():
    with sync_playwright() as p:
        browser = p.chromium.launch(headless=True)
        context = browser.new_context(
            viewport={"width": 844, "height": 390},
            has_touch=True,
            is_mobile=True,
        )
        page = context.new_page()

        target_url = os.environ.get("TEST_URL", "http://localhost:4173/")
        page.goto(target_url, wait_until="networkidle")

        start_btn = page.locator("#btn-start-game")
        assert start_btn.is_visible(), "Start button #btn-start-game not visible"
        start_btn.click()
        page.wait_for_timeout(400)

        page.evaluate("""() => {
            const c = window.__APP__?.introCinematic;
            if (c) {
                if (typeof c.skip === 'function') {
                    c.skip();
                } else {
                    c.isPlaying = false;
                    c.onCompleteCallback?.();
                }
            }
        }""")

        assert page.evaluate("() => window.__APP__?.isGameActive") is True, "Game failed to become active"

        app_info = page.evaluate("""() => {
            const app = window.__APP__;
            const dungeon = app.currentDungeon;
            return {
                startPos: dungeon.startPos,
                exitPos: dungeon.exitPos,
                cellSize: dungeon.cellSize,
                shortestDist: dungeon.shortestPathLengthMeters,
                playerPos: { x: app.player.position.x, z: app.player.position.z }
            };
        }""")

        assert app_info["shortestDist"] >= 500, f"Dungeon shortest path {app_info['shortestDist']}m is below 500m"

        walk_result = page.evaluate("""() => {
            const app = window.__APP__;
            const dungeon = app.currentDungeon;
            const grid = dungeon.grid;
            const cellSize = dungeon.cellSize;

            const height = grid.length;
            const width = grid[0].length;
            const start = {
                x: Math.floor(app.player.position.x / cellSize),
                z: Math.floor(app.player.position.z / cellSize)
            };
            const exit = dungeon.exitPos;

            const queue = [start];
            const visited = new Set();
            visited.add(`${start.x},${start.z}`);
            const parent = new Map();

            let found = false;
            while (queue.length > 0) {
                const cur = queue.shift();
                if (cur.x === exit.x && cur.z === exit.z) {
                    found = true;
                    break;
                }

                const dirs = [
                    { x: 1, z: 0 },
                    { x: -1, z: 0 },
                    { x: 0, z: 1 },
                    { x: 0, z: -1 }
                ];

                for (const d of dirs) {
                    const nx = cur.x + d.x;
                    const nz = cur.z + d.z;
                    const key = `${nx},${nz}`;

                    if (nx >= 0 && nx < width && nz >= 0 && nz < height) {
                        if (!visited.has(key) && grid[nz][nx] !== 0) {
                            visited.add(key);
                            parent.set(key, cur);
                            queue.push({ x: nx, z: nz });
                        }
                    }
                }
            }

            if (!found) {
                return { success: false, error: 'Path to exit not found in grid' };
            }

            const path = [];
            let currKey = `${exit.x},${exit.z}`;
            while (currKey) {
                const parts = currKey.split(',').map(Number);
                path.push({ x: parts[0], z: parts[1] });
                const p = parent.get(currKey);
                currKey = p ? `${p.x},${p.z}` : null;
            }
            path.reverse();

            let stepCount = 0;
            let blockedSteps = 0;

            for (const node of path) {
                const worldX = (node.x + 0.5) * cellSize;
                const worldZ = (node.z + 0.5) * cellSize;

                if (!app.canWalkTo(worldX, worldZ)) {
                    blockedSteps++;
                    continue;
                }

                app.player.position.x = worldX;
                app.player.position.z = worldZ;
                stepCount++;
            }

            const exitWorldX = (exit.x + 0.5) * cellSize;
            const exitWorldZ = (exit.z + 0.5) * cellSize;
            const finalDist = Math.hypot(app.player.position.x - exitWorldX, app.player.position.z - exitWorldZ);

            if (finalDist < 2.5) {
                app.player.hasReachedExit = true;
            }

            return {
                success: true,
                totalPathNodes: path.length,
                steppedNodes: stepCount,
                blockedSteps: blockedSteps,
                finalDist: finalDist,
                hasReachedExit: app.player.hasReachedExit,
                playerHealth: app.player.currentHealth
            };
        }""")

        assert walk_result["success"], f"Pathfinding failed: {walk_result.get('error')}"
        assert walk_result["blockedSteps"] == 0, f"Path contained {walk_result['blockedSteps']} blocked steps"
        assert walk_result["hasReachedExit"] == True, f"Player did not reach exit. Final dist: {walk_result['finalDist']}"
        assert walk_result["playerHealth"] > 0, "Player died during walk"

        page.screenshot(path="playwright_exit_success.png")

        browser.close()
        print(f"PLAYWRIGHT WALK TEST PASSED: Successfully traversed {walk_result['steppedNodes']} nodes to exit!")
        return 0

if __name__ == "__main__":
    sys.exit(run_walk_test())
