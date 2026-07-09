import { render, screen, fireEvent, act } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { ConfirmDeleteModal } from './ConfirmDeleteModal';
import type { Skill } from '../../../shared/api/types';

const mockSkill: Skill = {
  id: 'taste-skill',
  kind: 'pack',
  metadata: {
    name: 'Taste Skill',
    description: 'Design skills',
  },
  html_content: '',
  members: [],
  source: { kind: 'local' },
  update_status: 'not_applicable',
  has_executable_content: false,
  trusted: true,
  warnings: [],
};

describe('ConfirmDeleteModal', () => {
  it('renders default confirmation state and handles standard confirm', async () => {
    const handleConfirm = vi.fn().mockResolvedValue(undefined);
    const handleClose = vi.fn();

    render(
      <ConfirmDeleteModal
        skill={mockSkill}
        onClose={handleClose}
        onConfirm={handleConfirm}
      />
    );

    expect(screen.getByText('删除技能')).toBeInTheDocument();
    expect(screen.getByText(/确定要永久删除技能/)).toBeInTheDocument();
    expect(screen.getByText('Taste Skill')).toBeInTheDocument();

    const confirmBtn = screen.getByText('确认删除');
    await act(async () => {
      fireEvent.click(confirmBtn);
    });

    expect(handleConfirm).toHaveBeenCalledWith(false);
    expect(handleClose).toHaveBeenCalled();
  });

  it('transitions to State B warning when occupied error is thrown', async () => {
    const errorMsg = 'Skill Pack is enabled in projects: project-alpha, project-beta';
    const handleConfirm = vi.fn()
      .mockRejectedValueOnce(new Error(errorMsg)) // First standard delete fails
      .mockResolvedValueOnce(undefined);          // Second force delete succeeds
    const handleClose = vi.fn();

    render(
      <ConfirmDeleteModal
        skill={mockSkill}
        onClose={handleClose}
        onConfirm={handleConfirm}
      />
    );

    const confirmBtn = screen.getByText('确认删除');
    await act(async () => {
      fireEvent.click(confirmBtn);
    });

    // Should transition to State B: Project occupied warning
    expect(screen.getByText('该技能正在被项目使用')).toBeInTheDocument();
    expect(screen.getByText('project-alpha')).toBeInTheDocument();
    expect(screen.getByText('project-beta')).toBeInTheDocument();
    expect(screen.queryByText('确认删除')).not.toBeInTheDocument();

    // Click force delete button
    const forceConfirmBtn = screen.getByText('一键移除并彻底删除');
    await act(async () => {
      fireEvent.click(forceConfirmBtn);
    });

    expect(handleConfirm).toHaveBeenCalledWith(true);
    expect(handleClose).toHaveBeenCalled();
  });

  it('transitions to State B warning when occupied error is thrown in details (Tauri error structure)', async () => {
    const handleConfirm = vi.fn()
      .mockRejectedValueOnce({
        message: '本地数据库暂时不可用，请重试。',
        details: 'Skill Pack is enabled in projects: project-x, project-y'
      })
      .mockResolvedValueOnce(undefined);
    const handleClose = vi.fn();

    render(
      <ConfirmDeleteModal
        skill={mockSkill}
        onClose={handleClose}
        onConfirm={handleConfirm}
      />
    );

    const confirmBtn = screen.getByText('确认删除');
    await act(async () => {
      fireEvent.click(confirmBtn);
    });

    expect(screen.getByText('该技能正在被项目使用')).toBeInTheDocument();
    expect(screen.getByText('project-x')).toBeInTheDocument();
    expect(screen.getByText('project-y')).toBeInTheDocument();
    expect(screen.queryByText('确认删除')).not.toBeInTheDocument();
  });
});
