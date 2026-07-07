import React from 'react';
import { Download, Package, ShieldAlert, Trash2 } from 'lucide-react';
import { Skill, SkillMember, SkillUpdateStatus } from '../../../shared/api/types';

interface Props {
  skill: Skill;
  categoryName: string;
  onOpenDetail: () => void;
  onDelete?: (e: React.MouseEvent) => void;
  member?: SkillMember;
  updateStatus?: SkillUpdateStatus;
  onUpdate?: (e: React.MouseEvent) => void;
}

export function SkillCard({
  skill,
  categoryName,
  onOpenDetail,
  onDelete,
  member,
  updateStatus = skill.update_status,
  onUpdate,
}: Props) {
  const metadata = member?.metadata ?? skill.metadata;
  const customDescription = member ? member.custom_description : skill.custom_description;
  return (
    <div className="skill-card" onClick={onOpenDetail}>
      <div className="skill-card-header">
        <div className="skill-card-tags">
          <span className="category-tag">{categoryName}</span>
          {skill.kind === 'pack' && !member && (
            <span className="pack-tag"><Package size={12} />技能扩展包</span>
          )}
          {member && <span className="pack-tag">来自 {skill.metadata.name}</span>}
        </div>
        <div className="skill-card-controls" onClick={(e) => e.stopPropagation()}>
          {updateStatus === 'available' && onUpdate && (
            <button className="update-btn" onClick={onUpdate} title="安装更新">
              <Download size={14} />
            </button>
          )}
          {onDelete && (
            <button className="delete-btn" onClick={onDelete}>
              <Trash2 size={14} />
            </button>
          )}
        </div>
      </div>
      <h4>{metadata.name}</h4>
      <p className="skill-description">
        {customDescription ? (
          <>
            <span className="custom-badge">自定义</span>
            {customDescription}
          </>
        ) : (
          metadata.description
        )}
      </p>
      <div className="skill-state-row">
        {skill.kind === 'pack' && !member && <span>{skill.members.length} 个 Skills</span>}
        {updateStatus === 'available' && <span className="state-pill state-pill--update">有更新</span>}
        {updateStatus === 'dirty' && <span className="state-pill">本地已修改</span>}
        {skill.has_executable_content && !skill.trusted && (
          <span className="state-pill state-pill--warning"><ShieldAlert size={12} />需要信任</span>
        )}
      </div>
      <div className="skill-card-footer">
        {metadata.version && <span className="version-badge">v{metadata.version}</span>}
        {skill.source.kind === 'git' && <span>Git · {skill.source.installed_commit?.slice(0, 7) ?? '未知'}</span>}
        {metadata.author && <span className="author-badge">by {metadata.author}</span>}
      </div>
    </div>
  );
}
