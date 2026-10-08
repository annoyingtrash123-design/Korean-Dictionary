import { defineConfig } from 'vitest/config';
import preact from '@preact/preset-vite';
import { VitePWA } from 'vite-plugin-pwa';

const base = process.env.BASE_PATH ?? '/Korean-Dictionary/';

export default defineConfig({
  base,
  plugins: [
    preact(),
    VitePWA({
      registerType: 'prompt',
      injectRegister: false,
      manifest: {
        name: 'Korean Dictionary',
        short_name: '한영사전',
        description: 'Offline Korean–English dictionary',
        start_url: base,
        scope: base,
        display: 'standalone',
        background_color: '#faf8f5',
        theme_color: '#2f6f5e',
        icons: [
          { src: 'icon-192.png', sizes: '192x192', type: 'image/png' },
          { src: 'icon-512.png', sizes: '512x512', type: 'image/png' },
          { src: 'icon-maskable-512.png', sizes: '512x512', type: 'image/png', purpose: 'maskable' },
        ],
      },
      workbox: {
        globPatterns: ['**/*.{js,css,html,wasm,svg,png,webmanifest,woff2}'],
        globIgnores: ['data/**'],
        navigateFallback: 'index.html',
        navigateFallbackDenylist: [/\/data\//],
        maximumFileSizeToCacheInBytes: 8 * 1024 * 1024,
        cleanupOutdatedCaches: true,
      },
    }),
  ],
  worker: { format: 'es' },
  
  build: { target: 'es2022', sourcemap: false },
  test: { environment: 'jsdom', include: ['src/**/*.test.ts'], setupFiles: ['src/test-setup.ts'] },
});
