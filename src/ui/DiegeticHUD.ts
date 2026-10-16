import { PlayerState } from '../entities/PlayerState';
import { PotionInventory, PotionType, POTION_CONFIGS } from '../alchemy/PotionSystem';

export class DiegeticHUD {
  private container: HTMLDivElement;
  private heartsContainer: HTMLDivElement;
  private hotbarContainer: HTMLDivElement;
  private slotElements: HTMLDivElement[] = [];
  private onSlotSelectedCallback?: (index: number) => void;

  constructor() {
    this.container = document.createElement('div');
    this.container.id = 'diegetic-hud';
    this.container.style.position = 'fixed';
    this.container.style.top = '0';
    this.container.style.left = '0';
    this.container.style.width = '100%';
    this.container.style.height = '100%';
    this.container.style.pointerEvents = 'none';
    this.container.style.userSelect = 'none';
    this.container.style.zIndex = '50';

    this.heartsContainer = document.createElement('div');
    this.heartsContainer.style.position = 'absolute';
    this.heartsContainer.style.top = '12px';
    this.heartsContainer.style.left = '12px';
    this.heartsContainer.style.display = 'flex';
    this.heartsContainer.style.gap = '3px';
    this.container.appendChild(this.heartsContainer);

    this.hotbarContainer = document.createElement('div');
    this.hotbarContainer.style.position = 'absolute';
    this.hotbarContainer.style.bottom = '12px';
    this.hotbarContainer.style.left = '50%';
    this.hotbarContainer.style.transform = 'translateX(-50%)';
    this.hotbarContainer.style.display = 'flex';
    this.hotbarContainer.style.gap = '6px';
    this.hotbarContainer.style.pointerEvents = 'auto';
    this.container.appendChild(this.hotbarContainer);

    this.createHotbarSlots();
    document.body.appendChild(this.container);
  }

  public setOnSlotSelected(callback: (index: number) => void): void {
    this.onSlotSelectedCallback = callback;
  }

  private createHotbarSlots(): void {
    const potions = [
      PotionType.GLUE,
      PotionType.FIRE,
      PotionType.SHOCK,
      PotionType.ICE,
      PotionType.LIGHT,
    ];

    potions.forEach((potion, index) => {
      const slot = document.createElement('div');
      slot.className = 'hotbar-slot';
      slot.style.width = '48px';
      slot.style.height = '48px';
      slot.style.background = 'rgba(20, 20, 26, 0.85)';
      slot.style.border = '2px solid rgba(80, 80, 95, 0.8)';
      slot.style.borderRadius = '8px';
      slot.style.position = 'relative';
      slot.style.cursor = 'pointer';
      slot.style.display = 'flex';
      slot.style.alignItems = 'center';
      slot.style.justifyContent = 'center';
      slot.style.boxShadow = '0 4px 12px rgba(0,0,0,0.5)';
      slot.style.transition = 'transform 0.15s, border-color 0.15s, box-shadow 0.15s';

      const numberBadge = document.createElement('span');
      numberBadge.innerText = (index + 1).toString();
      numberBadge.style.position = 'absolute';
      numberBadge.style.top = '2px';
      numberBadge.style.left = '4px';
      numberBadge.style.fontSize = '11px';
      numberBadge.style.fontWeight = 'bold';
      numberBadge.style.color = '#ffffff';
      numberBadge.style.textShadow = '0 1px 2px #000';
      slot.appendChild(numberBadge);

      const flaskSvg = this.generatePotionSvg(potion);
      flaskSvg.style.width = '32px';
      flaskSvg.style.height = '32px';
      slot.appendChild(flaskSvg);

      const cdOverlay = document.createElement('div');
      cdOverlay.className = 'cd-overlay';
      cdOverlay.style.position = 'absolute';
      cdOverlay.style.bottom = '0';
      cdOverlay.style.left = '0';
      cdOverlay.style.width = '100%';
      cdOverlay.style.height = '0%';
      cdOverlay.style.background = 'rgba(0, 0, 0, 0.65)';
      cdOverlay.style.borderRadius = '6px';
      cdOverlay.style.pointerEvents = 'none';
      slot.appendChild(cdOverlay);

      slot.addEventListener('pointerdown', (e) => {
        e.stopPropagation();
        if (this.onSlotSelectedCallback) {
          this.onSlotSelectedCallback(index);
        }
      });

      this.hotbarContainer.appendChild(slot);
      this.slotElements.push(slot);
    });
  }

  public update(player: PlayerState, inventory: PotionInventory): void {
    const hearts = player.getDisplayHearts();
    this.renderHearts(hearts.full, hearts.half, hearts.empty);

    const activeIndex = inventory.getActiveSlotIndex();
    const slots = inventory.getSlots();

    this.slotElements.forEach((slot, idx) => {
      const isActive = idx === activeIndex;
      if (isActive) {
        slot.style.borderColor = '#ffd700';
        slot.style.boxShadow = '0 0 14px rgba(255, 215, 0, 0.8), inset 0 0 6px rgba(255, 215, 0, 0.3)';
        slot.style.transform = 'scale(1.1)';
      } else {
        slot.style.borderColor = 'rgba(80, 80, 95, 0.8)';
        slot.style.boxShadow = '0 4px 12px rgba(0,0,0,0.5)';
        slot.style.transform = 'scale(1.0)';
      }

      const potionType = slots[idx];
      const cdPercent = inventory.getCooldownPercent(potionType);
      const cdOverlay = slot.querySelector('.cd-overlay') as HTMLDivElement;
      if (cdOverlay) {
        cdOverlay.style.height = `${cdPercent * 100}%`;
      }
    });
  }

