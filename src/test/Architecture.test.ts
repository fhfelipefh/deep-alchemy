import { describe, it, expect } from 'vitest';
import * as fs from 'fs';
import * as path from 'path';

function getFilesRecursively(dir: string, extension: string): string[] {
  let results: string[] = [];
  const list = fs.readdirSync(dir);
  for (const file of list) {
    const fullPath = path.join(dir, file);
    const stat = fs.statSync(fullPath);
    if (stat && stat.isDirectory()) {
      results = results.concat(getFilesRecursively(fullPath, extension));
    } else if (file.endsWith(extension)) {
      results.push(fullPath);
    }
  }
  return results;
}

describe('Testes de Arquitetura e Higiene de Código', () => {
  it('garante que não existam comentários em nenhum arquivo TypeScript de src', () => {
    const srcDir = path.resolve(__dirname, '..');
    const tsFiles = getFilesRecursively(srcDir, '.ts');

    const filesWithComments: { file: string; line: number; text: string }[] = [];

    for (const file of tsFiles) {
      const content = fs.readFileSync(file, 'utf-8');
      const lines = content.split('\n');

      let inBlockComment = false;
      lines.forEach((lineText, idx) => {
        const trimmed = lineText.trim();

        if (inBlockComment) {
          filesWithComments.push({ file: path.basename(file), line: idx + 1, text: trimmed });
          if (trimmed.includes('*/')) {
            inBlockComment = false;
          }
          return;
        }

        if (trimmed.startsWith('/*')) {
          inBlockComment = true;
          filesWithComments.push({ file: path.basename(file), line: idx + 1, text: trimmed });
          if (trimmed.includes('*/')) {
            inBlockComment = false;
          }
          return;
        }

        if (trimmed.startsWith('//')) {
          filesWithComments.push({ file: path.basename(file), line: idx + 1, text: trimmed });
        }
      });
    }

    expect(filesWithComments).toEqual([]);
  });

  it('garante que todos os modelos 3D em public/models tenham utilização ativa no código', () => {
    const modelsDir = path.resolve(__dirname, '../../public/models');
    const modelFiles = fs.readdirSync(modelsDir).filter(f => f.endsWith('.glb'));

    const srcDir = path.resolve(__dirname, '..');
    const tsFiles = getFilesRecursively(srcDir, '.ts');
    const allCode = tsFiles.map(f => fs.readFileSync(f, 'utf-8')).join('\n');

    expect(modelFiles.length).toBeGreaterThan(0);

    for (const model of modelFiles) {
      const isReferenced = allCode.includes(model);
      expect(isReferenced, `Modelo ${model} deve estar sendo utilizado no código`).toBe(true);
    }
  });

  it('garante que os componentes essenciais de arquitetura estejam integrados', () => {
    const mainPath = path.resolve(__dirname, '../main.ts');
    const mainContent = fs.readFileSync(mainPath, 'utf-8');

    expect(mainContent).toContain('PlayerRenderer');
    expect(mainContent).toContain('DungeonRenderer');
    expect(mainContent).toContain('CritterRenderer');
    expect(mainContent).toContain('PotionRenderer');
    expect(mainContent).toContain('IntroCinematic');
    expect(mainContent).toContain('DiegeticHUD');
    expect(mainContent).toContain('MobileControls');
    expect(mainContent).toContain('StartMenu');
    expect(mainContent).toContain('WFCGenerator');
  });
});
