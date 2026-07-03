import * as THREE from 'three';
import { World }             from './World.js';
import { Player }            from './Player.js';
import { Enemy }             from './Enemy.js';
import { ThirdPersonCamera } from './ThirdPersonCamera.js';
import { InputManager }      from './InputManager.js';
import { UI }                from './UI.js';

// Enemy spawn ring positions (world-space XZ, Y=0)
const SPAWN_POSITIONS = [
  [  8,  8 ], [ -9,  7 ], [  7, -8 ],
  [ -8, -7 ], [ 12,  3 ], [ -5, 12 ],
];

async function init() {
  // ── Renderer ─────────────────────────────────────────────────
  const renderer = new THREE.WebGLRenderer({ antialias: true });
  renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
  renderer.setSize(window.innerWidth, window.innerHeight);
  renderer.shadowMap.enabled    = true;
  renderer.shadowMap.type       = THREE.PCFSoftShadowMap;
  renderer.toneMapping          = THREE.ACESFilmicToneMapping;
  renderer.toneMappingExposure  = 1.1;
  renderer.outputColorSpace     = THREE.SRGBColorSpace;
  document.body.appendChild(renderer.domElement);

  // ── Scene & Camera ───────────────────────────────────────────
  const scene  = new THREE.Scene();
  const camera = new THREE.PerspectiveCamera(60, innerWidth / innerHeight, 0.1, 400);

  // ── Input ────────────────────────────────────────────────────
  const input = new InputManager(renderer.domElement);

  // ── Loading UI refs ───────────────────────────────────────────
  const loadingEl  = document.getElementById('loading');
  const loadBarEl  = document.getElementById('load-bar');
  const loadTextEl = document.getElementById('load-text');
  const lockPrompt = document.getElementById('lock-prompt');

  function setLoadProgress(fraction, text) {
    if (loadBarEl)  loadBarEl.style.width  = `${Math.round(fraction * 100)}%`;
    if (loadTextEl) loadTextEl.textContent = text;
  }

  // ── World ────────────────────────────────────────────────────
  setLoadProgress(0.05, 'Building world…');
  new World(scene);

  // ── Player ───────────────────────────────────────────────────
  const player = new Player(scene);

  setLoadProgress(0.2, 'Loading character…');
  await player.init(e => {
    if (e.total) setLoadProgress(0.2 + (e.loaded / e.total) * 0.5, 'Loading character…');
  });

  // ── Enemies ───────────────────────────────────────────────────
  setLoadProgress(0.75, 'Spawning enemies…');
  const enemies = [];
  for (const [x, z] of SPAWN_POSITIONS) {
    const e = new Enemy(scene, new THREE.Vector3(x, 0, z));
    await e.init();
    enemies.push(e);
  }

  setLoadProgress(1, 'Ready!');

  // ── Camera controller ─────────────────────────────────────────
  const camCtrl = new ThirdPersonCamera(camera, player.group, input);

  // ── DOM UI (enemy health bars) ────────────────────────────────
  const ui = new UI(camera, renderer);

  // ── Hide loading screen ───────────────────────────────────────
  setTimeout(() => {
    loadingEl?.classList.add('hidden');
    setTimeout(() => { if (loadingEl) loadingEl.style.display = 'none'; }, 700);
  }, 300);

  window.gameReady = true;

  // ── Pointer-lock prompt ───────────────────────────────────────
  document.addEventListener('pointerlockchange', () => {
    const locked = !!document.pointerLockElement;
    lockPrompt?.classList.toggle('hidden', locked);
  });

  // ── HUD refs ──────────────────────────────────────────────────
  const playerHPFill = document.getElementById('player-hp-fill');
  const killNumEl    = document.getElementById('kill-num');
  const comboCnt     = document.getElementById('combo-count');
  const comboEl      = document.getElementById('combo');

  let kills = 0;
  const prevAlive = new Array(enemies.length).fill(true);

  function updateHUD() {
    if (playerHPFill) {
      playerHPFill.style.width = `${(player.health / player.maxHealth) * 100}%`;
    }

    // Count fresh kills this frame
    enemies.forEach((e, i) => {
      if (prevAlive[i] && !e.alive) kills++;
      prevAlive[i] = e.alive;
    });
    if (killNumEl) killNumEl.textContent = kills;

    const c = player.comboCount;
    if (comboEl) comboEl.classList.toggle('hidden', c < 2);
    if (comboCnt) comboCnt.textContent = c;
  }

  // ── Resize ────────────────────────────────────────────────────
  window.addEventListener('resize', () => {
    camera.aspect = innerWidth / innerHeight;
    camera.updateProjectionMatrix();
    renderer.setSize(innerWidth, innerHeight);
  });

  // ── Game loop ─────────────────────────────────────────────────
  const clock  = new THREE.Clock();
  const living = () => enemies.filter(e => e.alive);

  renderer.setAnimationLoop(() => {
    const dt = Math.min(clock.getDelta(), 0.05);

    camCtrl.update(dt);
    player.update(dt, input, camCtrl.azimuth, living());

    for (const e of enemies) e.update(dt, player);

    ui.update(enemies);
    updateHUD();
    input.flush();

    renderer.render(scene, camera);
  });
}

init().catch(console.error);
