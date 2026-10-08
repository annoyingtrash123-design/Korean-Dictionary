export type Source = 'krdict' | 'wikt' | 'kengdic' | 'stdict' | 'opendict' | 'cedict' | 'zhwikt';
export const SOURCE_ORDER: Source[] = ['krdict', 'wikt', 'kengdic', 'stdict', 'opendict', 'cedict', 'zhwikt'];
export const SOURCE_TITLE: Record<Source, string> = {
  krdict: 'krdict 한국어기초사전',
  wikt: 'Wiktionary',
  kengdic: 'kengdic',
  stdict: '표준국어대사전 (Korean)',
  opendict: '우리말샘 (Korean)',
  cedict: 'CC-CEDICT',
  zhwikt: 'Wiktionary (Chinese)',
};
export const SOURCE_SHORT: Record<Source, string> = { krdict: 'krdict', wikt: 'wikt', kengdic: 'kengdic', stdict: 'stdict', opendict: '우리말샘', cedict: 'CC-CEDICT', zhwikt: 'zh-wikt' };
/** Korean-only sources (lang 'ko'): never preferred over an English source. */
export const isKoSource = (s: string) => s === 'stdict' || s === 'opendict';

export interface Example { ko: string; en?: string; type?: 'phrase' | 'sentence' | 'dialogue' }
export interface Rel { type: string; word: string }
export interface Sense {
  pos?: string; gloss?: string; roman?: string; def?: string; ko_def?: string; note?: string; pattern?: string;
  tags?: string[]; examples?: Example[]; rel?: Rel[];
}
export interface EntryData {
  senses: Sense[]; related?: Rel[]; category?: string; etym?: string; origin_note?: string;
}
export interface EntryRow {
  id: number; source: Source; headword: string; hw_norm: string; homonym?: number;
  hanja?: string; pos?: string; pron?: string; lang: 'en' | 'ko';
  level?: number; rank: number; kind: 'word' | 'phrase' | 'idiom' | 'proverb' | 'grammar';
  gloss?: string;
}
export interface Entry extends EntryRow { data: EntryData }

export type MatchKind = 'exact' | 'form' | 'deconj' | 'prefix' | 'fts' | 'hanja';
export interface ResultRow {
  source: Source; id: number; headword: string; hanja?: string; pos?: string; level?: number;
  gloss?: string; kind: EntryRow['kind']; pack: string; via?: MatchKind;
  rank: number; homonym?: number; pron?: string; lang: 'en' | 'ko'; hw_norm: string;
}
export interface HanjaChar { ch: string; readings?: string; meaning_en?: string; strokes?: number; radical?: string; word_count?: number; hun?: string; eumhun?: string }
export interface GrammarRow { id: number; entry_id?: number; pattern: string; category: string; level?: number; summary_en?: string; sort?: number }
export interface Sentence { ko: string; en: string | null; source: string | null; id?: number }

export type SearchMode = 'hangul' | 'latin' | 'han' | 'empty';   // UI-side script detection
export interface SearchResult {
  mode: 'hangul' | 'english' | 'hanja'; rows: ResultRow[]; hanja?: HanjaChar[];
  deconj?: { lemma: string; rule: string }[]; grammarHints?: string[];
}

// ---- Reader (see docs/READER.md "texts pack + engine API") ----
export interface TextLabels { notes?: 'ai' | null; translation?: string | null; modern?: 'ai' | 'wikisource' | null; text?: 'original' | 'ai' | null }
/** A catalogue text not in the Reader library yet (the "To add" tab). */
export interface PendingText {
  id: string; title_ko: string; title_en: string; author_ko: string; author_en: string;
  shelf: string; period: string; year: number; date: string;
  /** source = no usable source yet · repair = source text needs fixing · held = copyright/source check · preparing = notes/translation in preparation */
  status: 'source' | 'repair' | 'held' | 'preparing';
}

