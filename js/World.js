import * as THREE from 'three';
import { Sky } from 'three/addons/objects/Sky.js';

export class World {
  constructor(scene) {
    this.scene = scene;
    this._setupFog();
    this._setupLights();
    this._setupSky();
    this._setupGround();
    this._setupProps();
  }

  _setupFog() {
    this.scene.fog = new THREE.FogExp2(0x9ab0c8, 0.016);
  }

  _setupLights() {
    // Warm late-afternoon sun
    this.sun = new THREE.DirectionalLight(0xffe8b0, 2.8);
    this.sun.position.set(25, 45, 15);
    this.sun.castShadow = true;
    const sc = this.sun.shadow.camera;
    sc.near = 0.5; sc.far = 120;
    sc.left = -35; sc.right = 35; sc.top = 35; sc.bottom = -35;
    this.sun.shadow.mapSize.setScalar(2048);
    this.sun.shadow.bias = -0.0005;
    this.sun.shadow.normalBias = 0.02;
    this.scene.add(this.sun);

    // Sky / ground hemisphere
    const hemi = new THREE.HemisphereLight(0x7fa8d4, 0x4a6030, 0.9);
    this.scene.add(hemi);
  }

  _setupSky() {
    const sky = new Sky();
    sky.scale.setScalar(500);
    this.scene.add(sky);

    const u = sky.material.uniforms;
    u['turbidity'].value        = 5;
    u['rayleigh'].value         = 2.2;
    u['mieCoefficient'].value   = 0.004;
    u['mieDirectionalG'].value  = 0.82;

    // Position the sun (phi=elevation, theta=azimuth)
    const sunVec = new THREE.Vector3();
    const phi   = THREE.MathUtils.degToRad(72);   // above horizon
    const theta = THREE.MathUtils.degToRad(220);  // south-west
    sunVec.setFromSphericalCoords(1, phi, theta);
    u['sunPosition'].value.copy(sunVec);

    // Align directional light with sky sun
    this.sun.position.copy(sunVec.clone().multiplyScalar(50));
  }

  _setupGround() {
    const tex = this._makeGrassTex();
    const mat = new THREE.MeshLambertMaterial({ map: tex });
    const geo = new THREE.PlaneGeometry(150, 150, 32, 32);

    // Tiny height ripples for visual depth
    const pos = geo.attributes.position;
    for (let i = 0; i < pos.count; i++) {
      const x = pos.getX(i), z = pos.getZ(i);
      pos.setZ(i, (Math.sin(x * 0.4) + Math.cos(z * 0.4)) * 0.06);
    }
    pos.needsUpdate = true;
    geo.computeVertexNormals();

    const ground = new THREE.Mesh(geo, mat);
    ground.rotation.x = -Math.PI / 2;
    ground.receiveShadow = true;
    this.scene.add(ground);
  }

  _makeGrassTex() {
    const S = 512;
    const c = document.createElement('canvas');
    c.width = c.height = S;
    const ctx = c.getContext('2d');

    // Warm mid-green base
    ctx.fillStyle = '#4e7a38';
    ctx.fillRect(0, 0, S, S);

    // Organic noise blobs
    const rng = () => Math.random();
    for (let i = 0; i < 6000; i++) {
      const x = rng() * S, y = rng() * S, r = rng() * 5 + 1;
      const v = (rng() - 0.5) * 40;
      const gr = Math.max(0, Math.min(255, 122 + v));
      const re = Math.max(0, Math.min(255, 78  + v));
      const bl = Math.max(0, Math.min(255, 56  + v));
      ctx.fillStyle = `rgba(${re},${gr},${bl},0.55)`;
      ctx.beginPath();
      ctx.arc(x, y, r, 0, Math.PI * 2);
      ctx.fill();
    }

    // Scattered dark earth patches
    for (let i = 0; i < 180; i++) {
      const x = rng() * S, y = rng() * S, r = rng() * 20 + 6;
      ctx.fillStyle = 'rgba(28,44,14,0.12)';
      ctx.beginPath(); ctx.arc(x, y, r, 0, Math.PI * 2); ctx.fill();
    }

    // Bright highlights (dewy grass)
    for (let i = 0; i < 400; i++) {
      const x = rng() * S, y = rng() * S;
      ctx.fillStyle = 'rgba(140,200,90,0.18)';
      ctx.fillRect(x, y, 2, 2);
    }

    const tex = new THREE.CanvasTexture(c);
    tex.wrapS = tex.wrapT = THREE.RepeatWrapping;
    tex.repeat.set(18, 18);
    tex.anisotropy = 8;
    return tex;
  }

