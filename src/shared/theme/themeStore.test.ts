import { beforeEach, describe, expect, it } from 'vitest';

import { applyTheme, useThemeStore } from './themeStore';

describe('themeStore', () => {
  beforeEach(() => {
    localStorage.clear();
    document.documentElement.removeAttribute('data-theme');
    useThemeStore.setState({ theme: 'system' });
  });

  it('resolves system preference to dark', () => {
    applyTheme('system', true);

    expect(document.documentElement).toHaveAttribute('data-theme', 'dark');
  });

  it('persists explicit theme and applies it to the document', () => {
    useThemeStore.getState().setTheme('dark');

    expect(document.documentElement).toHaveAttribute('data-theme', 'dark');
    expect(localStorage.getItem('agentpalette-theme')).toContain('dark');
  });
});
