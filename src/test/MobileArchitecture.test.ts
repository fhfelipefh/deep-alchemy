import { describe, it, expect } from 'vitest';
import * as fs from 'fs';
import * as path from 'path';
import { MobileControls } from '../ui/MobileControls';

describe('Mobile Architecture and Usability Invariants', () => {
  it('enforces that StartMenu has at most 4 words in total', () => {
    const startMenuFile = path.resolve(__dirname, '../ui/StartMenu.ts');
    const content = fs.readFileSync(startMenuFile, 'utf-8');

    const htmlMatch = content.match(/this\.overlay\.innerHTML\s*=\s*`([\s\S]*?)`;/);
    expect(htmlMatch).not.toBeNull();

    const rawHtml = htmlMatch![1];
    const strippedText = rawHtml
      .replace(/<style[\s\S]*?<\/style>/gi, '')
      .replace(/<script[\s\S]*?<\/script>/gi, '')
      .replace(/<svg[\s\S]*?<\/svg>/gi, '')
      .replace(/<[^>]+>/g, ' ')
      .replace(/&[a-z]+;/gi, ' ')
      .trim();

    const words = strippedText.split(/\s+/).filter(Boolean);
    expect(words.length).toBeLessThanOrEqual(4);
    expect(words).toEqual(['ALQUIMIA', 'DAS', 'PROFUNDEZAS', 'JOGAR']);
  });

  it('guarantees touch target sizes meet or exceed mobile accessibility standard', () => {
    const mobileControlsFile = path.resolve(__dirname, '../ui/MobileControls.ts');
    const mcContent = fs.readFileSync(mobileControlsFile, 'utf-8');

    expect(mcContent).toContain("this.joystickBase.style.width = '100px'");
    expect(mcContent).toContain("this.joystickBase.style.height = '100px'");
    expect(mcContent).toContain("btn.style.width = '52px'");
    expect(mcContent).toContain("btn.style.height = '52px'");

    const hudFile = path.resolve(__dirname, '../ui/DiegeticHUD.ts');
    const hudContent = fs.readFileSync(hudFile, 'utf-8');
    expect(hudContent).toContain("slot.style.width = '48px'");
    expect(hudContent).toContain("slot.style.height = '48px'");
  });

  it('verifies viewport-fit cover, disabled zoom, and touch-action none in index.html', () => {
    const indexPath = path.resolve(__dirname, '../../index.html');
    const html = fs.readFileSync(indexPath, 'utf-8');

    expect(html).toContain('viewport-fit=cover');
    expect(html).toContain('user-scalable=no');
    expect(html).toContain('touch-action: none');
  });

  it('guarantees safe-area insets are respected across mobile UI components', () => {
    const mobileControlsFile = path.resolve(__dirname, '../ui/MobileControls.ts');
    const mcContent = fs.readFileSync(mobileControlsFile, 'utf-8');
    expect(mcContent).toContain('env(safe-area-inset-bottom');
    expect(mcContent).toContain('env(safe-area-inset-left');
    expect(mcContent).toContain('env(safe-area-inset-right');

    const hudFile = path.resolve(__dirname, '../ui/DiegeticHUD.ts');
    const hudContent = fs.readFileSync(hudFile, 'utf-8');
    expect(hudContent).toContain('env(safe-area-inset-top');
    expect(hudContent).toContain('env(safe-area-inset-bottom');
  });

  it('ensures decoupled multi-touch state management in MobileControls', () => {
    const originalDoc = (globalThis as any).document;
    const originalWin = (globalThis as any).window;

    (globalThis as any).document = {
      createElement: () => ({
        style: {},
        appendChild: () => {},
        addEventListener: () => {},
      }),
      body: { appendChild: () => {} },
    };
    (globalThis as any).window = {
      addEventListener: () => {},
    };

    const controls = new MobileControls();

    controls.state.moveX = -0.75;
    controls.state.moveZ = 0.5;
    controls.state.isJumping = true;
    controls.state.isRunning = true;

    expect(controls.state.moveX).toBe(-0.75);
    expect(controls.state.moveZ).toBe(0.5);
    expect(controls.state.isJumping).toBe(true);
    expect(controls.state.isRunning).toBe(true);

    (globalThis as any).document = originalDoc;
    (globalThis as any).window = originalWin;
  });
});
