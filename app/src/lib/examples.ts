/** Example sentences: dialogue turns and speech. */

/** A dictionary dialogue example ("가: …\n나: …") as speaker turns; null for a plain sentence. */
export function dialogueTurns(ko: string): { who: string; text: string }[] | null {
  const lines = ko.split('\n').map((l) => l.trim()).filter(Boolean);
  if (lines.length < 2) return null;
  const turns = lines.map((l) => { const m = /^([가-힣A-Za-z]{1,3})\s*[:：]\s*(.+)$/.exec(l); return m ? { who: m[1], text: m[2] } : null; });
  return turns.every(Boolean) ? (turns as { who: string; text: string }[]) : null;
}

/** The text to read aloud: speaker labels dropped. */
export const speakable = (ko: string) => (dialogueTurns(ko)?.map((t) => t.text).join(' ') ?? ko.replace(/\n/g, ' '));

export const canSpeak = () => typeof window !== 'undefined' && 'speechSynthesis' in window && typeof SpeechSynthesisUtterance !== 'undefined';

/** Read Korean aloud with the device's own voice (offline on most phones). */
export function speak(ko: string) {
  if (!canSpeak()) return;
  const s = window.speechSynthesis;
  s.cancel();
  const u = new SpeechSynthesisUtterance(speakable(ko));
  u.lang = 'ko-KR';
  const v = s.getVoices().find((x) => x.lang?.toLowerCase().startsWith('ko'));
  if (v) u.voice = v;
  u.rate = 0.9;
  s.speak(u);
}
