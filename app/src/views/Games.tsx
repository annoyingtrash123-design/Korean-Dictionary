import { useEffect, useMemo, useRef, useState } from 'preact/hooks';
import { db } from '../db/client';
import { Icon } from '../components/Icons';
import { href, wordPath } from '../lib/router';
import type { GameWord, TextSummary } from '../lib/types';
import { hanjaKey } from '../lib/hanja-marks';
import { chainFollows, choseong, dueumStarts, firstSyllable, lastSyllable, sameKorean, shortGloss, syllables } from '../lib/games/hangul';
import { choQuestion, gapQuestion, meaningQuestion, shuffle, type Choice, type Dir } from '../lib/games/quiz';
import { LEVELS, loadPool, sourceLabel, type Source } from '../lib/games/sources';
import { dueWords, loadDays, loadDeck, pickRound, saveRound, streak } from '../lib/games/srs';

// ---- preferences (per device; a convenience, so failures are ignored) -----------------------

type Length = 'n10' | 'n20' | 't60';
interface Prefs { source: Source; length: Length }
const PREFS_KEY = 'kd.games';
const DEFAULT_PREFS: Prefs = { source: { kind: 'level', level: 1 }, length: 'n10' };
function loadPrefs(): Prefs {
  try { return { ...DEFAULT_PREFS, ...JSON.parse(localStorage.getItem(PREFS_KEY) || '{}') }; } catch { return DEFAULT_PREFS; }
}
const savePrefs = (p: Prefs) => { try { localStorage.setItem(PREFS_KEY, JSON.stringify(p)); } catch { /* */ } };

type GameId = 'meaning' | 'type' | 'match' | 'gap' | 'cho' | 'chain';
const GAMES: { id: GameId; ko: string; en: string; desc: string; usesWords: boolean }[] = [
  { id: 'meaning', ko: '뜻 맞히기', en: 'Meaning quiz', desc: 'Pick the right meaning, Korean ↔ English.', usesWords: true },
  { id: 'type', ko: '단어 쓰기', en: 'Type the word', desc: 'See the English, type the Korean.', usesWords: true },
  { id: 'match', ko: '짝 맞추기', en: 'Match pairs', desc: 'Pair Korean and English tiles.', usesWords: true },
  { id: 'gap', ko: '빈칸 채우기', en: 'Fill the gap', desc: 'Choose the missing word in a real sentence.', usesWords: true },
  { id: 'cho', ko: '초성 퀴즈', en: 'Initials quiz', desc: 'Guess the word from its initial consonants.', usesWords: true },
  { id: 'chain', ko: '끝말잇기', en: 'Word chain', desc: 'Take turns: start with the last syllable. Nouns only.', usesWords: false },
];

interface Answer { word: GameWord; correct: boolean }
const ROUND_N: Record<Length, number> = { n10: 10, n20: 20, t60: 60 };
const TIMED = (l: Length) => l === 't60';

// ---- shell ------------------------------------------------------------------------------------