/** AI-written learner essay attached to a primary source as context (`notes.background`). */
export interface TextBackground {
  id: string; title_ko?: string | null; title_en?: string | null; level?: string | null;
  paragraphs: { ko: string; en?: string | null }[];
  vocab?: { word: string; gloss_en: string; level?: number | null }[] | null;
  questions?: { q_ko: string; q_en?: string; answer_en?: string }[] | null;
  reviewed?: boolean;
}
export interface TextSummary {
  id: string; shelf: string; period?: string | null; year?: number | null; script: 'hangul' | 'hanmun' | 'mixed' | string;
  level?: string | null; chars: number; title_ko: string; title_en?: string | null; author_ko?: string | null; author_en?: string | null;
  date?: string | null; themes: string[]; excerpt?: boolean; labels: TextLabels; summary_ko?: string; summary_en?: string;
}
export interface TextParagraph { n: number; orig: string; modern?: string | null; reading?: string | null; en?: string | null }
export interface TextProvenance { source?: string; url?: string; page_title?: string; revision_id?: string | number; revision_timestamp?: string; edition?: string; licence?: string; fetched_at?: string; english_source?: string }
export interface TextDoc {
  id: string;
  meta: { title_ko: string; title_en?: string; author_ko?: string; author_en?: string; author_dates?: string; date?: string; year?: number; period?: string; themes?: string[]; shelf?: string; script?: string; excerpt?: boolean; excerpt_note?: string; pd_basis?: string; [k: string]: unknown };
  card: { summary_ko?: string; summary_en?: string; level?: string; edition_ko?: string; edition_en?: string };
  notes: { ko?: string; en?: string; background?: TextBackground | null };
  provenance?: TextProvenance | null;
  labels: TextLabels;
  review?: { status?: string; [k: string]: unknown } | null;
  vocab?: { word: string; gloss_en: string; level?: number | null }[] | null;
  questions?: { q_ko: string; q_en?: string; answer_en?: string }[] | null;
  paragraphs: TextParagraph[];
}
/** `lookupInText` result: offsets are UTF-16 code units of the text passed in, `end` exclusive. */
export interface TextMatch { match: string; start: number; end: number; rows: ResultRow[]; hanja?: HanjaChar; deconj?: { lemma: string; rule: string }[] }

export interface Engine {
  init(): Promise<void>;
  installedPacks(): Promise<{ id: string; version: string; bytes: number }[]>;
  beginImport(packId: string, totalBytes: number): Promise<void>;
  writeChunk(packId: string, bytes: Uint8Array): Promise<void>;
  /** `sha256`: hex digest of the uncompressed DB; the engine verifies it (throws on mismatch, old pack stays intact). */
  finishImport(packId: string, version: string, sha256?: string | null): Promise<void>;
  deletePack(packId: string): Promise<void>;
  search(query: string, opts: { packs: string[]; limit?: number }): Promise<SearchResult>;
  entriesByHeadword(hw: string, packs: string[]): Promise<Entry[]>;
  entry(source: string, id: number): Promise<Entry | null>;
  hanjaChar(ch: string): Promise<HanjaChar | null>;
  wordsWithHanja(ch: string, limit: number, offset: number): Promise<ResultRow[]>;
  sentences(text: string, limit: number): Promise<{ ko: string; en: string | null; source: string | null }[]>;
  grammarList(): Promise<GrammarRow[]>;
  wordOfDay(date: string): Promise<ResultRow | null>;
  listTexts(): Promise<TextSummary[]>;
  pendingTexts(): Promise<PendingText[]>;
  getText(id: string): Promise<TextDoc | null>;
  lookupInText(text: string, offset: number, opts: { packs: string[]; limit?: number }): Promise<TextMatch>;
  /** Background warm-up step over `packs`; resolves to whether more steps remain. */
  warm(step: number, packs: string[]): Promise<boolean>;
  /** Engine diagnostics log (startup file discovery, pack swaps). */
  diagnostics(): Promise<string>;
}

export interface PackInfo { id: string; installed: boolean; version?: string; bytes?: number; installedAt?: number }
export interface PackStatus { ready: boolean; packs: Record<string, PackInfo>; error?: string }

export interface ManifestChunk { file: string; bytes?: number }
export interface ManifestPack {
  id: string; required: boolean; file: string; bytes: number; gz_bytes: number; sha256?: string;
  chunks: (string | { file?: string; name?: string; url?: string; bytes?: number; gz_bytes?: number })[];
  counts?: Record<string, number>;
}
export interface Manifest { version: string; packs: ManifestPack[] }

export interface Progress { pack: string; phase: 'download' | 'install' | 'done'; done: number; total: number; chunk?: number; chunks?: number }
