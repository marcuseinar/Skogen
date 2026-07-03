import * as THREE from 'three';
import { GLTFLoader } from 'three/addons/loaders/GLTFLoader.js';

const CHASE_SPEED  = 2.6;
const TURN_SPEED   = 8;
const CHASE_RADIUS = 16;
const ATTACK_RANGE = 1.9;
const ATTACK_RATE  = 1.6;    // seconds between attacks
const ATTACK_DMG   = 12;
const KNOCKBACK    = 4.5;
const PI2          = Math.PI * 2;

let _sharedGltf  = null;
let _loadPromise = null;

function loadSharedModel() {
  if (_loadPromise) return _loadPromise;
  _loadPromise = new Promise((resolve) => {
    const loader = new GLTFLoader();
    loader.load(`${import.meta.env.BASE_URL}models/RobotExpressive.glb`,
      g => { _sharedGltf = g; resolve(g); },
      undefined,
      () => resolve(null),
    );
  });
  return _loadPromise;
}

export class Enemy {
  constructor(scene, position) {
    this.scene = scene;

    this.group = new THREE.Group();
    this.group.position.copy(position);
    scene.add(this.group);

    this.maxHealth = 60;
    this.health    = 60;
    this.alive     = true;

    this.state          = 'idle';    // idle | chase | attack | dead
    this.attackTimer    = 0;
    this.velocity       = new THREE.Vector3();
    this.knockbackVel   = new THREE.Vector3();

    this._flashTimer    = 0;
    this._origMaterials = [];

    this.mixer   = null;
    this.actions = {};
    this._currentAction = null;

    this._loaded = false;
    this._tint   = new THREE.Color(0.6, 0.15, 0.15);  // dark red tint
  }

  async init() {
    const gltf = await loadSharedModel();
    if (!gltf) {
      this._buildPlaceholder();
      return;
    }
    this._setupModel(gltf);
  }

  _setupModel(gltf) {
    const model = gltf.scene.clone(true);
    model.scale.setScalar(0.75);

    // Dark red palette to distinguish from player
    model.traverse(child => {
      if (child.isMesh) {
        child.castShadow    = true;
        child.receiveShadow = true;
        const m = child.material.clone();
        // Tint the model dark red
        m.color.multiply(new THREE.Color(0.55, 0.20, 0.20));
        child.material = m;
        this._origMaterials.push({ mesh: child, mat: m });
      }
    });

    this.group.add(model);
    this._model = model;

    // Rebuild animation mixer on the clone
    this.mixer = new THREE.AnimationMixer(model);
    for (const clip of gltf.animations) {
      const action = this.mixer.clipAction(
        THREE.AnimationClip.findByName(gltf.animations, clip.name) ?? clip,
        model,
      );
      this.actions[clip.name] = action;
    }

    this._playAction(this._resolveAnim('idle'));
    this._loaded = true;
  }

  _buildPlaceholder() {
    const body = new THREE.Mesh(
      new THREE.CapsuleGeometry(0.3, 1.0, 4, 8),
      new THREE.MeshLambertMaterial({ color: 0x7a1010 }),
    );
    body.position.y = 0.9;
    body.castShadow = true;
    this.group.add(body);

    const head = new THREE.Mesh(
      new THREE.SphereGeometry(0.28, 8, 6),
      new THREE.MeshLambertMaterial({ color: 0x500808 }),
    );
    head.position.y = 1.9;
    head.castShadow = true;
    this.group.add(head);

    this._model = body;
    this._origMaterials.push({
      mesh: body, mat: body.material,
    });
    this._loaded = true;
  }

  // ── Anim helpers ─────────────────────────────────────────────

  _resolveAnim(key) {
    const ANIM = {
      idle:   ['Idle',   'idle',   'TPose', 'Standing'],
      walk:   ['Walking','walk',   'Walk'],
      run:    ['Running','run',    'Run'],
      attack: ['Punch',  'Attack', 'attack', 'Wave'],
      death:  ['Death',  'death',  'Die'],
    };
    for (const name of (ANIM[key] || [])) {
      if (this.actions[name]) return name;
    }
    return Object.keys(this.actions)[0] ?? null;
  }

