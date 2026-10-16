import sys
import os
import time
import json
from playwright.sync_api import sync_playwright

def run_performance_benchmark():
    target_url = os.environ.get("TEST_URL", "http://localhost:4173/")
    print(f"--- INICIANDO BENCHMARK DE PERFORMANCE E GARGALOS EM: {target_url} ---")

    with sync_playwright() as p:
        browser = p.chromium.launch(
            headless=True,
            args=[
                "--enable-webgl",
                "--use-gl=angle",
                "--use-angle=d3d11",
                "--ignore-gpu-blocklist",
                "--enable-gpu-rasterization",
            ],
        )
        context = browser.new_context(
            viewport={"width": 844, "height": 390},
            has_touch=True,
            is_mobile=True,
        )
        page = context.new_page()

        nav_start = time.perf_counter()
        page.goto(target_url, wait_until="networkidle")
        load_time_ms = (time.perf_counter() - nav_start) * 1000

        start_btn = page.locator("#btn-start-game")
        assert start_btn.is_visible(), "Start button #btn-start-game not visible"

        gen_start = time.perf_counter()
        start_btn.click()
        page.wait_for_timeout(300)

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
        dungeon_init_ms = (time.perf_counter() - gen_start) * 1000

        perf_data = page.evaluate("""() => {
            return new Promise((resolve) => {
                const app = window.__APP__;
                const renderer = app.renderer;
                const glInfo = renderer.info;

                const frameTimes = [];
                let lastTimestamp = performance.now();
                let frameCount = 0;
                const maxFrames = 180;
                let longFrames = 0;

                function recordFrame(now) {
                    const delta = now - lastTimestamp;
                    lastTimestamp = now;
                    frameCount++;

                    if (frameCount > 5) {
                        frameTimes.push(delta);
                        if (delta > 50) {
                            longFrames++;
                        }
                    }

                    if (app.player) {
                        app.player.position.x += Math.sin(frameCount * 0.1) * 0.1;
                        app.player.position.z += Math.cos(frameCount * 0.1) * 0.1;
                    }
                    if (app.playerRenderer) {
                        app.playerRenderer.rotateView(0.015, Math.sin(frameCount * 0.05) * 0.005);
                    }

                    if (frameCount < maxFrames) {
                        requestAnimationFrame(recordFrame);
                    } else {
                        frameTimes.sort((a, b) => a - b);
                        const sum = frameTimes.reduce((a, b) => a + b, 0);
                        const avgDelta = sum / frameTimes.length;
                        const fps = 1000 / avgDelta;
                        const p50 = frameTimes[Math.floor(frameTimes.length * 0.50)];
                        const p95 = frameTimes[Math.floor(frameTimes.length * 0.95)];
                        const p99 = frameTimes[Math.floor(frameTimes.length * 0.99)];
                        const maxDelta = frameTimes[frameTimes.length - 1];

                        const memory = performance.memory ? {
                            usedJSHeapSizeMB: +(performance.memory.usedJSHeapSize / (1024 * 1024)).toFixed(2),
                            totalJSHeapSizeMB: +(performance.memory.totalJSHeapSize / (1024 * 1024)).toFixed(2)
                        } : null;

                        resolve({
                            fps: +fps.toFixed(1),
                            avgDeltaMs: +avgDelta.toFixed(2),
                            p50Ms: +p50.toFixed(2),
                            p95Ms: +p95.toFixed(2),
                            p99Ms: +p99.toFixed(2),
                            maxDeltaMs: +maxDelta.toFixed(2),
                            longFrames: longFrames,
                            drawCalls: glInfo.render.calls,
                            triangles: glInfo.render.triangles,
                            geometriesCount: glInfo.memory.geometries,
                            texturesCount: glInfo.memory.textures,
                            memory: memory
                        });
                    }
                }

                requestAnimationFrame(recordFrame);
            });
        }""")

        browser.close()

        print("\n" + "="*65)
        print("          RELATÓRIO DE PERFORMANCE E GARGALOS (WEBGL)")
        print("="*65)
        print(f"Tempo de Carregamento Inicial:  {load_time_ms:.1f} ms")
        print(f"Inicialização do Labirinto:     {dungeon_init_ms:.1f} ms")
        print(f"Taxa Média de Quadros (FPS):    {perf_data['fps']} FPS")
        print(f"Tempo Médio de Quadro:          {perf_data['avgDeltaMs']} ms (Budget: 16.6 ms)")
        print(f"Latência P50 (Mediana):         {perf_data['p50Ms']} ms")
        print(f"Latência P95 (95% dos frames):  {perf_data['p95Ms']} ms")
        print(f"Latência P99 (Piores 1%):       {perf_data['p99Ms']} ms")
        print(f"Pior Quadro (Spike Máximo):     {perf_data['maxDeltaMs']} ms")
        print(f"Quadros com Engasgo (>50ms):    {perf_data['longFrames']}")
        print(f"Chamadas de Desenho (Draws):    {perf_data['drawCalls']} (Limite Otimizado: < 35)")
        print(f"Total de Triângulos na GPU:     {perf_data['triangles']}")
        print(f"Geometrias em Memória:          {perf_data['geometriesCount']}")
        if perf_data.get("memory"):
            print(f"Uso de Heap JS:                 {perf_data['memory']['usedJSHeapSizeMB']} MB")
        print("="*65)

        assert perf_data["fps"] >= 45.0, f"FPS abaixo do aceitável: {perf_data['fps']} FPS"
        assert perf_data["drawCalls"] <= 35, f"Excesso de draw calls (gargalo): {perf_data['drawCalls']}"
        assert perf_data["longFrames"] <= 2, f"Muitos frames de jank detectados: {perf_data['longFrames']}"

        with open("perf_benchmark_report.json", "w", encoding="utf-8") as f:
            json.dump({
                "load_time_ms": load_time_ms,
                "dungeon_init_ms": dungeon_init_ms,
                "metrics": perf_data
            }, f, indent=2)

        print("\nBENCHMARK CONCLUÍDO COM SUCESSO! ZERO GARGALOS CRÍTICOS DETECTADOS.")
        return 0

if __name__ == "__main__":
    sys.exit(run_performance_benchmark())
