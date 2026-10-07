import { useAsync } from '../lib/useAsync';
import { db } from '../db/client';
import { history, clearHistory } from '../lib/history';
import { useStore } from '../lib/store';
import { entryPath, href } from '../lib/router';
import { savedPath } from '../lib/entry-key';
import { Hanja, LevelBadge, Pos } from '../components/common';
import { UpdateBanner } from '../components/UpdateBanner';
import type { ResultRow } from '../db/types';
import { todayStr } from '../lib/app-state';

/** Word of the day is cached per date (the query is slow on the full pack) and fetched shortly after first paint. */
function peekWotd(): ResultRow | undefined {
  try { const c = JSON.parse(localStorage.getItem('kd.wotd') || 'null'); if (c?.day === todayStr() && c.row) return c.row; } catch { /* ignore */ }
  return undefined;
}
// engine.wordOfDay() full-scans the 118 MB pack (~11 s cold) and would block searches, so pick from a curated
// beginner list (all krdict level 1–2) by date and look it up by headword (~10 ms); fall back to the engine.
const WOTD_WORDS = '가다 오다 먹다 마시다 보다 듣다 읽다 쓰다 말하다 사랑 친구 학교 학생 선생님 가족 어머니 아버지 형 누나 동생 집 방 문 창문 책 연필 가방 옷 신발 모자 물 밥 김치 과일 사과 우유 커피 차 음식 시장 가게 돈 시간 오늘 어제 내일 아침 저녁 하늘 바다 산 강 꽃 나무 비 눈 바람 날씨 봄 여름 가을 겨울 크다 작다 많다 적다 좋다 나쁘다 예쁘다 춥다 덥다 맛있다 재미있다 어렵다 쉽다 빠르다 느리다 아프다 행복 건강 여행 공부 운동 음악 영화 사진 전화 이름 나이 생일 고향 나라 한국 서울 도시 길 차 버스 지하철 기차 비행기 병원 은행 도서관 식당 공원'.split(' ');
async function pickFromList(day: string): Promise<ResultRow | null> {
  const n = [...day].reduce((h, c) => (h * 31 + c.charCodeAt(0)) >>> 0, 7);
  for (let i = 0; i < 6; i++) {
    const hw = WOTD_WORDS[(n + i * 17) % WOTD_WORDS.length];
    const es = await db.getEntriesByHeadword(hw);
    const e = es.find((x) => x.source === 'krdict' && (x.level ?? 9) <= 2 && x.kind === 'word' && x.gloss);
    if (e) return { source: e.source, id: e.id, headword: e.headword, hanja: e.hanja, pos: e.pos, level: e.level, gloss: e.gloss, kind: e.kind, pack: 'core', rank: e.rank, lang: e.lang, hw_norm: e.hw_norm };
  }
  return null;
}
async function wordOfDay(): Promise<ResultRow | null> {
  const day = todayStr();
  try { const c = JSON.parse(localStorage.getItem('kd.wotd') || 'null'); if (c?.day === day && c.row) return c.row; } catch { /* ignore */ }
  const row = (await pickFromList(day)) ?? (await db.randomWordOfDay(day));
  try { if (row) localStorage.setItem('kd.wotd', JSON.stringify({ day, row })); } catch { /* ignore */ }
  return row;
}

export function Home() {
  const hist = useStore(history);
  const wotd = useAsync(() => wordOfDay(), [], peekWotd);
  const w = wotd.data;
  return (
    <div class="page">
      <UpdateBanner />
      {w && (
        <a class="wotd" href={href(entryPath(w.source, w.id, w.headword))}>
          <div class="eyebrow">Word of the day</div>
          <div class="wotd-head">
            <span class="hangul wotd-hw" lang="ko">{w.headword}</span>
            {w.hanja && <Hanja text={w.hanja} />}
            <LevelBadge level={w.level} />
          </div>
          <div class="wotd-gloss"><Pos pos={w.pos} /> {w.gloss}</div>
        </a>
      )}
      <div class="section-head">
        <h2>Recent</h2>
        {hist.length > 0 && <button type="button" class="link" onClick={() => clearHistory()}>Clear</button>}
      </div>
      {hist.length === 0 ? (
        <p class="muted pad">Words you look up will appear here.</p>
      ) : (
        <ul class="plain list">
          {hist.slice(0, 30).map((h) => (
            <li key={h.key}>
              <a class="row compact" href={href(savedPath(h))}>
                <div class="row-main">
                  <div class="row-head">
                    <span class="hangul hw" lang="ko">{h.headword}</span>{h.hanja && <Hanja text={h.hanja} />}
                  </div>
                  {h.gloss && <div class="row-gloss">{h.gloss}</div>}
                </div>
              </a>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