export function GamesView() {
  const [prefs, setPrefsState] = useState(loadPrefs);
  const setPrefs = (p: Prefs) => { setPrefsState(p); savePrefs(p); };
  const [playing, setPlaying] = useState<{ game: GameId; pool: GameWord[]; words: GameWord[] } | null>(null);
  const [result, setResult] = useState<{ game: GameId; answers: Answer[]; score?: string } | null>(null);
  const [busy, setBusy] = useState<GameId | null>(null);
  const [err, setErr] = useState('');

  const start = async (game: GameId) => {
    setErr('');
    if (game === 'chain') { setPlaying({ game, pool: [], words: [] }); return; }
    setBusy(game);
    try {
      const pool = await loadPool(prefs.source);
      const [deck] = await Promise.all([loadDeck()]);
      let usable = pool;
      if (game === 'gap') usable = pool.filter((w) => gapQuestion(w, pool));
      if (game === 'cho') usable = pool.filter((w) => choQuestion(w));
      if (usable.length < 4) { setErr(prefs.source.kind === 'recent' ? 'Look up a few more words first: this game needs at least 4 with English meanings.' : 'Not enough words for this game from this source. Try another source.'); return; }
      const n = TIMED(prefs.length) ? usable.length : ROUND_N[prefs.length];
      const words = pickRound(usable, deck, Math.min(n, usable.length), Date.now());
      setPlaying({ game, pool, words });
    } catch (e) {
      setErr(String((e as Error)?.message ?? e));
    } finally { setBusy(null); }
  };
  const finish = (game: GameId, answers: Answer[], score?: string) => {
    if (game !== 'chain' && answers.length) void saveRound(answers);
    setPlaying(null);
    setResult({ game, answers, score });
  };

  if (playing) {
    const props = { words: playing.words, pool: playing.pool, length: prefs.length, onQuit: () => setPlaying(null), onDone: (a: Answer[], score?: string) => finish(playing.game, a, score) };
    const g = GAMES.find((x) => x.id === playing.game)!;
    return (
      <div class="gm">
        <GameHead title={`${g.ko} · ${g.en}`} onQuit={props.onQuit} />
        {playing.game === 'meaning' && <MeaningGame {...props} />}
        {playing.game === 'type' && <TypeGame {...props} />}
        {playing.game === 'match' && <MatchGame {...props} />}
        {playing.game === 'gap' && <GapGame {...props} />}
        {playing.game === 'cho' && <ChoGame {...props} />}
        {playing.game === 'chain' && <ChainGame onQuit={props.onQuit} onDone={props.onDone} />}
      </div>
    );
  }
  if (result) return <Results result={result} onAgain={() => { setResult(null); void start(result.game); }} onBack={() => setResult(null)} />;
  return <GamesHome prefs={prefs} setPrefs={setPrefs} onStart={start} busy={busy} err={err} />;
}

function GameHead({ title, onQuit }: { title: string; onQuit: () => void }) {
  return (
    <header class="gm-head gm-head-play">
      <button type="button" class="iconbtn" aria-label="Quit game" onClick={onQuit}><Icon name="x" /></button>
      <h1 class="gm-play-title">{title}</h1>
      <span class="gm-head-gap" />
    </header>
  );
}

// ---- home ---------------------------------------------------------------------------------

