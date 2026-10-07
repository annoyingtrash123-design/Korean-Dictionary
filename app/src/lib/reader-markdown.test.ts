import { describe, expect, it } from 'vitest';
import { parseInline, parseNotes } from './reader-markdown';

describe('reader markdown', () => {
  it('parses paragraphs, bold, italic, lists and headings', () => {
    const b = parseNotes('# Title\n\n**배경** 이야기 *italic*\n\n- one\n- two\n  wrapped\n\n1. first\n2. second');
    expect(b.map((x) => x.t)).toEqual(['h', 'p', 'ul', 'ol']);
    const p = b[1] as any;
    expect(p.c[0]).toEqual({ k: 'b', c: [{ k: 'text', s: '배경' }] });
    expect(p.c.at(-1)).toEqual({ k: 'i', c: [{ k: 'text', s: 'italic' }] });
    expect((b[2] as any).items).toHaveLength(2);
    expect(JSON.stringify((b[2] as any).items[1])).toContain('two wrapped');
  });
  it('never produces HTML: tags stay literal text', () => {
    const inl = parseInline('<img src=x onerror=alert(1)> and <script>alert(1)</script> **b**');
    const texts = inl.filter((n) => n.k === 'text').map((n: any) => n.s).join('');
    expect(texts).toContain('<img src=x onerror=alert(1)>');
    expect(inl.some((n) => (n as any).k === 'html')).toBe(false);
  });
  it('only allows http(s) links', () => {
    expect(parseInline('[ok](https://example.org/a)')[0]).toMatchObject({ k: 'a', href: 'https://example.org/a' });
    const bad = parseInline('[x](javascript:alert(1)) [y](data:text/html,hi)');
    expect(bad.some((n) => n.k === 'a')).toBe(false);
  });
  it('keeps unmatched markers as text and handles empty input', () => {
    expect(parseInline('a * b ** c')).toEqual([{ k: 'text', s: 'a * b ** c' }]);
    expect(parseNotes('')).toEqual([]);
  });
});
