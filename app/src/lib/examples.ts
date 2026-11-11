/** Example sentences: dialogue turns and speech. */

const SPEAKERS = ['가', '나', '다', '라'];
/** A dialogue example as speaker turns, or null for a plain sentence. Lines labelled "가: …" keep
 *  their labels; a dictionary dialogue (type "dialogue", one line per turn, no labels) gets
 *  가 / 나 in turn. */
export function dialogueTurns(ko: string, type?: string | null): { who: string; text: string }[] | null {
  const lines = ko.split('\n').map((l) => l.trim()).filter(Boolean);
  if (lines.length < 2) return null;
  const labelled = lines.map((l) => { const m = /^([가-힣A-Za-z]{1,3})\s*[:：]\s*(.+)$/.exec(l); return m ? { who: m[1], text: m[2] } : null; });
  if (labelled.every(Boolean)) return labelled as { who: string; text: string }[];
  return type === 'dialogue' ? lines.map((text, i) => ({ who: SPEAKERS[i % 2], text })) : null;
}

/** The text to read aloud: speaker labels dropped. */
export const speakable = (ko: string, type?: string | null) => (dialogueTurns(ko, type)?.map((t) => t.text).join(' ') ?? ko.replace(/\n/g, ' '));

export const canSpeak = () => typeof window !== 'undefined' && 'speechSynthesis' in window && typeof SpeechSynthesisUtterance !== 'undefined';

/** Read Korean aloud with the device's own voice (offline on most phones). */
export function speak(ko: string, type?: string | null) {
  if (!canSpeak()) return;
  const s = window.speechSynthesis;
  s.cancel();
  const u = new SpeechSynthesisUtterance(speakable(ko, type));
  u.lang = 'ko-KR';
  const v = s.getVoices().find((x) => x.lang?.toLowerCase().startsWith('ko'));
  if (v) u.voice = v;
  u.rate = 0.9;
  s.speak(u);
}
