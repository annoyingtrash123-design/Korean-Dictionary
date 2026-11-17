// Grammar mini-lessons: the JSON body the engine returns (see core/src/search/lesson.rs and
// pipeline/src/grammar_lessons.rs). Only reviewed lessons are in the pack; the whole lesson is AI-written.
import type { GrammarLessonRow } from '../db/types';

export interface LessonSentence { ko: string; en: string }
export interface LessonExample extends LessonSentence { ctx?: string }
export interface LessonCompare { pattern: string; difference: string; pair: [LessonSentence, LessonSentence]; entry?: { source: string; id: number } }
export interface LessonPitfall { note: string; wrong: string; right: string; en: string }
export interface LessonBody {
  label: string;
  meaning_en: string;
  usage: string;
  examples: LessonExample[];
  compare: LessonCompare[];
  pitfalls: LessonPitfall[];
  topik_tip?: string;
}
export interface Lesson extends LessonBody { level: number; form: string }

/** Parse the rows of one entry; a damaged body is skipped rather than breaking the page. */
export function parseLessons(rows: GrammarLessonRow[]): Lesson[] {
  const out: Lesson[] = [];
  for (const r of rows) {
    try {
      const b = JSON.parse(r.body) as LessonBody;
      if (b && typeof b.meaning_en === 'string' && Array.isArray(b.examples)) {
        out.push({ ...b, compare: b.compare ?? [], pitfalls: b.pitfalls ?? [], level: r.level, form: r.form });
      }
    } catch { /* skip */ }
  }
  return out;
}

/** The official list's form without its homonym digit or trailing comma ("-거들랑1" -> "-거들랑"). */
export const lessonForm = (form: string) => form.replace(/[,\s]+$/, '').replace(/(?<=[가-힣])\d+$/, '');

export const CTX_LABEL: Record<string, string> = {
  conversation: 'Conversation', news: 'News', essay: 'Essay', formal: 'Formal', literary: 'Literary', proverb: 'Proverb', collocation: 'Collocation',
};
