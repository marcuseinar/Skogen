import * as THREE from 'three';

const PI2 = Math.PI * 2;

function lerpAngle(a, b, t) {
  let diff = ((b - a) % PI2 + PI2) % PI2;
  if (diff > Math.PI) diff -= PI2;
  return a + diff * t;
}

export class ThirdPersonCamera {
  constructor(camera, playerGroup, input) {
    this.camera = camera;
    this.target = playerGroup;    // THREE.Group (the player root)
    this.input  = input;

    // Spherical-ish camera angles
    this.azimuth   = Math.PI;     // start behind player
    this.elevation = 0.32;        // ~18° up

    this.DISTANCE  = 6.5;
    this.EL_MIN    = -0.05;
    this.EL_MAX    =  0.78;

    // Look-at offset above player feet
    this.lookHeight = 1.4;

    // Current smoothed position & look-at
    this._pos    = new THREE.Vector3();
    this._lookAt = new THREE.Vector3();

    // Initialised flag (skip lerp on first frame)
    this._initialised = false;
  }

  get lookForward() {
    // World-space forward vector of the camera (horizontal plane)
    return new THREE.Vector3(
      Math.sin(this.azimuth), 0, Math.cos(this.azimuth)
    );
  }

  get lookRight() {
    const f = this.lookForward;
    return new THREE.Vector3(f.z, 0, -f.x);
  }

  update(dt) {
    // ── Mouse look ──────────────────────────────────────────
    if (this.input.pointerLocked) {
      const m = this.input.consumeMouseMovement();
      this.azimuth   -= m.x * 0.0028;
      this.elevation  = THREE.MathUtils.clamp(
        this.elevation - m.y * 0.0028,
        this.EL_MIN,
        this.EL_MAX
      );
    } else {
      // Auto drift behind player when unlocked and player is moving
      // (handled in Player.js via camCtrl.azimuth nudge)
    }

    // ── Ideal position ───────────────────────────────────────
    const playerPos = this.target.position;
    const lookAtPt  = playerPos.clone().add(new THREE.Vector3(0, this.lookHeight, 0));

    const sinEl = Math.sin(this.elevation);
    const cosEl = Math.cos(this.elevation);
    const offset = new THREE.Vector3(
      Math.sin(this.azimuth) * cosEl * this.DISTANCE,
      sinEl                           * this.DISTANCE,
      Math.cos(this.azimuth) * cosEl  * this.DISTANCE
    );

    const idealPos = lookAtPt.clone().add(offset);

    // ── Smooth follow ────────────────────────────────────────
    if (!this._initialised) {
      this._pos.copy(idealPos);
      this._lookAt.copy(lookAtPt);
      this._initialised = true;
    }

    const smooth = Math.min(1, dt * 12);
    this._pos.lerp(idealPos, smooth);
    this._lookAt.lerp(lookAtPt, smooth);

    this.camera.position.copy(this._pos);
    this.camera.lookAt(this._lookAt);
  }
}
