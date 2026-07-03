import { defineConfig } from 'vite';

export default defineConfig({
  root: '.',
  publicDir: 'public',
  build: {
    outDir: 'dist',
    assetsDir: 'assets',
    emptyOutDir: true,
  },
  server: {
    port: 3000,
    strictPort: true,
    headers: {
      // Allow SharedArrayBuffer if ever needed; no-op otherwise
      'Cross-Origin-Opener-Policy': 'same-origin',
      'Cross-Origin-Embedder-Policy': 'require-corp',
    },
  },
  // Avoid Vite trying to resolve Rust/WASM artefacts
  optimizeDeps: {
    exclude: [],
  },
});
