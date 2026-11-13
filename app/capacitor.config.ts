import type { CapacitorConfig } from '@capacitor/cli';

// Android app: the same web app, run in the app's own WebView. Its storage (dictionaries, bookmarks)
// lives in the app's private folder, so clearing Chrome cannot touch it.
// Build: BASE_PATH=/ VITE_NATIVE=1 npm run build && npx cap sync android  (CI: .github/workflows/android.yml)
const config: CapacitorConfig = {
  appId: 'app.hanppuri.dictionary',
  appName: '한뿌리',
  webDir: 'dist',
  android: { backgroundColor: '#294d38' },
  server: { androidScheme: 'https' },
};

export default config;
