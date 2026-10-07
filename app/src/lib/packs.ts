import type { Manifest, ManifestPack, PackStatus } from '../db/types';
import type { Settings } from './settings';

export const PACK_INFO: Record<string, { label: string; desc: string }> = {
  core: { label: 'Core dictionary', desc: 'English definitions, hanja, example sentences' },
  stdict: { label: '표준국어대사전 (Korean–Korean)', desc: 'Standard dictionary · ~436k words, Korean definitions, hanja' },
  texts: { label: 'Reader library', desc: 'Public-domain and openly licensed Korean texts for the 읽기 Reader · tap-to-look-up, parallel English' },
  cedict: { label: 'CC-CEDICT (Chinese–English)', desc: 'For hanja and hanmun lookups in the Reader · CC BY-SA 4.0' },
  zhwikt: { label: 'Wiktionary Chinese', desc: 'Classical senses, Middle Chinese and Sino-Korean readings for hanmun · large' },
  opendict: { label: '우리말샘 (Korean–Korean)', desc: 'Open dictionary · ~1M extra words incl. dialect, archaic, North Korean, technical terms' },
};
/** Packs that are content, not dictionaries: never passed to search/lookup. */
export const CONTENT_PACKS = ['texts'];
export const packLabel = (id: string) => PACK_INFO[id]?.label ?? id;
export const packDesc = (id: string) => PACK_INFO[id]?.desc ?? 'Optional dictionary';

/** Optional packs offered by a manifest (everything not `required`). */
export const optionalPacks = (m: Manifest | null | undefined): ManifestPack[] => (m?.packs ?? []).filter((p) => !p.required && p.id !== 'core');
/** Is an optional pack switched on in settings (defaults to off for unknown packs)? */
export const packEnabled = (s: Pick<Settings, 'packs'>, id: string) => !!s.packs[id];
/** Pack ids to search: core plus enabled, installed optional packs. */
export function activePacks(s: Pick<Settings, 'packs'>, st: PackStatus | null, hint: string[] = []): string[] {
  if (!st) return hint.filter((id) => !CONTENT_PACKS.includes(id) && (id === 'core' || packEnabled(s, id)));
  const ids = Object.keys(st?.packs ?? {}).filter((id) => !CONTENT_PACKS.includes(id) && (id === 'core' || (st!.packs[id].installed && packEnabled(s, id))));
  return ids.sort((a, b) => (a === 'core' ? -1 : b === 'core' ? 1 : a.localeCompare(b)));
}
export const packsKey = (s: Pick<Settings, 'packs'>) => JSON.stringify(s.packs);
