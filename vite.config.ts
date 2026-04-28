import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import wasm from 'vite-plugin-wasm'
import topLevelAwait from 'vite-plugin-top-level-await'

// Base path: GitHub Pages serves user/org sites at /<repo-name>/.
// Set BASE_PATH at build time (e.g. BASE_PATH=/clean-dashboard/ npm run build).
// Defaults to "/" for local dev.
const base = process.env.BASE_PATH || '/'

// https://vitejs.dev/config/
export default defineConfig({
  base,
  plugins: [
    react(),
    wasm(),
    topLevelAwait()
  ],
  server: {
    port: 3000,
    open: true
  },
  build: {
    target: 'esnext'
  },
  worker: {
    format: 'es',
    plugins: () => [
      wasm(),
      topLevelAwait()
    ]
  },
  optimizeDeps: {
    exclude: ['energy-simulator']
  }
})
