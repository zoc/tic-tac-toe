import { defineConfig } from 'vite';
import wasm from 'vite-plugin-wasm';
import topLevelAwait from 'vite-plugin-top-level-await';

export default defineConfig({
  plugins: [wasm(), topLevelAwait()],
  server: {
    fs: {
      // Allow serving pkg/ which is outside src/ but inside project root
      allow: ['.']
    }
  }
});
