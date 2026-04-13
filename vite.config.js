import { defineConfig } from 'vite';
import wasm from 'vite-plugin-wasm';

// Note: vite-plugin-top-level-await is not needed when build.target = 'esnext'
// (modern browsers support top-level await natively)

export default defineConfig({
  plugins: [wasm()],
  build: {
    target: 'esnext'  // enables top-level await without plugin; targets modern browsers
  },
  server: {
    fs: {
      // Allow serving pkg/ which is outside src/ but inside project root
      allow: ['.']
    }
  }
});