function GamesHome({ prefs, setPrefs, onStart, busy, err }: { prefs: Prefs; setPrefs: (p: Prefs) => void; onStart: (g: GameId) => void; busy: GameId | null; err: string }) {
  const [texts, setTexts] = useState<TextSummary[]>([]);
  const [stats, setStats] = useState<{ streak: number; due: number }>({ streak: 0, due: 0 });
  useEffect(() => {
    db.listTexts().then((t) => setTexts(t.filter((x) => x.script !== 'hanmun')), () => setTexts([]));
    Promise.all([loadDeck(), loadDays()]).then(([deck, days]) => setStats({ streak: streak(days, Date.now()), due: dueWords(deck, Date.now()).length }));
  }, []);
  const src = prefs.source;
  const setSource = (s: Source) => setPrefs({ ...prefs, source: s });
  return (
    <div class="gm">
      <header class="gm-head">
        <div class="gm-head-row">
          <h1 class="gm-title"><span lang="ko">게임</span> <span class="gm-h-en">Games</span></h1>
          <span class="gm-stats" aria-label="Progress">
            {stats.streak > 0 && <span title="Days in a row">🔥 {stats.streak}</span>}
            {stats.due > 0 && <span title="Words due for review">{stats.due} to review</span>}
          </span>
        </div>
      </header>

      <section class="gm-section" aria-label="Words">
        <h2 class="gm-label">Words from</h2>
        <div class="seg inline gm-seg" role="radiogroup" aria-label="Word source">
          {LEVELS.map((l) => (
            <button key={l.level} type="button" role="radio" aria-checked={src.kind === 'level' && src.level === l.level} class={src.kind === 'level' && src.level === l.level ? 'on' : ''}
              onClick={() => setSource({ kind: 'level', level: l.level })}><span lang="ko">{l.ko}</span></button>
          ))}
          <button type="button" role="radio" aria-checked={src.kind === 'recent'} class={src.kind === 'recent' ? 'on' : ''} onClick={() => setSource({ kind: 'recent' })}>Recent</button>
        </div>
        {texts.length > 0 && (
          <label class="gm-text-pick">
            <span class="muted small">or a Reader text</span>
            <select class="field" value={src.kind === 'text' ? src.id : ''} onChange={(e) => {
              const id = (e.currentTarget as HTMLSelectElement).value;
              const t = texts.find((x) => x.id === id);
              if (t) setSource({ kind: 'text', id, title: t.title_ko });
            }}>
              <option value="" disabled>Choose a text…</option>
              {texts.map((t) => <option key={t.id} value={t.id}>{t.title_ko}{t.title_en ? ` · ${t.title_en}` : ''}</option>)}
            </select>
          </label>
        )}
        <p class="muted small gm-src-now">Playing with: {sourceLabel(src)}</p>

        <h2 class="gm-label">Round</h2>
        <div class="seg inline gm-seg" role="radiogroup" aria-label="Round length">
          {([['n10', '10 words'], ['n20', '20 words'], ['t60', '60 seconds']] as const).map(([k, l]) => (
            <button key={k} type="button" role="radio" aria-checked={prefs.length === k} class={prefs.length === k ? 'on' : ''} onClick={() => setPrefs({ ...prefs, length: k })}>{l}</button>
          ))}
        </div>
      </section>

      {err && <p class="error gm-err" role="alert">{err}</p>}
      <div class="gm-cards">
        {GAMES.map((g) => (
          <button key={g.id} type="button" class="gm-card" disabled={!!busy} onClick={() => onStart(g.id)}>
            <span class="gm-card-ko" lang="ko">{g.ko}</span>
            <span class="gm-card-en">{g.en}</span>
            <span class="gm-card-desc muted small">{busy === g.id ? 'Loading words…' : g.desc}</span>
          </button>
        ))}
      </div>
      <p class="muted small gm-foot">Words you miss come back sooner; words you know come back later. Progress stays on this device.</p>
    </div>
  );
}

// ---- shared round state -------------------------------------------------------------------

interface GameProps { words: GameWord[]; pool: GameWord[]; length: Length; onQuit: () => void; onDone: (a: Answer[], score?: string) => void }

/** Question index, answers and (for timed rounds) the clock. */
function useRound(total: number, length: Length, onDone: (a: Answer[]) => void) {
  const [i, setI] = useState(0);
  const answers = useRef<Answer[]>([]);
  const [left, setLeft] = useState(60);
  const done = useRef(false);
  const end = () => { if (!done.current) { done.current = true; onDone(answers.current); } };
  useEffect(() => {
    if (!TIMED(length)) return;
    const t0 = Date.now();
    const id = setInterval(() => { const l = 60 - Math.floor((Date.now() - t0) / 1000); setLeft(Math.max(0, l)); if (l <= 0) { clearInterval(id); end(); } }, 250);
    return () => clearInterval(id);
  }, []);
  const record = (word: GameWord, correct: boolean) => { answers.current = [...answers.current, { word, correct }]; };
  const next = () => { if (i + 1 >= total) end(); else setI(i + 1); };
  const status = TIMED(length) ? `${left}s · ${answers.current.filter((a) => a.correct).length} right` : `${Math.min(i + 1, total)} / ${total}`;
  return { i, record, next, end, status, answers };
}

function Progress({ text }: { text: string }) {
  return <div class="gm-progress" aria-live="off">{text}</div>;
}

