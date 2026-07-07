import { render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import type { Skill } from '../../../shared/api/types';
import { SkillCard } from './SkillCard';

const pack = {
  id: 'taste-skill',
  kind: 'pack',
  metadata: { name: 'Taste Skill', description: 'Design skills' },
  html_content: '',
  members: Array.from({ length: 13 }, (_, index) => ({
    id: `taste-skill::${index}`,
    relative_path: `skills/${index}`,
    metadata: { name: `Skill ${index}`, description: 'Member' },
    html_content: '',
  })),
  source: { kind: 'git', url: 'https://github.com/example/taste-skill', installed_commit: 'abcdef123' },
  update_status: 'available',
  has_executable_content: true,
  trusted: false,
  warnings: [],
} as Skill;

describe('SkillCard', () => {
  it('identifies packs and their lifecycle state', () => {
    render(
      <SkillCard
        skill={pack}
        categoryName="设计"
        onOpenDetail={vi.fn()}
        onDelete={vi.fn()}
      />,
    );

    expect(screen.getByText('技能扩展包')).toBeInTheDocument();
    expect(screen.getByText('13 个 Skills')).toBeInTheDocument();
    expect(screen.getByText('有更新')).toBeInTheDocument();
    expect(screen.getByText('需要信任')).toBeInTheDocument();
  });

  it('renders custom description and custom badge if provided', () => {
    const skillWithCustom = {
      ...pack,
      custom_description: '这是自定义的技能包说明'
    };
    
    render(
      <SkillCard
        skill={skillWithCustom}
        categoryName="设计"
        onOpenDetail={vi.fn()}
      />
    );

    expect(screen.getByText('自定义')).toBeInTheDocument();
    expect(screen.getByText('这是自定义的技能包说明')).toBeInTheDocument();
    expect(screen.queryByText('Design skills')).not.toBeInTheDocument();
  });
});
