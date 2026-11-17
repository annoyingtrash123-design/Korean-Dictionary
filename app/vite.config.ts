import { defineConfig } from 'vitest/config';
import preact from '@preact/preset-vite';

export default defineConfig({
  base: '/',   // the Android WebView serves the bundle from https://localhost/
  define: {
    __BUILD__: JSON.stringify(new Date().toISOString().slice(0, 16)),   // shown in storage diagnostics
    __APP_VERSION__: JSON.stringify(process.env.ANDROID_VERSION_NAME ?? 'dev'),   // Android app's versionName
  },
  plugins: [
    preact(),
  ],
  worker: { format: 'es' },
  
  build: { target: 'es2022', sourcemap: false },
  test: { environment: 'jsdom', include: ['src/**/*.test.ts'], setupFiles: ['src/test-setup.ts'] },
});
