/** Differences of the Android app build (Capacitor, `VITE_NATIVE=1`) from the website. Everything
 *  here is a no-op on the web, and the Capacitor plugins are only loaded in the app. */

export const IS_NATIVE = import.meta.env.VITE_NATIVE === '1';

/** Where dictionary data comes from: the website's /data/ in the app (its own files are bundled). */
export const DATA_BASE: string = IS_NATIVE
  ? (import.meta.env.VITE_DATA_BASE as string | undefined) ?? 'https://annoyingtrash123-design.github.io/Korean-Dictionary/'
  : import.meta.env.BASE_URL;

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
  if (!IS_NATIVE) return;
  const { App } = await import('@capacitor/app');
  await App.addListener('backButton', ({ canGoBack }) => { if (canGoBack) history.back(); else void App.exitApp(); });
}
