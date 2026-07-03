export class InputManager {
  constructor(domElement) {
    this.dom = domElement;

    // Held keys
    this.keys = new Set();

    // Just-pressed this frame (cleared each frame by caller)
    this.justPressed = new Set();

    // Mouse
    this.mouseMovement = { x: 0, y: 0 };
    this.mouseDown = false;
    this.mouseJustDown = false;
    this.pointerLocked = false;

    window.addEventListener('keydown', e => {
      if (!this.keys.has(e.code)) this.justPressed.add(e.code);
      this.keys.add(e.code);
      // Prevent space from scrolling page
      if (e.code === 'Space') e.preventDefault();
    });
    window.addEventListener('keyup', e => this.keys.delete(e.code));

    domElement.addEventListener('mousedown', e => {
      if (e.button === 0) {
        this.mouseDown = true;
        this.mouseJustDown = true;
        domElement.requestPointerLock();
      }
    });
    window.addEventListener('mouseup', e => {
      if (e.button === 0) this.mouseDown = false;
    });
    document.addEventListener('mousemove', e => {
      if (document.pointerLockElement === domElement) {
        this.mouseMovement.x += e.movementX;
        this.mouseMovement.y += e.movementY;
      }
    });
    document.addEventListener('pointerlockchange', () => {
      this.pointerLocked = document.pointerLockElement === domElement;
    });
  }

  isKey(code)        { return this.keys.has(code); }
  wasJustPressed(code) { return this.justPressed.has(code); }

  // Call once per frame at the END of the frame to clear transient state
  flush() {
    this.justPressed.clear();
    this.mouseMovement.x = 0;
    this.mouseMovement.y = 0;
    this.mouseJustDown = false;
  }

  consumeMouseMovement() {
    const m = { x: this.mouseMovement.x, y: this.mouseMovement.y };
    this.mouseMovement.x = 0;
    this.mouseMovement.y = 0;
    return m;
  }
}
