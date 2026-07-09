import { render, screen, fireEvent, act } from '@testing-library/react';
import { describe, expect, it, vi, beforeEach } from 'vitest';
import { ImportSkillModal } from './ImportSkillModal';
import { inspectSkillImport } from '../../../shared/api/tauriClient';

vi.mock('../../../shared/api/tauriClient', () => ({
  inspectSkillImport: vi.fn(),
}));

describe('ImportSkillModal', () => {
  beforeEach(() => {
    vi.mocked(inspectSkillImport).mockReset();
  });

  it('renders installation ID and normalized Git source on inspection preview', async () => {
    vi.mocked(inspectSkillImport).mockResolvedValue({
      name: 'Sample Pack',
      kind: 'pack',
      member_count: 3,
      has_executable_content: false,
      warnings: [],
      install_id: 'owner-sample-pack',
      normalized_source: 'github.com/owner/sample-pack',
    });

    const handleClose = vi.fn();
    const handleImport = vi.fn();

    render(<ImportSkillModal onClose={handleClose} onImport={handleImport} />);

    // Switch to Git mode
    const gitTab = screen.getByText('Git 仓库导入');
    fireEvent.click(gitTab);

    // Enter URL and click check
    const input = screen.getByPlaceholderText('https://github.com/org/my-skill-repo.git');
    fireEvent.change(input, { target: { value: 'https://github.com/owner/sample-pack.git' } });

    const checkBtn = screen.getByText('检查内容');
    await act(async () => {
      fireEvent.click(checkBtn);
    });

    expect(inspectSkillImport).toHaveBeenCalledWith('https://github.com/owner/sample-pack.git', 'git');
    
    // Verify fields are rendered
    expect(screen.getByText('Sample Pack')).toBeInTheDocument();
    expect(screen.getByText('安装 ID：owner-sample-pack')).toBeInTheDocument();
    expect(screen.getByText('Git 来源：github.com/owner/sample-pack')).toBeInTheDocument();
    expect(screen.getByText('技能扩展包 · 3 个 Skills')).toBeInTheDocument();

    // The submit button should be enabled
    const submitBtn = screen.getByText('确认导入');
    expect(submitBtn).toBeEnabled();

    // Confirm import triggers callback
    await act(async () => {
      fireEvent.click(submitBtn);
    });
    expect(handleImport).toHaveBeenCalledWith('https://github.com/owner/sample-pack.git', 'git');
    expect(handleClose).toHaveBeenCalled();
  });

  it('disables confirmation button and shows duplicate warning when source is duplicate', async () => {
    vi.mocked(inspectSkillImport).mockResolvedValue({
      name: 'Sample Standalone',
      kind: 'standalone',
      member_count: 1,
      has_executable_content: false,
      warnings: [],
      install_id: 'owner-sample-standalone',
      normalized_source: 'github.com/owner/sample-standalone',
      duplicate_skill_id: 'owner-sample-standalone',
    });

    const handleClose = vi.fn();
    const handleImport = vi.fn();

    render(<ImportSkillModal onClose={handleClose} onImport={handleImport} />);

    const gitTab = screen.getByText('Git 仓库导入');
    fireEvent.click(gitTab);

    const input = screen.getByPlaceholderText('https://github.com/org/my-skill-repo.git');
    fireEvent.change(input, { target: { value: 'https://github.com/owner/sample-standalone.git' } });

    const checkBtn = screen.getByText('检查内容');
    await act(async () => {
      fireEvent.click(checkBtn);
    });

    // Verification of warning message and disabled submit button
    expect(screen.getByText('已安装为 owner-sample-standalone，不会创建重复副本。')).toBeInTheDocument();
    const submitBtn = screen.getByText('确认导入');
    expect(submitBtn).toBeDisabled();
  });

  it('does not show warning when different repo name has same base name but different source', async () => {
    vi.mocked(inspectSkillImport).mockResolvedValue({
      name: 'Different Repo Name',
      kind: 'standalone',
      member_count: 1,
      has_executable_content: false,
      warnings: [],
      install_id: 'another-skills',
      normalized_source: 'github.com/another/skills',
    });

    render(<ImportSkillModal onClose={vi.fn()} onImport={vi.fn()} />);

    const gitTab = screen.getByText('Git 仓库导入');
    fireEvent.click(gitTab);

    const input = screen.getByPlaceholderText('https://github.com/org/my-skill-repo.git');
    fireEvent.change(input, { target: { value: 'https://github.com/another/skills.git' } });

    const checkBtn = screen.getByText('检查内容');
    await act(async () => {
      fireEvent.click(checkBtn);
    });

    // Warning is absent and confirm button is enabled
    expect(screen.queryByText(/已安装为/)).not.toBeInTheDocument();
    const submitBtn = screen.getByText('确认导入');
    expect(submitBtn).toBeEnabled();
  });
});
