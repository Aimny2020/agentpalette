import { act, render, screen } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { AppProviders } from './AppProviders';
import { useThemeStore } from '../../shared/theme/themeStore';

describe('AppProviders', () => {
  beforeEach(() => {
    document.documentElement.removeAttribute('data-theme');
    useThemeStore.setState({ theme: 'system' });
  });

  it('renders application content inside shared providers', () => {
    render(<AppProviders><span>Workbench ready</span></AppProviders>);

    expect(screen.getByText('Workbench ready')).toBeInTheDocument();
  });

  it('tracks system appearance changes while using the system theme', () => {
    let onChange: ((event: MediaQueryListEvent) => void) | undefined;
    vi.spyOn(window, 'matchMedia').mockReturnValue({
      matches: false,
      media: '(prefers-color-scheme: dark)',
      onchange: null,
      addListener: vi.fn(),
      removeListener: vi.fn(),
      addEventListener: vi.fn((_type, listener) => { onChange = listener as (event: MediaQueryListEvent) => void; }),
      removeEventListener: vi.fn(),
      dispatchEvent: vi.fn(),
    });

    render(<AppProviders><span>Workbench ready</span></AppProviders>);
    expect(document.documentElement).toHaveAttribute('data-theme', 'light');

    act(() => onChange?.({ matches: true } as MediaQueryListEvent));
    expect(document.documentElement).toHaveAttribute('data-theme', 'dark');
  });
});
