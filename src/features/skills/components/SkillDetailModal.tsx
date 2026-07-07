import React, { useState } from 'react';
import { Download, Package, ShieldCheck, X } from 'lucide-react';
import { Skill, SkillMember, Category, SkillUpdateStatus } from '../../../shared/api/types';

interface Props {
  skill: Skill;
  categories: Category[];
  onClose: () => void;
  onUpdate: (categoryId: string | null, userNotes: string | null) => void;
  initialMember?: SkillMember;
  updateStatus?: SkillUpdateStatus;
  onTrust?: () => void;
  onInstallUpdate?: () => void;
}

export function SkillDetailModal({
  skill,
  categories,
  onClose,
  onUpdate,
  initialMember,
  updateStatus = skill.update_status,
  onTrust,
  onInstallUpdate,
}: Props) {
  const [notes, setNotes] = useState(skill.user_notes || '');
  const [catId, setCatId] = useState(skill.category_id || '');
  const [selectedMember, setSelectedMember] = useState<SkillMember | undefined>(initialMember);

  const handleSave = () => {
    onUpdate(catId || null, notes || null);
    onClose();
  };

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal-body" onClick={(e) => e.stopPropagation()}>
        <div className="modal-header">
          <h3>{skill.kind === 'pack' ? '技能扩展包' : '技能详情'}</h3>
          <button className="close-btn" onClick={onClose}>
            <X size={20} />
          </button>
        </div>
        <div className="modal-grid-content">
          <div className="modal-markdown-area">
            {selectedMember ? (
              <>
                <button className="member-back" onClick={() => setSelectedMember(undefined)}>
                  ← 返回 {skill.metadata.name}
                </button>
                <h1>{selectedMember.metadata.name}</h1>
                <div
                  className="markdown-body"
                  dangerouslySetInnerHTML={{ __html: selectedMember.html_content || '<p>暂无文档说明</p>' }}
                />
              </>
            ) : (
              <>
                <h1>{skill.metadata.name}</h1>
                {skill.html_content && (
                  <div
                    className="markdown-body"
                    dangerouslySetInnerHTML={{ __html: skill.html_content }}
                  />
                )}
                {skill.kind === 'pack' && (
                  <div className="pack-members-section">
                    <h3 className="section-title" style={{ marginTop: '2rem', marginBottom: '1rem', borderBottom: '1px solid var(--color-outline)', paddingBottom: '0.5rem' }}>
                      所含子技能 ({skill.members.length})
                    </h3>
                    <div className="pack-members-grid">
                      {skill.members.map((member) => (
                        <div
                          key={member.id}
                          className="pack-member-card"
                          onClick={() => setSelectedMember(member)}
                        >
                          <div className="pack-member-card__header">
                            <Package size={16} className="pack-member-card__icon" />
                            <h4>{member.metadata.name}</h4>
                          </div>
                          <p className="pack-member-card__desc">{member.metadata.description || '暂无描述信息'}</p>
                          <div className="pack-member-card__footer">
                            <span className="pack-member-card__version">
                              {member.metadata.version ? `v${member.metadata.version}` : ''}
                            </span>
                            <span className="pack-member-card__action">查看详情 →</span>
                          </div>
                        </div>
                      ))}
                    </div>
                  </div>
                )}
              </>
            )}
          </div>
          <div className="modal-meta-editor">
            {skill.warnings.length > 0 && (
              <div className="skill-warnings">
                <strong>检测警告</strong>
                {skill.warnings.map((warning) => <p key={warning}>{warning}</p>)}
              </div>
            )}
            <div className="form-group">
              <label>设置分类</label>
              <select value={catId} onChange={(e) => setCatId(e.target.value)}>
                <option value="">未分类</option>
                {categories.map((c) => (
                  <option key={c.id} value={c.id}>
                    {c.name}
                  </option>
                ))}
              </select>
            </div>
            <div className="form-group flex-fill">
              <label>技能使用说明与备注</label>
              <textarea
                placeholder="在此添加该技能的个性化使用备注或说明..."
                value={notes}
                onChange={(e) => setNotes(e.target.value)}
                style={{ minHeight: '18rem' }}
              />
            </div>
            <div className="actions-footer">
              <div className="actions-footer__left" style={{ marginRight: 'auto', display: 'flex', alignItems: 'center', gap: '8px' }}>
                {updateStatus === 'available' && onInstallUpdate && (
                  <button className="button button--secondary" onClick={onInstallUpdate}>
                    <Download size={15} /> 安装更新
                  </button>
                )}
                {skill.has_executable_content && (
                  skill.trusted ? (
                    <span className="trusted-badge" style={{ display: 'inline-flex', alignItems: 'center', gap: '4px', fontSize: '0.85rem', color: 'var(--color-primary-ink)', fontWeight: 500 }}>
                      <ShieldCheck size={15} /> 已信任此版本
                    </span>
                  ) : (
                    onTrust && (
                      <button className="button button--secondary" onClick={onTrust}>
                        <ShieldCheck size={15} /> 信任此版本
                      </button>
                    )
                  )
                )}
              </div>
              <button className="button button--secondary" onClick={onClose}>
                取消
              </button>
              <button className="button button--primary" onClick={handleSave}>
                保存更改
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