/** The answer line shown after a question: word, hanja, meaning. */
function Reveal({ word, correct }: { word: GameWord; correct: boolean }) {
  return (
    <div class={`gm-reveal ${correct ? 'ok' : 'no'}`} role="status">
      <strong>{correct ? 'Correct' : 'Not quite'}</strong>
      <span><span class="hangul" lang="ko">{word.headword.replace(/\d+$/, '')}</span>{word.hanja ? <> <span class="hanja" lang="zh-Hant">{hanjaKey(word.hanja)}</span></> : null} · {word.gloss}</span>
    </div>
  );
}

function Choices({ choices, picked, onPick, ko }: { choices: Choice[]; picked: number | null; onPick: (i: number) => void; ko: boolean }) {
  return (
    <div class="gm-choices">
      {choices.map((c, i) => {
        const state = picked == null ? '' : c.correct ? ' right' : i === picked ? ' wrong' : ' dim';
        return <button key={i} type="button" class={`gm-choice${state}${ko ? ' ko' : ''}`} lang={ko ? 'ko' : undefined} disabled={picked != null} onClick={() => onPick(i)}>{c.label}</button>;
      })}
    </div>
  );
}

// ---- 1. meaning quiz ------------------------------------------------------------------------

function MeaningGame({ words, pool, length, onDone }: GameProps) {
  const r = useRound(words.length, length, onDone);
  const word = words[r.i % words.length];
  const q = useMemo(() => meaningQuestion(word, pool, (Math.random() < 0.5 ? 'ko-en' : 'en-ko') as Dir), [r.i]);
  const [picked, setPicked] = useState<number | null>(null);
  useEffect(() => setPicked(null), [r.i]);
  const pick = (i: number) => {
    setPicked(i);
    const ok = q.choices[i].correct;
    r.record(word, ok);
    if (ok) setTimeout(r.next, 700);
  };
  return (
    <div class="gm-play">
      <Progress text={r.status} />
      <p class="gm-ask muted small">{q.dir === 'ko-en' ? 'What does this mean?' : 'Which Korean word means…'}</p>
      <div class={`gm-prompt${q.dir === 'ko-en' ? ' ko' : ''}`} lang={q.dir === 'ko-en' ? 'ko' : undefined}>{q.prompt}</div>
      <Choices choices={q.choices} picked={picked} onPick={pick} ko={q.dir === 'en-ko'} />
      {picked != null && !q.choices[picked].correct && <><Reveal word={word} correct={false} /><button type="button" class="btn primary gm-next" onClick={r.next}>Next</button></>}
    </div>
  );
}

// ---- 2. type the word -----------------------------------------------------------------------

function TypeGame({ words, length, onDone }: GameProps) {
  const r = useRound(words.length, length, onDone);
  const word = words[r.i % words.length];
  const [typed, setTyped] = useState('');
  const [state, setState] = useState<null | boolean>(null);
  const [hint, setHint] = useState(false);
  const input = useRef<HTMLInputElement>(null);
  useEffect(() => { setTyped(''); setState(null); setHint(false); input.current?.focus(); }, [r.i]);
  const check = (giveUp = false) => {
    if (state != null) { r.next(); return; }
    const ok = !giveUp && sameKorean(typed, word.headword);
    setState(ok); r.record(word, ok);
    if (ok) setTimeout(r.next, 700);
  };
  return (
    <div class="gm-play">
      <Progress text={r.status} />
      <p class="gm-ask muted small">Type the Korean word for</p>
      <div class="gm-prompt">{word.gloss}</div>
      <p class="muted small gm-sub">{word.pos}{hint ? <> · starts with <span lang="ko">{firstSyllable(word.headword)}</span>{word.hanja ? <> · <span class="hanja" lang="zh-Hant">{hanjaKey(word.hanja)}</span></> : null}</> : null}</p>
      <form class="gm-type" onSubmit={(e) => { e.preventDefault(); check(); }}>
        <input ref={input} class="field" lang="ko" autocomplete="off" autocapitalize="off" spellcheck={false} aria-label="Your answer" value={typed} disabled={state != null}
          onInput={(e) => setTyped((e.currentTarget as HTMLInputElement).value)} />
        <button type="submit" class="btn primary">{state == null ? 'Check' : 'Next'}</button>
      </form>
      {state == null && (
        <div class="btn-row gm-row">
          {!hint && <button type="button" class="btn" onClick={() => setHint(true)}>Hint</button>}
          <button type="button" class="btn" onClick={() => check(true)}>I don't know</button>
        </div>
      )}
      {state != null && <Reveal word={word} correct={state} />}
    </div>
  );
}

