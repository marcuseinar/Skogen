import { defineConfig } from 'vite';

// GitHub Pages serves the repo at /Skogen/ — use that base for production
// builds so asset paths resolve correctly. Local dev stays at /.
const base = process.env.NODE_ENV === 'production' ? '/Skogen/' : '/';

export default defineConfig({
  root: '.',
  publicDir: 'public',
  base,
  build: {
    outDir: 'dist',
    assetsDir: 'assets',
    emptyOutDir: true,
  },
  server: {
    port: 3000,
    strictPort: true,
  },
});
