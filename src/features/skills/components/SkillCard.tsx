import React from 'react';
import { Trash2 } from 'lucide-react';
import { Skill } from '../../../shared/api/types';

interface Props {
  skill: Skill;
  categoryName: string;
  onOpenDetail: () => void;
  onToggleEnable: (e: React.MouseEvent) => void;
  onDelete: (e: React.MouseEvent) => void;
}

export function SkillCard({ skill, categoryName, onOpenDetail, onToggleEnable, onDelete }: Props) {
  return (
    <div className="skill-card" onClick={onOpenDetail}>
      <div className="skill-card-header">
        <span className="category-tag">{categoryName}</span>
        <div className="skill-card-controls" onClick={(e) => e.stopPropagation()}>
          <input
            type="checkbox"
            className="toggle-switch"
            checked={skill.is_enabled}
            onChange={() => {}}
            onClick={onToggleEnable}
          />
          <button className="delete-btn" onClick={onDelete}>
            <Trash2 size={14} />
          </button>
        </div>
      </div>
      <h4>{skill.metadata.name}</h4>
      <p className="skill-description">{skill.metadata.description}</p>
      <div className="skill-card-footer">
        {skill.metadata.version && <span className="version-badge">v{skill.metadata.version}</span>}
        {skill.metadata.author && <span className="author-badge">by {skill.metadata.author}</span>}
      </div>
    </div>
  );
}