// ---- 3. match pairs -------------------------------------------------------------------------

const BOARD = 5;
function MatchGame({ words, length, onDone }: GameProps) {
  const boards = Math.max(1, Math.floor(words.length / BOARD));
  const [b, setB] = useState(0);
  const answers = useRef<Answer[]>([]);
  const [left, setLeft] = useState(60);
  const done = useRef(false);
  const end = () => { if (!done.current) { done.current = true; onDone(answers.current); } };
  useEffect(() => {
    if (!TIMED(length)) return;
    const t0 = Date.now();
    const id = setInterval(() => { const l = 60 - Math.floor((Date.now() - t0) / 1000); setLeft(Math.max(0, l)); if (l <= 0) { clearInterval(id); end(); } }, 250);
    return () => clearInterval(id);
  }, []);
  const set = useMemo(() => words.slice((b % boards) * BOARD, (b % boards) * BOARD + BOARD), [b]);
  const ko = useMemo(() => shuffle(set.map((w, i) => ({ i, label: w.headword.replace(/\d+$/, '') }))), [set]);
  const en = useMemo(() => shuffle(set.map((w, i) => ({ i, label: shortGloss(w.gloss) }))), [set]);
  const [sel, setSel] = useState<{ side: 'ko' | 'en'; i: number } | null>(null);
  const [matched, setMatched] = useState<Set<number>>(new Set());
  const [missed, setMissed] = useState<Set<number>>(new Set());
  const [flash, setFlash] = useState<string | null>(null);
  useEffect(() => { setMatched(new Set()); setMissed(new Set()); setSel(null); }, [b]);
  const tap = (side: 'ko' | 'en', i: number) => {
    if (matched.has(i)) return;
    if (!sel || sel.side === side) { setSel({ side, i }); return; }
    if (sel.i === i) {
      const m = new Set(matched); m.add(i); setMatched(m); setSel(null);
      answers.current = [...answers.current, { word: set[i], correct: !missed.has(i) }];
      if (m.size === set.length) setTimeout(() => { if (!TIMED(length) && b + 1 >= boards) end(); else setB(b + 1); }, 450);
    } else {
      setMissed(new Set([...missed, sel.i, i]));
      setFlash(`${side}:${i}`); setTimeout(() => setFlash(null), 450); setSel(null);
    }
  };
  const status = TIMED(length) ? `${left}s · ${answers.current.filter((a) => a.correct).length} right` : `Board ${b + 1} / ${boards}`;
  const tile = (side: 'ko' | 'en', t: { i: number; label: string }) => {
    const on = sel?.side === side && sel.i === t.i;
    const cls = `gm-tile${side === 'ko' ? ' ko' : ''}${on ? ' on' : ''}${matched.has(t.i) ? ' done' : ''}${flash === `${side}:${t.i}` ? ' wrong' : ''}`;
    return <button key={`${side}${t.i}`} type="button" class={cls} lang={side === 'ko' ? 'ko' : undefined} disabled={matched.has(t.i)} onClick={() => tap(side, t.i)}>{t.label}</button>;
  };
  return (
    <div class="gm-play">
      <Progress text={status} />
      <p class="gm-ask muted small">Tap a Korean word, then its meaning.</p>
      <div class="gm-board"><div class="gm-col">{ko.map((t) => tile('ko', t))}</div><div class="gm-col">{en.map((t) => tile('en', t))}</div></div>
    </div>
  );
}

// ---- 4. fill the gap ------------------------------------------------------------------------

