// Tiny markdown subset for the licence text: headings, bullet lists (with wrapped lines), paragraphs.
// Produces a plain data structure; the view renders it with normal elements (no innerHTML).
export type Block =
  | { t: 'h'; level: number; text: string }
  | { t: 'ul'; items: string[] }
  | { t: 'p'; text: string };

export function parseMarkdown(src: string): Block[] {
  const out: Block[] = [];
  let para: string[] = [];
  let list: string[] | null = null;
  const flushPara = () => { if (para.length) out.push({ t: 'p', text: para.join(' ') }); para = []; };
  const flushList = () => { if (list) out.push({ t: 'ul', items: list }); list = null; };
  for (const raw of src.replace(/\r/g, '').split('\n')) {
    const line = raw.trimEnd();
    const h = /^(#{1,6})\s+(.*)$/.exec(line);
    const li = /^\s{0,3}[*+-]\s+(.*)$/.exec(line);
    if (!line.trim()) { flushPara(); flushList(); }
    else if (h) { flushPara(); flushList(); out.push({ t: 'h', level: h[1].length, text: h[2] }); }
    else if (li) { flushPara(); (list ??= []).push(li[1]); }
    else if (list && /^\s+\S/.test(line)) list[list.length - 1] += ' ' + line.trim();   // wrapped list item
    else { flushList(); para.push(line.trim()); }
  }
  flushPara(); flushList();
  return out;
}

/** Split text into plain and http(s) URL parts; also drops simple emphasis/code markers. */
export function inlineParts(text: string): { text: string; href?: string }[] {
  const clean = text.replace(/\*\*([^*]+)\*\*/g, '$1').replace(/`([^`]+)`/g, '$1');
  const parts: { text: string; href?: string }[] = [];
  let last = 0;
  for (const m of clean.matchAll(/https?:\/\/[^\s)]+[^\s).,;]/g)) {
    if (m.index! > last) parts.push({ text: clean.slice(last, m.index) });
    parts.push({ text: m[0], href: m[0] });
    last = m.index! + m[0].length;
  }
  if (last < clean.length) parts.push({ text: clean.slice(last) });
  return parts;
}
