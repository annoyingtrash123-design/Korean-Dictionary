import { useEffect, useRef, useState } from 'preact/hooks';
import { settings, updateSettings, type ColorKey, type ThemeName } from '../lib/settings';
import { currentColor } from '../lib/theme';
import { useStore } from '../lib/store';
import { db, fetchManifest, packStatus$ } from '../db/client';
import { fmtBytes, update } from '../lib/app-state';
import { PackInstaller } from '../components/PackInstaller';
import { optionalPacks, packDesc, packEnabled, packLabel } from '../lib/packs';
import { cachedManifest } from '../db/client';
import { UpdateBanner } from '../components/UpdateBanner';
import { bookmarks, exportBookmarks, importBookmarks } from '../lib/bookmarks';
import { useAsync } from '../lib/useAsync';
import type { Manifest } from '../lib/types';

const THEMES: [ThemeName, string][] = [['light', 'Light'], ['dark', 'Dark'], ['sepia', 'Sepia'], ['system', 'System']];
const COLORS: [ColorKey, string][] = [['accent', 'Accent'], ['bg', 'Background'], ['text', 'Text'], ['hangul', 'Hangul'], ['hanja', 'Hanja']];

export function checkForUpdate(): Promise<Manifest | null> {
  return fetchManifest().then((m) => {
    const core = packStatus$.get()?.packs.core;
    update.set({ manifest: m, available: !!core?.installed && core.version !== m.version });
    return m;
  }, () => null);
}

