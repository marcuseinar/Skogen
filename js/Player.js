import * as THREE from 'three';
import { GLTFLoader } from 'three/addons/loaders/GLTFLoader.js';

const WALK_SPEED = 3.8;
const RUN_SPEED  = 7.2;
const TURN_SPEED = 14;   // rad/s

// Animation names in RobotExpressive.glb (fallback to any available)
const ANIM = {
  idle:   ['Idle',   'idle',   'TPose'],
  walk:   ['Walking','walk',   'Walk'],
  run:    ['Running','run',    'Run'],
  attack: ['Punch',  'Attack', 'attack', 'Wave'],
  death:  ['Death',  'death',  'Die'],
};

export class Player {
  constructor(scene) {
    this.scene = scene;

    this.group = new THREE.Group();
    scene.add(this.group);

    this.maxHealth = 100;
    this.health    = 100;
    this.alive     = true;

    // Physics-ish
    this.velocity = new THREE.Vector3();

    // Combat
    this.attackCooldown  = 0;
    this.attackDuration  = 0;
    this.attackAnimTime  = 0.7;  // seconds
    this.attackDamage    = 25;
    this.attackRange     = 2.2;

    this.comboCount  = 0;
    this.comboTimer  = 0;
    this.comboWindow = 1.4;    // seconds to chain hits

    // Hit-flash
    this._flashTimer = 0;
    this._origMaterials = [];

    this.mixer   = null;
    this.actions = {};
    this._currentAction = null;

    this._loaded = false;
  }

  get position() { return this.group.position; }

  async init(onProgress) {
    const MODEL_URL = '/models/RobotExpressive.glb';

    const loader = new GLTFLoader();
    let gltf;
    try {
      gltf = await loadGLTF(loader, MODEL_URL, onProgress);
    } catch (e) {
      console.error('[Player] Could not load model at', MODEL_URL, e);
      gltf = null;
    }

    if (!gltf) {
      // We need a real model — show a warning and use a visible temp stand-in
      // so the camera/movement demo is still functional.
      console.warn('[Player] Using emergency placeholder — replace /public/models/RobotExpressive.glb');
      this._buildPlaceholder();
      return;
    }

    this._setupModel(gltf);
  }

  _setupModel(gltf) {
    const model = gltf.scene;
    model.scale.setScalar(0.9);
    model.position.y = 0;
    // Fix orientations that bake a 180° offset
    // (RobotExpressive faces -Z, so no adjustment needed)
    model.traverse(child => {
      if (child.isMesh) {
        child.castShadow    = true;
        child.receiveShadow = true;
        // Store original materials for hit-flash
        this._origMaterials.push({ mesh: child, mat: child.material });
      }
    });

    this.group.add(model);
    this._model = model;

    // Animation mixer
    this.mixer = new THREE.AnimationMixer(model);
    for (const clip of gltf.animations) {
      this.actions[clip.name] = this.mixer.clipAction(clip);
    }

    console.log('[Player] animations:', Object.keys(this.actions));

    // Start idle
    this._playAction(this._resolveAnim('idle'));
    this._loaded = true;
  }

  _buildPlaceholder() {
    // Temporary orange stand-in so the scene isn't empty
    const body = new THREE.Mesh(
      new THREE.CapsuleGeometry(0.35, 1.1, 4, 8),
      new THREE.MeshLambertMaterial({ color: 0xe07020 }),
    );
    body.position.y = 1.05;
    body.castShadow = true;
    this.group.add(body);

    const head = new THREE.Mesh(
      new THREE.SphereGeometry(0.3, 8, 6),
      new THREE.MeshLambertMaterial({ color: 0xf0c090 }),
    );
    head.position.y = 2.05;
    head.castShadow = true;
    this.group.add(head);

    this._model = body;
    this._loaded = true;
  }

  // ── Animation helpers ────────────────────────────────────

  _resolveAnim(key) {
    for (const name of (ANIM[key] || [])) {
      if (this.actions[name]) return name;
    }
    const fallback = Object.keys(this.actions)[0];
    return fallback || null;
  }

  _playAction(name, options = {}) {
    if (!name || !this.actions[name]) return;
    const next = this.actions[name];
    if (this._currentAction === next && !options.forceRestart) return;

    const fadeDuration = options.fadeDuration ?? 0.25;
    if (this._currentAction && this._currentAction !== next) {
      this._currentAction.fadeOut(fadeDuration);
    }
    next.reset()
        .setEffectiveTimeScale(options.timeScale ?? 1)
        .setEffectiveWeight(1)
        .fadeIn(fadeDuration)
        .play();

    if (options.once) next.setLoop(THREE.LoopOnce, 1).clampWhenFinished = true;
    this._currentAction = next;
  }

  // ── Update ───────────────────────────────────────────────

