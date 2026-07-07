import { render, screen, fireEvent } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { SkillDetailModal } from './SkillDetailModal';
import type { Skill, Category } from '../../../shared/api/types';

const mockCategories: Category[] = [
  { id: '1', name: 'Design', created_at: '2026-07-06T00:00:00Z' },
  { id: '2', name: 'Development', created_at: '2026-07-06T00:00:00Z' },
];

const mockSkillPack: Skill = {
  id: 'taste-skill',
  kind: 'pack',
  metadata: {
    name: 'Taste Skill Package',
    description: 'A beautiful package containing multiple designer skills',
    version: '1.2.0',
    author: 'Antigravity Team',
  },
  html_content: '<p>Welcome to taste-skill package documentation.</p>',
  members: [
    {
      id: 'taste-skill::sub-1',
      relative_path: 'skills/sub-1',
      metadata: { name: 'Sub Skill One', description: 'This is the first sub skill description', version: '1.0.0' },
      html_content: '<div>Detail document of sub skill one</div>',
    },
    {
      id: 'taste-skill::sub-2',
      relative_path: 'skills/sub-2',
      metadata: { name: 'Sub Skill Two', description: 'This is the second sub skill description', version: '1.0.1' },
      html_content: '<div>Detail document of sub skill two</div>',
    },
  ],
  source: { kind: 'git', url: 'https://github.com/example/taste-skill', installed_commit: 'abcdef123' },
  update_status: 'current',
  has_executable_content: false,
  trusted: true,
  warnings: [],
  user_notes: 'Some user notes',
  category_id: '1',
};

describe('SkillDetailModal', () => {
  it('renders package title, README overview and a grid of child skill cards by default', () => {
    render(
      <SkillDetailModal
        skill={mockSkillPack}
        categories={mockCategories}
        onClose={vi.fn()}
        onUpdate={vi.fn()}
      />
    );

    // Should render package title and README content
    expect(screen.getByText('Taste Skill Package')).toBeInTheDocument();
    expect(screen.getByText('Welcome to taste-skill package documentation.')).toBeInTheDocument();

    // Should render section title and both child skill cards
    expect(screen.getByText('所含子技能 (2)')).toBeInTheDocument();
    expect(screen.getByText('Sub Skill One')).toBeInTheDocument();
    expect(screen.getByText('This is the first sub skill description')).toBeInTheDocument();
    expect(screen.getByText('Sub Skill Two')).toBeInTheDocument();
    expect(screen.getByText('This is the second sub skill description')).toBeInTheDocument();

    // Back button should NOT be visible
    expect(screen.queryByText(/返回/)).not.toBeInTheDocument();

    // Sidebar should not contain the old navigation buttons
    const packMembersHeading = screen.queryByText(/2 个 Skills/);
    expect(packMembersHeading).not.toBeInTheDocument();
  });

  it('navigates to child skill doc when card is clicked and returns when back button is clicked', () => {
    render(
      <SkillDetailModal
        skill={mockSkillPack}
        categories={mockCategories}
        onClose={vi.fn()}
        onUpdate={vi.fn()}
      />
    );

    // Click the first child skill card
    const firstCard = screen.getByText('Sub Skill One').closest('.pack-member-card');
    expect(firstCard).not.toBeNull();
    fireEvent.click(firstCard!);

    // Should now display sub skill content and doc
    expect(screen.getByText('Detail document of sub skill one')).toBeInTheDocument();
    expect(screen.getByText(/返回 Taste Skill Package/)).toBeInTheDocument();

    // Click the back button
    const backBtn = screen.getByText(/返回 Taste Skill Package/);
    fireEvent.click(backBtn);

    // Should return to the grid overview and package README
    expect(screen.getByText('Welcome to taste-skill package documentation.')).toBeInTheDocument();
    expect(screen.getByText('Sub Skill One')).toBeInTheDocument();
    expect(screen.queryByText(/返回/)).not.toBeInTheDocument();
  });

  it('renders "信任此版本" button when untrusted and "已信任此版本" badge when trusted', () => {
    const untrustedSkill = {
      ...mockSkillPack,
      has_executable_content: true,
      trusted: false,
    };

    const { rerender } = render(
      <SkillDetailModal
        skill={untrustedSkill}
        categories={mockCategories}
        onClose={vi.fn()}
        onUpdate={vi.fn()}
        onTrust={vi.fn()}
      />
    );

    // Should render "信任此版本" button
    expect(screen.getByText('信任此版本')).toBeInTheDocument();
    expect(screen.queryByText('已信任此版本')).not.toBeInTheDocument();

    // Rerender with trusted: true
    const trustedSkill = {
      ...mockSkillPack,
      has_executable_content: true,
      trusted: true,
    };

    rerender(
      <SkillDetailModal
        skill={trustedSkill}
        categories={mockCategories}
        onClose={vi.fn()}
        onUpdate={vi.fn()}
        onTrust={vi.fn()}
      />
    );

    // Should now render "已信任此版本" badge, and "信任此版本" button should disappear
    expect(screen.getByText('已信任此版本')).toBeInTheDocument();
    expect(screen.queryByText('信任此版本')).not.toBeInTheDocument();
  });
});