  _setupProps() {
    const rng = seededRng(42);

    // ── Stones ──────────────────────────────────────────────
    for (let i = 0; i < 14; i++) {
      const angle = rng() * Math.PI * 2;
      const dist  = 10 + rng() * 30;
      const scale = 0.4 + rng() * 0.9;

      const geo = new THREE.DodecahedronGeometry(scale * 0.65, 0);
      const mat = new THREE.MeshLambertMaterial({
        color: new THREE.Color().setHSL(0.08, 0.15, 0.4 + rng() * 0.2),
        flatShading: true,
      });
      const rock = new THREE.Mesh(geo, mat);
      rock.position.set(
        Math.cos(angle) * dist,
        scale * 0.25,
        Math.sin(angle) * dist,
      );
      rock.rotation.set(rng() * 6, rng() * 6, rng() * 6);
      rock.castShadow = true;
      rock.receiveShadow = true;
      this.scene.add(rock);
    }

    // ── Stylised low-poly trees ─────────────────────────────
    for (let i = 0; i < 20; i++) {
      const angle = rng() * Math.PI * 2;
      const dist  = 12 + rng() * 32;
      this._addTree(
        Math.cos(angle) * dist,
        Math.sin(angle) * dist,
        0.55 + rng() * 0.6,
        rng,
      );
    }

    // ── Ruined walls (level dressing) ───────────────────────
    this._addRuins(rng);
  }

  _addTree(x, z, s, rng) {
    const g = new THREE.Group();

    const trunkMat = new THREE.MeshLambertMaterial({
      color: 0x5c3317, flatShading: true,
    });
    const trunk = new THREE.Mesh(
      new THREE.CylinderGeometry(0.14 * s, 0.22 * s, 1.8 * s, 6, 1),
      trunkMat,
    );
    trunk.position.y = 0.9 * s;
    trunk.castShadow = true;
    g.add(trunk);

    const hues = [0.32, 0.30, 0.28];
    for (let j = 0; j < 3; j++) {
      const cone = new THREE.Mesh(
        new THREE.ConeGeometry((1.3 - j * 0.28) * s, (1.0 + j * 0.12) * s, 6, 1),
        new THREE.MeshLambertMaterial({
          color: new THREE.Color().setHSL(hues[j], 0.55, 0.25 + j * 0.04),
          flatShading: true,
        }),
      );
      cone.position.y = (1.8 + j * 0.68) * s;
      cone.rotation.y = rng() * Math.PI;
      cone.castShadow = true;
      g.add(cone);
    }

    g.position.set(x, 0, z);
    g.rotation.y = rng() * Math.PI * 2;
    this.scene.add(g);
  }

  _addRuins(rng) {
    const stoneMat = new THREE.MeshLambertMaterial({
      color: 0x8a8070, flatShading: true,
    });

    const positions = [
      [-15, 8], [16, -12], [-12, -16], [18, 14],
    ];
    for (const [x, z] of positions) {
      const group = new THREE.Group();
      const wallH = 1.2 + rng() * 0.8;
      const wallW = 2.5 + rng() * 1.5;

      const wall = new THREE.Mesh(
        new THREE.BoxGeometry(wallW, wallH, 0.45),
        stoneMat,
      );
      wall.position.y = wallH * 0.5;
      wall.rotation.y = rng() * Math.PI * 0.5;
      wall.castShadow = true;
      wall.receiveShadow = true;
      group.add(wall);

      // Rubble chunks
      for (let i = 0; i < 4; i++) {
        const rb = new THREE.Mesh(
          new THREE.BoxGeometry(0.3 + rng() * 0.4, 0.2 + rng() * 0.3, 0.3 + rng() * 0.4),
          stoneMat,
        );
        rb.position.set((rng() - 0.5) * 2.5, 0.2, (rng() - 0.5) * 2.5);
        rb.rotation.set(rng() * 0.8, rng() * 3, rng() * 0.8);
        rb.castShadow = true;
        group.add(rb);
      }

      group.position.set(x, 0, z);
      this.scene.add(group);
    }
  }
}

// Simple deterministic RNG (mulberry32)
function seededRng(seed) {
  let s = seed >>> 0;
  return function () {
    s += 0x6d2b79f5;
    let t = s;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}