  private renderHearts(full: number, half: number, empty: number): void {
    this.heartsContainer.innerHTML = '';

    for (let i = 0; i < full; i++) {
      this.heartsContainer.appendChild(this.createHeartSvg('full'));
    }
    for (let i = 0; i < half; i++) {
      this.heartsContainer.appendChild(this.createHeartSvg('half'));
    }
    for (let i = 0; i < empty; i++) {
      this.heartsContainer.appendChild(this.createHeartSvg('empty'));
    }
  }

  private createHeartSvg(type: 'full' | 'half' | 'empty'): SVGSVGElement {
    const svg = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
    svg.setAttribute('viewBox', '0 0 24 24');
    svg.setAttribute('width', '20');
    svg.setAttribute('height', '20');
    svg.style.filter = 'drop-shadow(0 2px 4px rgba(0,0,0,0.8))';

    const path = document.createElementNS('http://www.w3.org/2000/svg', 'path');
    path.setAttribute(
      'd',
      'M12 21.35l-1.45-1.32C5.4 15.36 2 12.28 2 8.5 2 5.42 4.42 3 7.5 3c1.74 0 3.41.81 4.5 2.09C13.09 3.81 14.76 3 16.5 3 19.58 3 22 5.42 22 8.5c0 3.78-3.4 6.86-8.55 11.54L12 21.35z'
    );

    if (type === 'full') {
      path.setAttribute('fill', '#e62234');
      path.setAttribute('stroke', '#4a050c');
      path.setAttribute('stroke-width', '1.5');
      svg.appendChild(path);
    } else if (type === 'half') {
      const defs = document.createElementNS('http://www.w3.org/2000/svg', 'defs');
      const clip = document.createElementNS('http://www.w3.org/2000/svg', 'clipPath');
      clip.id = 'half-clip';
      const rect = document.createElementNS('http://www.w3.org/2000/svg', 'rect');
      rect.setAttribute('x', '0');
      rect.setAttribute('y', '0');
      rect.setAttribute('width', '12');
      rect.setAttribute('height', '24');
      clip.appendChild(rect);
      defs.appendChild(clip);
      svg.appendChild(defs);

      const emptyBg = path.cloneNode(true) as SVGPathElement;
      emptyBg.setAttribute('fill', '#261b1c');
      emptyBg.setAttribute('stroke', '#4a050c');
      emptyBg.setAttribute('stroke-width', '1.5');
      svg.appendChild(emptyBg);

      path.setAttribute('fill', '#e62234');
      path.setAttribute('clip-path', 'url(#half-clip)');
      svg.appendChild(path);
    } else {
      path.setAttribute('fill', '#261b1c');
      path.setAttribute('stroke', '#4a050c');
      path.setAttribute('stroke-width', '1.5');
      svg.appendChild(path);
    }

    return svg;
  }

  private generatePotionSvg(potion: PotionType): SVGSVGElement {
    const config = POTION_CONFIGS[potion];
    const hex = '#' + config.colorHex.toString(16).padStart(6, '0');

    const svg = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
    svg.setAttribute('viewBox', '0 0 32 32');

    const cork = document.createElementNS('http://www.w3.org/2000/svg', 'rect');
    cork.setAttribute('x', '13');
    cork.setAttribute('y', '3');
    cork.setAttribute('width', '6');
    cork.setAttribute('height', '4');
    cork.setAttribute('fill', '#8d5524');
    cork.setAttribute('rx', '1');
    svg.appendChild(cork);

    const neck = document.createElementNS('http://www.w3.org/2000/svg', 'rect');
    neck.setAttribute('x', '14');
    neck.setAttribute('y', '7');
    neck.setAttribute('width', '4');
    neck.setAttribute('height', '6');
    neck.setAttribute('fill', 'rgba(255,255,255,0.7)');
    svg.appendChild(neck);

    const bulb = document.createElementNS('http://www.w3.org/2000/svg', 'circle');
    bulb.setAttribute('cx', '16');
    bulb.setAttribute('cy', '20');
    bulb.setAttribute('r', '9');
    bulb.setAttribute('fill', 'rgba(255,255,255,0.3)');
    bulb.setAttribute('stroke', 'rgba(255,255,255,0.8)');
    bulb.setAttribute('stroke-width', '1.5');
    svg.appendChild(bulb);

    const liquid = document.createElementNS('http://www.w3.org/2000/svg', 'circle');
    liquid.setAttribute('cx', '16');
    liquid.setAttribute('cy', '21');
    liquid.setAttribute('r', '7.5');
    liquid.setAttribute('fill', hex);
    liquid.style.filter = `drop-shadow(0 0 5px ${hex})`;
    svg.appendChild(liquid);

    const highlight = document.createElementNS('http://www.w3.org/2000/svg', 'ellipse');
    highlight.setAttribute('cx', '13.5');
    highlight.setAttribute('cy', '17');
    highlight.setAttribute('rx', '2');
    highlight.setAttribute('ry', '4');
    highlight.setAttribute('fill', 'rgba(255,255,255,0.7)');
    highlight.setAttribute('transform', 'rotate(-30 13.5 17)');
    svg.appendChild(highlight);

    return svg;
  }

  public show(): void {
    this.container.style.display = 'block';
  }

  public hide(): void {
    this.container.style.display = 'none';
  }
}