function GapGame({ words, pool, length, onDone }: GameProps) {
  const r = useRound(words.length, length, onDone);
  const word = words[r.i % words.length];
  const q = useMemo(() => gapQuestion(word, pool), [r.i]);
  const [picked, setPicked] = useState<number | null>(null);
  useEffect(() => setPicked(null), [r.i]);
  useEffect(() => { if (!q) r.next(); }, [q]);
  if (!q) return null;
  const pick = (i: number) => { setPicked(i); const ok = q.choices[i].correct; r.record(word, ok); if (ok) setTimeout(r.next, 900); };
  return (
    <div class="gm-play">
      <Progress text={r.status} />
      <p class="gm-ask muted small">Which word fits? (dictionary form)</p>
      <p class="gm-sentence" lang="ko">{picked == null ? <>{q.before}<span class="gm-blank">＿＿</span>{q.after}</> : q.sentence}</p>
      <Choices choices={q.choices} picked={picked} onPick={pick} ko />
      {picked != null && <Reveal word={word} correct={q.choices[picked].correct} />}
      {picked != null && !q.choices[picked].correct && <button type="button" class="btn primary gm-next" onClick={r.next}>Next</button>}
    </div>
  );
}

// ---- 5. 초성 quiz ----------------------------------------------------------------------------

function ChoGame({ words, length, onDone }: GameProps) {
  const r = useRound(words.length, length, onDone);
  const word = words[r.i % words.length];
  const cho = choseong(syllables(word.headword));
  const [typed, setTyped] = useState('');
  const [state, setState] = useState<null | { ok: boolean; other?: GameWord }>(null);
  const [hint, setHint] = useState(false);
  const [msg, setMsg] = useState('');
  const input = useRef<HTMLInputElement>(null);
  useEffect(() => { setTyped(''); setState(null); setHint(false); setMsg(''); input.current?.focus(); }, [r.i]);
  const check = async (giveUp = false) => {
    if (state) { r.next(); return; }
    if (giveUp) { setState({ ok: false }); r.record(word, false); return; }
    if (sameKorean(typed, word.headword)) { setState({ ok: true }); r.record(word, true); setTimeout(r.next, 800); return; }
    // another real word with the same initials also counts (as in the party game)
    if (choseong(syllables(typed)) === cho) {
      const other = (await db.gameWordsFor([syllables(typed)]))[0];
      if (other) { setState({ ok: true, other }); r.record(word, true); return; }
    }
    setMsg(choseong(syllables(typed)) !== cho ? `That word's initials are ${choseong(syllables(typed)) || '—'}.` : 'Not a word in the dictionary. Try again.');
  };
  return (
    <div class="gm-play">
      <Progress text={r.status} />
      <p class="gm-ask muted small">Which word has these initial consonants?</p>
      <div class="gm-prompt ko gm-cho" lang="ko">{cho}</div>
      <p class="muted small gm-sub">{word.pos} · {shortGloss(word.gloss, 60)}{hint ? <> · starts with <span lang="ko">{firstSyllable(word.headword)}</span></> : null}</p>
      <form class="gm-type" onSubmit={(e) => { e.preventDefault(); void check(); }}>
        <input ref={input} class="field" lang="ko" autocomplete="off" autocapitalize="off" spellcheck={false} aria-label="Your answer" value={typed} disabled={!!state}
          onInput={(e) => { setTyped((e.currentTarget as HTMLInputElement).value); setMsg(''); }} />
        <button type="submit" class="btn primary">{state ? 'Next' : 'Check'}</button>
      </form>
      {msg && <p class="muted small" role="status">{msg}</p>}
      {!state && (
        <div class="btn-row gm-row">
          {!hint && <button type="button" class="btn" onClick={() => setHint(true)}>Hint</button>}
          <button type="button" class="btn" onClick={() => void check(true)}>I don't know</button>
        </div>
      )}
      {state && <Reveal word={word} correct={state.ok} />}
      {state?.other && <p class="muted small">Your word <span lang="ko">{state.other.headword}</span> ({shortGloss(state.other.gloss)}) also fits.</p>}
    </div>
  );
}