export function SettingsView() {
  const s = useStore(settings);
  const st = useStore(packStatus$);
  const upd = useStore(update);
  const bm = useStore(bookmarks);
  const [msg, setMsg] = useState('');
  const [installing, setInstalling] = useState<string[] | null>(null);
  const fileRef = useRef<HTMLInputElement>(null);
  const est = useAsync(async () => (await navigator.storage?.estimate?.()) ?? {}, [st]);
  const about = useAsync(async () => {
    try { const r = await fetch(import.meta.env.BASE_URL + 'data/ATTRIBUTION.md'); if (r.ok && !(r.headers.get('content-type') ?? '').includes('html')) return await r.text(); } catch { /* offline */ }
    return null;
  }, []);
  const [theme, setTheme] = useState(s.theme);
  useEffect(() => setTheme(s.theme), [s.theme]);

  const core = st?.packs.core;
  const manifest = upd.manifest ?? cachedManifest();
  const installedIds = Object.keys(st?.packs ?? {}).filter((id) => id !== 'core' && st!.packs[id].installed);
  const optIds = [...new Set([...optionalPacks(manifest).map((p) => p.id), ...installedIds])];
  const allInstalled = ['core', ...installedIds];
  const setColor = (k: ColorKey, v: string) => updateSettings({ colors: { ...s.colors, [k]: v } });

  const doCheck = async () => { setMsg('Checking…'); const m = await checkForUpdate(); setMsg(m ? (update.get().available ? 'Update available.' : 'Dictionary is up to date.') : 'Could not reach the server (offline?).'); };
  const startInstall = async (ids: string[]) => { setMsg(''); const m = manifest ?? await checkForUpdate(); if (!m) { setMsg('You need to be online to download.'); return; } update.set((u) => ({ ...u, manifest: m })); setInstalling(ids); };

  const exportFile = () => {
    const url = URL.createObjectURL(new Blob([exportBookmarks()], { type: 'application/json' }));
    const a = Object.assign(document.createElement('a'), { href: url, download: `korean-dictionary-backup-${new Date().toISOString().slice(0, 10)}.json` });
    document.body.append(a); a.click(); a.remove(); setTimeout(() => URL.revokeObjectURL(url), 1000);
  };
  const importFile = async (f: File | undefined) => {
    if (!f) return;
    try { const r = await importBookmarks(await f.text()); setMsg(`Imported ${r.items} bookmark(s) and ${r.folders} folder(s).`); }
    catch { setMsg('That file is not a valid backup.'); }
    if (fileRef.current) fileRef.current.value = '';
  };

  return (
    <div class="page settings">
      <UpdateBanner />
      <section class="group">
        <h2>Appearance</h2>
        <div class="seg" role="radiogroup" aria-label="Theme">
          {THEMES.map(([v, l]) => <button key={v} type="button" role="radio" aria-checked={theme === v} class={theme === v ? 'on' : ''} onClick={() => updateSettings({ theme: v, colors: {} })}>{l}</button>)}
        </div>
        <div class="colors">
          {COLORS.map(([k, l]) => (
            <label key={k} class="color"><input type="color" aria-label={`${l} colour`} value={s.colors[k] ?? currentColor(k)} onInput={(e) => setColor(k, (e.currentTarget as HTMLInputElement).value)} /><span>{l}</span></label>
          ))}
        </div>
        {Object.keys(s.colors).length > 0 && <button type="button" class="link" onClick={() => updateSettings({ colors: {} })}>Reset colours to preset</button>}
        <label class="field-row"><span>Entry text size <span class="muted">{s.fontSize}px</span></span>
          <input type="range" min={14} max={26} step={1} value={s.fontSize} onInput={(e) => updateSettings({ fontSize: +(e.currentTarget as HTMLInputElement).value })} aria-label="Entry text size" /></label>
        <label class="field-row"><span>Show Korean definitions</span>
          <input type="checkbox" role="switch" checked={s.showKoDef} onChange={(e) => updateSettings({ showKoDef: (e.currentTarget as HTMLInputElement).checked })} /></label>
        <div class="preview ex"><div class="ex-ko hangul" lang="ko">저는 학교에 갑니다. <span class="hanja" lang="zh-Hant">學校</span></div><div class="ex-en">I go to school.</div></div>
      </section>

      <section class="group">
        <h2>Dictionaries</h2>
        <div class="pack-line"><div><strong>{packLabel('core')}</strong><div class="muted small">{core ? `v${core.version} · ${fmtBytes(core.bytes)}` : 'Not installed'}</div></div></div>
        {optIds.map((id) => {
          const inst = st?.packs[id]; const mp = manifest?.packs.find((p) => p.id === id);
          return (
            <div key={id} class="pack-opt">
              <div class="pack-line">
                <div><strong>{packLabel(id)}</strong><div class="muted small">{packDesc(id)}</div>
                  <div class="muted small">{inst?.installed ? `v${inst.version} · ${fmtBytes(inst.bytes)} on device` : mp ? `Not downloaded · ${fmtBytes(mp.gz_bytes)} download` : 'Not downloaded'}</div></div>
                {inst?.installed
                  ? <input type="checkbox" role="switch" aria-label={`Use ${packLabel(id)} in search`} checked={packEnabled(s, id)} onChange={(e) => updateSettings({ packs: { ...s.packs, [id]: (e.currentTarget as HTMLInputElement).checked } })} />
                  : <button type="button" class="btn" onClick={() => startInstall([id])}>Download</button>}
              </div>
              {inst?.installed && <button type="button" class="link danger" onClick={async () => { if (confirm(`Delete ${packLabel(id)} from this device?`)) { await db.removePack(id); updateSettings({ packs: { ...s.packs, [id]: false } }); } }}>Delete data</button>}
            </div>
          );
        })}
        {installing && manifest && <PackInstaller manifest={manifest} packIds={installing} label={upd.available ? 'Update' : 'Download'} onDone={() => { const ids = installing; setInstalling(null); update.set((u) => ({ ...u, available: false })); updateSettings({ packs: { ...s.packs, ...Object.fromEntries(ids.filter((i) => i !== 'core' && !installedIds.includes(i)).map((i) => [i, true])) } }); checkForUpdate(); }} />}
      </section>

      <section class="group">
        <h2>Data</h2>
        <dl class="facts wide">
          <dt>Installed version</dt><dd>{core?.version ?? '–'}</dd>
          <dt>Latest online</dt><dd>{manifest?.version ?? '–'}</dd>
          <dt>Storage used</dt><dd>{fmtBytes(est.data?.usage)}{est.data?.quota ? ` of ${fmtBytes(est.data.quota)}` : ''}</dd>
        </dl>
        <div class="btn-row">
          <button type="button" class="btn" onClick={doCheck}>Check for update</button>
          {upd.available && <button type="button" class="btn primary" onClick={() => startInstall(allInstalled)}>Update now</button>}
          <button type="button" class="btn" onClick={() => startInstall(allInstalled)}>Re-download</button>
        </div>
        {msg && <p class="muted small" role="status">{msg}</p>}
      </section>

      <section class="group">
        <h2>Backup</h2>
        <p class="muted small">{bm.items.length} saved word(s). Exports bookmarks and folders as a JSON file.</p>
        <div class="btn-row">
          <button type="button" class="btn" onClick={exportFile}>Export bookmarks</button>
          <button type="button" class="btn" onClick={() => fileRef.current?.click()}>Import…</button>
          <input ref={fileRef} type="file" accept="application/json,.json" hidden aria-label="Import backup file" onChange={(e) => importFile((e.currentTarget as HTMLInputElement).files?.[0])} />
        </div>
      </section>

      <section class="group">
        <h2>About &amp; licences</h2>
        {about.data ? <pre class="licence">{about.data}</pre> : <Licences />}
      </section>
    </div>
  );
}

function Licences() {
  const rows = [
    ['krdict 한국어기초사전', 'National Institute of Korean Language', 'CC BY-SA 2.0 KR'],
    ['표준국어대사전 (stdict)', 'National Institute of Korean Language', 'CC BY-SA 2.0 KR'],
    ['Wiktionary (via kaikki.org)', 'Wiktionary contributors', 'CC BY-SA 4.0'],
    ['kengdic', 'Garfield Nate et al.', 'MPL 2.0 / LGPL'],
    ['Tatoeba', 'Tatoeba contributors', 'CC BY 2.0 FR'],
    ['Unihan', 'Unicode, Inc.', 'Unicode licence'],
    ['FrequencyWords', 'hermitdave (OpenSubtitles)', 'CC BY-SA 4.0'],
  ];
  return (
    <ul class="plain licences">
      {rows.map(([n, a, l]) => <li key={n}><strong>{n}</strong><div class="muted small">{a} · {l}</div></li>)}
    </ul>
  );
}
