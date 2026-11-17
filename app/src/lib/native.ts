/** Android-app plumbing: where dictionary data comes from, and the Capacitor plugins (loaded on demand).
 *  In a desktop browser (the e2e tests) the plugins fall back to their web implementations. */

/** Where dictionary data comes from: the data host (GitHub Pages). The app's own files are bundled, and only
 *  the e2e build points this at the local server (`VITE_DATA_BASE=/`). */
export const DATA_BASE: string =
  (import.meta.env.VITE_DATA_BASE as string | undefined) ?? 'https://annoyingtrash123-design.github.io/Korean-Dictionary/';

/** Read Korean aloud with Android's text-to-speech (the WebView has no speechSynthesis). */
export async function nativeSpeak(text: string): Promise<void> {
  const { TextToSpeech } = await import('@capacitor-community/text-to-speech');
  await TextToSpeech.stop().catch(() => undefined); // nothing playing is fine
  await TextToSpeech.speak({ text, lang: 'ko-KR', rate: 0.9 });
}

/** Save a backup file and offer it through the share sheet (downloads don't work in a WebView). */
export async function nativeSaveFile(name: string, text: string): Promise<void> {
  const { Filesystem, Directory, Encoding } = await import('@capacitor/filesystem');
  const { Share } = await import('@capacitor/share');
  const r = await Filesystem.writeFile({ path: name, data: text, directory: Directory.Cache, encoding: Encoding.UTF8 });
  await Share.share({ title: name, files: [r.uri] });
}

/** Android back button: go back in the app; on the first screen, leave the app. */
export async function installBackButton(): Promise<void> {
  const { App } = await import('@capacitor/app');
  await App.addListener('backButton', ({ canGoBack }) => { if (canGoBack) history.back(); else void App.exitApp(); });
}
