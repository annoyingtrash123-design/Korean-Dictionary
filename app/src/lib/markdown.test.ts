import { describe, expect, it } from 'vitest';
import { inlineParts, parseMarkdown } from './markdown';

describe('markdown subset', () => {
  it('parses headings, wrapped bullets and paragraphs', () => {
    const b = parseMarkdown('# T\n\nHello\nworld\n\n## S\n* one\n  more\n* two\n');
    expect(b).toEqual([
      { t: 'h', level: 1, text: 'T' }, { t: 'p', text: 'Hello world' },
      { t: 'h', level: 2, text: 'S' }, { t: 'ul', items: ['one more', 'two'] },
    ]);
  });
  it('linkifies only http(s) URLs and never emits HTML', () => {
    const p = inlineParts('see **https://a.org/x**. <script>alert(1)</script> (https://b.org)');
    expect(p.filter((x) => x.href).map((x) => x.href)).toEqual(['https://a.org/x', 'https://b.org']);
    expect(p.map((x) => x.text).join('')).toContain('<script>');
  });
});