// ---- 6. 끝말잇기 -------------------------------------------------------------------------------

const TURN_SECONDS = 15;
const HINT_COST = 1;
interface Move { by: 'me' | 'app'; word: GameWord }

function ChainGame({ onDone }: { onQuit: () => void; onDone: (a: Answer[], score?: string) => void }) {
  const [moves, setMoves] = useState<Move[]>([]);
  const [typed, setTyped] = useState('');
  const [msg, setMsg] = useState('');
  const [left, setLeft] = useState(TURN_SECONDS);
  const [hint, setHint] = useState<GameWord | null>(null);
  const [hints, setHints] = useState(0);
  const [over, setOver] = useState<null | 'won' | 'lost'>(null);
  const [thinking, setThinking] = useState(true);
  const input = useRef<HTMLInputElement>(null);
  const list = useRef<HTMLOListElement>(null);
  const used = () => moves.map((m) => syllables(m.word.headword));
  const last = moves[moves.length - 1];

  // the app opens with a common noun
  useEffect(() => {
    db.gameWords(1, 40).then(async (ws) => {
      const nouns = ws.filter((w) => w.pos === 'noun' && [2, 3].includes(syllables(w.headword).length));
      for (const w of shuffle(nouns)) {
        // only open with a word the player can answer
        if ((await db.chainCandidates(dueumStarts(lastSyllable(w.headword)), [w.headword], 1)).length) { setMoves([{ by: 'app', word: w }]); break; }
      }
      setThinking(false);
    });
  }, []);
  // turn clock
  useEffect(() => {
    if (over || thinking || !moves.length) return;
    setLeft(TURN_SECONDS);
    const t0 = Date.now();
    const id = setInterval(() => {
      const l = TURN_SECONDS - Math.floor((Date.now() - t0) / 1000);
      setLeft(Math.max(0, l));
      if (l <= 0) { clearInterval(id); setOver('lost'); setMsg('Time is up.'); }
    }, 250);
    input.current?.focus();
    return () => clearInterval(id);
  }, [moves.length, thinking, over]);
  useEffect(() => { list.current?.lastElementChild?.scrollIntoView({ block: 'nearest' }); }, [moves.length]);

  const score = moves.filter((m) => m.by === 'me').length - hints * HINT_COST;
  const play = async () => {
    if (!last || over || thinking) return;
    const w = syllables(typed);
    if (w.length < 2) { setMsg('Words need at least two syllables.'); return; }
    if (!chainFollows(last.word.headword, w)) { setMsg(`Start with ${dueumStarts(lastSyllable(last.word.headword)).join(' or ')}.`); return; }
    if (used().includes(w)) { setMsg('That word was already played.'); return; }
    const entry = await db.chainCheck(w);
    if (!entry) { setMsg(`${w} is not a noun in the dictionary.`); return; }
    setMsg(''); setTyped(''); setHint(null);
    const mine: Move = { by: 'me', word: entry };
    setMoves((m) => [...m, mine]);
    setThinking(true);
    const cands = await db.chainCandidates(dueumStarts(lastSyllable(entry.headword)), [...used(), w], 12);
    await new Promise((res) => setTimeout(res, 500));
    setThinking(false);
    if (!cands.length) { setOver('won'); setMsg('The app has no word left. You win!'); return; }
    const pick = cands[Math.floor(Math.random() * Math.min(cands.length, 8))];
    setMoves((m) => [...m, { by: 'app', word: pick }]);
  };
  const giveHint = async () => {
    if (!last) return;
    const c = await db.chainCandidates(dueumStarts(lastSyllable(last.word.headword)), used(), 8);
    if (!c.length) { setMsg('No word left for you either: well played.'); setOver('won'); return; }
    setHint(c[Math.floor(Math.random() * c.length)]); setHints((h) => h + 1);
  };
  const need = last ? dueumStarts(lastSyllable(last.word.headword)).join(' / ') : '…';
  return (
    <div class="gm-play">
      <Progress text={over ? (over === 'won' ? 'You win' : 'Game over') : thinking ? 'Thinking…' : `${left}s · next starts with ${need}`} />
      <ol class="gm-chain" ref={list} aria-label="Words played">
        {moves.map((m, i) => (
          <li key={i} class={`gm-bubble by-${m.by}`}>
            <a class="hangul" lang="ko" href={href(wordPath(m.word.headword))}>{m.word.headword.replace(/\d+$/, '')}</a>
            {m.word.gloss && <span class="muted small">{shortGloss(m.word.gloss)}</span>}
          </li>
        ))}
      </ol>
      {hint && <p class="muted small gm-hint">Hint: a word meaning “{shortGloss(hint.gloss)}” ({hint.pos}, {syllables(hint.headword).length} syllables).</p>}
      {msg && <p class="muted small" role="status">{msg}</p>}
      {!over ? (
        <>
          <form class="gm-type" onSubmit={(e) => { e.preventDefault(); void play(); }}>
            <input ref={input} class="field" lang="ko" autocomplete="off" autocapitalize="off" spellcheck={false} aria-label="Your word" placeholder={`${need}…`} value={typed} disabled={thinking}
              onInput={(e) => { setTyped((e.currentTarget as HTMLInputElement).value); setMsg(''); }} />
            <button type="submit" class="btn primary" disabled={thinking}>Play</button>
          </form>
          <div class="btn-row gm-row">
            <button type="button" class="btn" disabled={thinking} onClick={() => void giveHint()}>Hint (−{HINT_COST})</button>
            <button type="button" class="btn" onClick={() => { setOver('lost'); setMsg('You gave up.'); }}>Give up</button>
          </div>
        </>
      ) : (
        <button type="button" class="btn primary gm-next" onClick={() => onDone([], `${Math.max(0, score)} point${score === 1 ? '' : 's'} · ${moves.filter((m) => m.by === 'me').length} words played${over === 'won' ? ' · you won' : ''}`)}>See result</button>
      )}
    </div>
  );
}

