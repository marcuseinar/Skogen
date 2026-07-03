// DOM-based UI: enemy health bars projected to screen space
export class UI {
  constructor(camera, renderer) {
    this.camera   = camera;
    this.renderer = renderer;
    this.hud      = document.getElementById('hud');
    this._bars    = new Map();   // enemy -> div
  }

  update(enemies) {
    const camera   = this.camera;
    const canvas   = this.renderer.domElement;
    const w        = canvas.clientWidth;
    const h        = canvas.clientHeight;

    const seen = new Set();

    for (const enemy of enemies) {
      if (!enemy.alive) {
        this._removeBar(enemy);
        continue;
      }

      seen.add(enemy);

      // Project world label position to screen
      const worldPos = enemy.labelPosition;
      const ndc = worldPos.clone().project(camera);

      // Behind camera — hide
      if (ndc.z > 1) { this._hideBar(enemy); continue; }

      const sx = (ndc.x *  0.5 + 0.5) * w;
      const sy = (ndc.y * -0.5 + 0.5) * h;

      if (sx < -80 || sx > w + 80 || sy < -20 || sy > h + 20) {
        this._hideBar(enemy);
        continue;
      }

      const bar = this._getOrCreateBar(enemy);
      bar.style.display = 'block';
      bar.style.left    = `${sx}px`;
      bar.style.top     = `${sy}px`;

      const fill = bar.querySelector('.enemy-hp-fill');
      fill.style.width  = `${(enemy.health / enemy.maxHealth) * 100}%`;
    }

    // Clean up bars for enemies no longer in the list
    for (const [enemy, bar] of this._bars) {
      if (!seen.has(enemy)) {
        bar.remove();
        this._bars.delete(enemy);
      }
    }
  }

  _getOrCreateBar(enemy) {
    if (this._bars.has(enemy)) return this._bars.get(enemy);

    const wrapper = document.createElement('div');
    wrapper.className = 'enemy-hp-bar';
    wrapper.innerHTML = `
      <div class="enemy-hp-bg">
        <div class="enemy-hp-fill" style="width:100%"></div>
      </div>`;
    this.hud?.appendChild(wrapper);
    this._bars.set(enemy, wrapper);
    return wrapper;
  }

  _hideBar(enemy) {
    const bar = this._bars.get(enemy);
    if (bar) bar.style.display = 'none';
  }

  _removeBar(enemy) {
    const bar = this._bars.get(enemy);
    if (bar) { bar.remove(); this._bars.delete(enemy); }
  }

  destroy() {
    for (const bar of this._bars.values()) bar.remove();
    this._bars.clear();
  }
}
