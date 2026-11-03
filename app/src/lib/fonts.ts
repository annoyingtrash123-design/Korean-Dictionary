import type { FontName } from './settings';

/** The Korean font choices: [setting, label, CSS family for the preview]. */
export const FONTS: { id: FontName; label: string; ko: string; family: string }[] = [
  { id: 'system', label: 'System', ko: '기본', family: "'Apple SD Gothic Neo', 'Noto Sans KR', 'Noto Sans CJK KR', system-ui, sans-serif" },
  { id: 'dodum', label: 'Gowun Dodum', ko: '고운돋움', family: "'KD Gowun Dodum', sans-serif" },
  { id: 'batang', label: 'Gowun Batang', ko: '고운바탕', family: "'KD Gowun Batang', serif" },
  { id: 'myeongjo', label: 'Nanum Myeongjo', ko: '나눔명조', family: "'KD Nanum Myeongjo', serif" },
  { id: 'pen', label: 'Nanum Pen (handwriting)', ko: '나눔손글씨 펜', family: "'KD Nanum Pen', cursive" },
];