// ---- results --------------------------------------------------------------------------------

function Results({ result, onAgain, onBack }: { result: { game: GameId; answers: Answer[]; score?: string }; onAgain: () => void; onBack: () => void }) {
  const g = GAMES.find((x) => x.id === result.game)!;
  const right = result.answers.filter((a) => a.correct).length;
  const missed = result.answers.filter((a) => !a.correct);
  return (
    <div class="gm">
      <header class="gm-head gm-head-play">
        <button type="button" class="iconbtn" aria-label="Back" onClick={onBack}><Icon name="back" /></button>
        <h1 class="gm-play-title">{g.ko} · {g.en}</h1>
        <span class="gm-head-gap" />
      </header>
      <div class="gm-play">
        <div class="gm-score">{result.score ?? `${right} / ${result.answers.length}`}</div>
        {result.game !== 'chain' && <p class="muted small gm-center">{missed.length ? 'These come back in your next rounds:' : result.answers.length ? 'All correct. These words will come back later to keep them fresh.' : 'No answers this time.'}</p>}
        {missed.length > 0 && (
          <ul class="gm-missed">
            {missed.map((a, i) => (
              <li key={i}><a href={href(wordPath(a.word.headword))}><span class="hangul" lang="ko">{a.word.headword.replace(/\d+$/, '')}</span>{a.word.hanja ? <> <span class="hanja" lang="zh-Hant">{hanjaKey(a.word.hanja)}</span></> : null}<span class="muted"> · {a.word.gloss}</span></a></li>
            ))}
          </ul>
        )}
        <button type="button" class="btn primary gm-next" onClick={onAgain}>Play again</button>
        <button type="button" class="btn gm-back" onClick={onBack}>Back to games</button>
      </div>
    </div>
  );
}