  update(dt, input, camAzimuth, enemies) {
    if (!this.alive) { this.mixer?.update(dt); return; }

    // ── Move input ──────────────────────────────────────────
    const fwd   = new THREE.Vector3(Math.sin(camAzimuth), 0,  Math.cos(camAzimuth));
    const right = new THREE.Vector3(Math.cos(camAzimuth), 0, -Math.sin(camAzimuth));

    let moveDir = new THREE.Vector3();
    if (input.isKey('KeyW') || input.isKey('ArrowUp'))    moveDir.addScaledVector(fwd,   1);
    if (input.isKey('KeyS') || input.isKey('ArrowDown'))  moveDir.addScaledVector(fwd,  -1);
    if (input.isKey('KeyA') || input.isKey('ArrowLeft'))  moveDir.addScaledVector(right, -1);
    if (input.isKey('KeyD') || input.isKey('ArrowRight')) moveDir.addScaledVector(right,  1);

    const isRunning = input.isKey('ShiftLeft') || input.isKey('ShiftRight');
    const speed     = isRunning ? RUN_SPEED : WALK_SPEED;
    const moving    = moveDir.length() > 0.01;

    if (moving) {
      moveDir.normalize();
      this.group.position.addScaledVector(moveDir, speed * dt);

      // Smooth rotation toward movement dir
      const targetYaw = Math.atan2(moveDir.x, moveDir.z);
      this.group.rotation.y = lerpAngle(this.group.rotation.y, targetYaw, TURN_SPEED * dt);
    }

    // Ground clamp (simple flat world)
    this.group.position.y = 0;

    // ── Attack ───────────────────────────────────────────────
    this.attackCooldown = Math.max(0, this.attackCooldown - dt);

    const attackTrigger = input.wasJustPressed('Space')
      || input.wasJustPressed('KeyE')
      || input.mouseJustDown;

    if (attackTrigger && this.attackCooldown <= 0) {
      this._doAttack(enemies);
    }
    if (this.attackDuration > 0) this.attackDuration -= dt;

    // ── Combo timer ──────────────────────────────────────────
    if (this.comboTimer > 0) {
      this.comboTimer -= dt;
      if (this.comboTimer <= 0) this.comboCount = 0;
    }

    // ── Hit flash ────────────────────────────────────────────
    if (this._flashTimer > 0) {
      this._flashTimer -= dt;
      if (this._flashTimer <= 0) this._clearFlash();
    }

    // ── Animation state machine ──────────────────────────────
    if (this.mixer) {
      this.mixer.update(dt);

      const isAttacking = this.attackDuration > 0;
      if (!isAttacking) {
        if (moving) {
          this._playAction(this._resolveAnim(isRunning ? 'run' : 'walk'));
        } else {
          this._playAction(this._resolveAnim('idle'));
        }
      }
    }
  }

  _doAttack(enemies) {
    // Combo chain
    const baseAnim   = this._resolveAnim('attack');
    const timeScales = [1.0, 1.35, 0.80];   // light / medium / heavy
    const ts         = timeScales[this.comboCount % timeScales.length];
    const damage     = this.attackDamage * (1 + this.comboCount * 0.4);

    this._playAction(baseAnim, { fadeDuration: 0.1, once: true, timeScale: ts, forceRestart: true });

    this.attackCooldown = this.attackAnimTime / ts;
    this.attackDuration = this.attackAnimTime / ts;

    // Hit enemies in range
    let hit = false;
    if (enemies) {
      for (const enemy of enemies) {
        if (!enemy.alive) continue;
        const d = this.group.position.distanceTo(enemy.group.position);
        if (d <= this.attackRange) {
          enemy.takeDamage(damage, this.group.position);
          hit = true;
        }
      }
    }

    // Combo tracking
    if (hit) {
      this.comboCount++;
      this.comboTimer = this.comboWindow;
    } else {
      // Miss — reset combo but still play animation
      this.comboCount = 0;
    }
  }

  takeDamage(amount) {
    if (!this.alive) return;
    this.health = Math.max(0, this.health - amount);
    this._triggerFlash(new THREE.Color(1, 0.2, 0.2));
    if (this.health <= 0) {
      this.alive = false;
      this._playAction(this._resolveAnim('death'), { fadeDuration: 0.15, once: true });
    }
  }

  _triggerFlash(color) {
    this._flashTimer = 0.18;
    this._origMaterials.forEach(({ mesh }) => {
      const m = mesh.material.clone();
      m.emissive = color;
      m.emissiveIntensity = 0.6;
      mesh.material = m;
    });
  }

  _clearFlash() {
    this._origMaterials.forEach(({ mesh, mat }) => {
      mesh.material = mat;
    });
  }
}

// ── Helpers ──────────────────────────────────────────────────

function loadGLTF(loader, url, onProgress) {
  return new Promise((resolve, reject) => {
    loader.load(url, resolve, onProgress, reject);
  });
}

const PI2 = Math.PI * 2;
function lerpAngle(a, b, t) {
  let d = ((b - a) % PI2 + PI2) % PI2;
  if (d > Math.PI) d -= PI2;
  return a + d * Math.min(1, t);
}
