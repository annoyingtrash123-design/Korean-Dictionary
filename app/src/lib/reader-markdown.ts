// Tiny, safe markdown for Reader notes: headings, paragraphs, bullet/numbered lists, **bold**, *italic*, `code`,
// and http(s) links. Produces a data structure; the view renders it as elements (never innerHTML).
export type Inline = { k: 'text'; s: string } | { k: 'b'; c: Inline[] } | { k: 'i'; c: Inline[] } | { k: 'code'; c: Inline[] } | { k: 'a'; href: string; c: Inline[] };
export type MdBlock =
  | { t: 'h'; level: number; c: Inline[] }
  | { t: 'p'; c: Inline[] }
  | { t: 'ul' | 'ol'; items: Inline[][] };

const safeUrl = (u: string) => /^https?:\/\/[^\s<>"']+$/i.test(u);

export function parseInline(src: string): Inline[] {
  const out: Inline[] = [];
  let buf = '';
  const flush = () => { if (buf) out.push({ k: 'text', s: buf }); buf = ''; };
  let i = 0;
  while (i < src.length) {
    const rest = src.slice(i);
    let m: RegExpExecArray | null;
    if ((m = /^\*\*(?=\S)(.+?)(?<=\S)\*\*/s.exec(rest))) { flush(); out.push({ k: 'b', c: parseInline(m[1]) }); i += m[0].length; }
    else if ((m = /^\*(?=[^\s*])(.+?)(?<=[^\s*])\*(?!\*)/s.exec(rest))) { flush(); out.push({ k: 'i', c: parseInline(m[1]) }); i += m[0].length; }
    else if ((m = /^`([^`]+)`/.exec(rest))) { flush(); out.push({ k: 'code', c: [{ k: 'text', s: m[1] }] }); i += m[0].length; }
    else if ((m = /^\[([^\]]+)\]\(([^)\s]+)\)/.exec(rest))) {
      if (safeUrl(m[2])) { flush(); out.push({ k: 'a', href: m[2], c: parseInline(m[1]) }); } else buf += m[1];
      i += m[0].length;
    } else { buf += src[i]; i++; }
  }
  flush();
  return out;
}

export function parseNotes(src: string): MdBlock[] {
  const out: MdBlock[] = [];
  let para: string[] = [];
  let list: { t: 'ul' | 'ol'; items: string[] } | null = null;
  const flushP = () => { if (para.length) out.push({ t: 'p', c: parseInline(para.join(' ')) }); para = []; };
  const flushL = () => { if (list) out.push({ t: list.t, items: list.items.map(parseInline) }); list = null; };
  for (const raw of (src ?? '').replace(/\r/g, '').split('\n')) {
    const line = raw.trimEnd();
    const h = /^(#{1,6})\s+(.*)$/.exec(line);
    const ul = /^\s{0,3}[-*+]\s+(.*)$/.exec(line);
    const ol = /^\s{0,3}\d+[.)]\s+(.*)$/.exec(line);
    if (!line.trim()) { flushP(); flushL(); }
    else if (h) { flushP(); flushL(); out.push({ t: 'h', level: h[1].length, c: parseInline(h[2]) }); }
    else if (ul || ol) {
      flushP();
      const t = ul ? 'ul' : 'ol';
      if (list && list.t !== t) flushL();
      (list ??= { t, items: [] }).items.push((ul ?? ol)![1]);
    } else if (list && /^\s+\S/.test(line)) list.items[list.items.length - 1] += ' ' + line.trim();
    else { flushL(); para.push(line.trim()); }
  }
  flushP(); flushL();
  return out;
}