  _playAction(name, options = {}) {
    if (!name || !this.actions[name]) return;
    const next = this.actions[name];
    if (this._currentAction === next && !options.forceRestart) return;

    const fade = options.fadeDuration ?? 0.2;
    if (this._currentAction && this._currentAction !== next) {
      this._currentAction.fadeOut(fade);
    }
    next.reset()
        .setEffectiveTimeScale(options.timeScale ?? 1)
        .setEffectiveWeight(1)
        .fadeIn(fade)
        .play();
    if (options.once) {
      next.setLoop(THREE.LoopOnce, 1);
      next.clampWhenFinished = true;
    }
    this._currentAction = next;
  }

  // ── Update ───────────────────────────────────────────────────

  update(dt, player) {
    if (!this.alive) {
      this.mixer?.update(dt);
      return;
    }

    const toPlayer = new THREE.Vector3().subVectors(player.group.position, this.group.position);
    const dist     = toPlayer.length();

    // ── State transitions ─────────────────────────────────────
    if (dist < ATTACK_RANGE) {
      this.state = 'attack';
    } else if (dist < CHASE_RADIUS) {
      this.state = 'chase';
    } else {
      this.state = 'idle';
    }

    // ── State behaviour ───────────────────────────────────────
    let moving = false;

    if (this.state === 'chase') {
      const dir = toPlayer.clone().normalize();
      this.group.position.addScaledVector(dir, CHASE_SPEED * dt);

      const targetYaw = Math.atan2(dir.x, dir.z);
      this.group.rotation.y = lerpAngle(this.group.rotation.y, targetYaw, TURN_SPEED * dt);
      moving = true;
    }

    if (this.state === 'attack') {
      // Face player
      const dir = toPlayer.clone().normalize();
      const targetYaw = Math.atan2(dir.x, dir.z);
      this.group.rotation.y = lerpAngle(this.group.rotation.y, targetYaw, TURN_SPEED * dt);

      this.attackTimer -= dt;
      if (this.attackTimer <= 0 && player.alive) {
        this.attackTimer = ATTACK_RATE;
        player.takeDamage(ATTACK_DMG);
        this._playAction(this._resolveAnim('attack'), {
          fadeDuration: 0.1, once: true, forceRestart: true,
        });
      }
    }

    // Knockback
    if (this.knockbackVel.length() > 0.01) {
      this.group.position.addScaledVector(this.knockbackVel, dt);
      this.knockbackVel.multiplyScalar(1 - Math.min(1, dt * 10));
    }

    // Ground clamp
    this.group.position.y = 0;

    // ── Flash ─────────────────────────────────────────────────
    if (this._flashTimer > 0) {
      this._flashTimer -= dt;
      if (this._flashTimer <= 0) this._clearFlash();
    }

    // ── Animation ─────────────────────────────────────────────
    if (this.mixer) {
      this.mixer.update(dt);
      const isAttacking = this.state === 'attack' && this.attackTimer > ATTACK_RATE - 0.6;
      if (!isAttacking) {
        this._playAction(this._resolveAnim(moving ? 'walk' : 'idle'));
      }
    }
  }

  takeDamage(amount, fromPos) {
    if (!this.alive) return;
    this.health = Math.max(0, this.health - amount);

    // Knockback away from attacker
    if (fromPos) {
      const dir = new THREE.Vector3().subVectors(this.group.position, fromPos).normalize();
      this.knockbackVel.copy(dir.multiplyScalar(KNOCKBACK));
    }

    this._triggerFlash(new THREE.Color(1, 0.3, 0.3));

    if (this.health <= 0) {
      this.alive = false;
      this.state = 'dead';
      this._playAction(this._resolveAnim('death'), { fadeDuration: 0.1, once: true });
      setTimeout(() => {
        this.scene.remove(this.group);
      }, 3000);
    }
  }

  _triggerFlash(color) {
    this._flashTimer = 0.15;
    this._origMaterials.forEach(({ mesh }) => {
      const m = mesh.material.clone();
      m.emissive    = color;
      m.emissiveIntensity = 0.8;
      mesh.material = m;
    });
  }

  _clearFlash() {
    this._origMaterials.forEach(({ mesh, mat }) => {
      mesh.material = mat;
    });
  }

  // World-space position to attach the health bar label (above head)
  get labelPosition() {
    return this.group.position.clone().add(new THREE.Vector3(0, 2.4, 0));
  }
}

function lerpAngle(a, b, t) {
  let d = ((b - a) % PI2 + PI2) % PI2;
  if (d > Math.PI) d -= PI2;
  return a + d * Math.min(1, t);
}
