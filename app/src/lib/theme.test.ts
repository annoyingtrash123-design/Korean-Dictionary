import { describe, expect, it, beforeEach } from 'vitest';
import { applyTheme, resolveTheme } from './theme';

describe('theme', () => {
  beforeEach(() => {
    document.documentElement.removeAttribute('style');
    document.head.innerHTML = '<meta name="theme-color" content="#fff">';
  });
  it('resolves system theme', () => {
    expect(resolveTheme('system', true)).toBe('dark');
    expect(resolveTheme('system', false)).toBe('light');
    expect(resolveTheme('sepia', true)).toBe('sepia');
  });
  it('sets data-theme, custom colours and font size', () => {
    const t = applyTheme({ theme: 'dark', colors: { accent: '#ff0000', hanja: '#00ff00' }, fontSize: 20 }, document);
    const r = document.documentElement;
    expect(t).toBe('dark');
    expect(r.getAttribute('data-theme')).toBe('dark');
    expect(r.style.getPropertyValue('--accent')).toBe('#ff0000');
    expect(r.style.getPropertyValue('--hanja')).toBe('#00ff00');
    expect(r.style.getPropertyValue('--entry-font-size')).toBe('20px');
    expect(document.querySelector('meta[name=theme-color]')!.getAttribute('content')).toBe('#16181a');
  });
  it('clears overrides when colours are reset', () => {
    applyTheme({ theme: 'light', colors: { accent: '#ff0000' }, fontSize: 17 }, document);
    applyTheme({ theme: 'light', colors: {}, fontSize: 17 }, document);
    expect(document.documentElement.style.getPropertyValue('--accent')).toBe('');
  });
  it('follows the system preference', () => {
    applyTheme({ theme: 'system', colors: {}, fontSize: 17 }, document, true);
    expect(document.documentElement.getAttribute('data-theme')).toBe('dark');
  });
});
