import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import { PageState } from './PageState';

describe('PageState', () => {
  it('renders loading, empty, error, and content states', () => {
    const retry = vi.fn();
    const view = render(<PageState state="loading" label="正在读取项目" />);
    expect(screen.getByRole('status')).toHaveTextContent('正在读取项目');

    view.rerender(
      <PageState state="empty" title="暂无项目" description="添加本地项目以开始。" />,
    );
    expect(screen.getByText('暂无项目')).toBeInTheDocument();

    view.rerender(
      <PageState
        state="error"
        title="读取失败"
        description="请检查数据目录。"
        onRetry={retry}
      />,
    );
    fireEvent.click(screen.getByRole('button', { name: 'Retry' }));
    expect(retry).toHaveBeenCalledOnce();

    view.rerender(<PageState state="content"><p>项目内容</p></PageState>);
    expect(screen.getByText('项目内容')).toBeInTheDocument();
  });
});
