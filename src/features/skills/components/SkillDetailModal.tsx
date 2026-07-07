import React, { useState } from 'react';
import { Download, Package, ShieldCheck, X } from 'lucide-react';
import { Skill, SkillMember, Category, SkillUpdateStatus } from '../../../shared/api/types';
import { saveCustomDescription } from '../../../shared/api/tauriClient';

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
  const [catId, setCatId] = useState(skill.category_id || '');
  const [selectedMember, setSelectedMember] = useState<SkillMember | undefined>(initialMember);

  // Track custom descriptions for current editing target
  const getTargetInitialDesc = (member?: SkillMember) => {
    return member ? (member.custom_description || '') : (skill.custom_description || '');
  };

  const [customDesc, setCustomDesc] = useState(getTargetInitialDesc(initialMember));
  const [initialCustomDesc, setInitialCustomDesc] = useState(getTargetInitialDesc(initialMember));

  const isDescDirty = customDesc !== initialCustomDesc;
  const isMetaDirty = !selectedMember && (catId !== (skill.category_id || ''));
  const isLengthExceeded = customDesc.length > 2000;

  const checkDirtyAndProceed = () => {
    if (isDescDirty) {
      return window.confirm('您对“技能说明”的修改尚未保存，确定要放弃修改吗？');
    }
    return true;
  };

  const handleSwitchTarget = (nextMember: SkillMember | undefined) => {
    if (!checkDirtyAndProceed()) return;
    setSelectedMember(nextMember);
    const desc = getTargetInitialDesc(nextMember);
    setCustomDesc(desc);
    setInitialCustomDesc(desc);
  };

  const handleCloseAttempt = () => {
    if (isDescDirty || isMetaDirty) {
      if (!window.confirm('您有未保存的更改，确定要关闭并放弃所有更改吗？')) {
        return;
      }
    }
    onClose();
  };

  const handleSave = async () => {
    if (isLengthExceeded) return;
    try {
      if (selectedMember) {
        await saveCustomDescription(selectedMember.id, 'member', customDesc || null);
        selectedMember.custom_description = customDesc || undefined;
      } else {
        await onUpdate(catId || null, skill.user_notes || null);
        await saveCustomDescription(skill.id, 'package', customDesc || null);
        skill.custom_description = customDesc || undefined;
        skill.category_id = catId || undefined;
      }
      setInitialCustomDesc(customDesc);
      onClose();
    } catch (err) {
      alert(`保存失败: ${err instanceof Error ? err.message : String(err)}`);
    }
  };

  return (
    <div className="modal-overlay" onClick={handleCloseAttempt}>
      <div className="modal-body" onClick={(e) => e.stopPropagation()}>
        <div className="modal-header">
          <h3>{skill.kind === 'pack' ? '技能扩展包' : '技能详情'}</h3>
          <button className="close-btn" onClick={handleCloseAttempt}>
            <X size={20} />
          </button>
        </div>
        <div className="modal-grid-content">
          <div className="modal-markdown-area">
            {selectedMember ? (
              <>
                <button className="member-back" onClick={() => handleSwitchTarget(undefined)}>
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
                          onClick={() => handleSwitchTarget(member)}
                        >
                          <div className="pack-member-card__header">
                            <Package size={16} className="pack-member-card__icon" />
                            <h4>{member.metadata.name}</h4>
                          </div>
                          <p className="pack-member-card__desc">
                            {member.custom_description ? (
                              <>
                                <span className="custom-badge" style={{ fontSize: '0.6rem', padding: '1px 3px', marginRight: '4px' }}>自定义</span>
                                {member.custom_description}
                              </>
                            ) : (
                              member.metadata.description || '暂无描述信息'
                            )}
                          </p>
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
            
            {!selectedMember && (
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
            )}

            <div className="form-group flex-fill">
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                <label style={{ margin: 0 }}>技能说明</label>
                <span style={{ fontSize: '0.8rem', color: isLengthExceeded ? 'var(--color-danger, #cf222e)' : 'var(--color-muted)' }}>
                  {customDesc.length}/2000
                </span>
              </div>
              <textarea
                placeholder="添加中文技能说明，以快速说明用途并支持搜索（纯文本，限2000字）..."
                value={customDesc}
                onChange={(e) => setCustomDesc(e.target.value)}
                style={{ minHeight: '12rem', resize: 'vertical' }}
              />
              {isLengthExceeded && (
                <span style={{ fontSize: '0.75rem', color: 'var(--color-danger, #cf222e)', marginTop: '4px' }}>
                  技能说明不能超过 2000 个字符
                </span>
              )}
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
              <button className="button button--secondary" onClick={handleCloseAttempt}>
                取消
              </button>
              <button className="button button--primary" onClick={handleSave} disabled={isLengthExceeded}>
                保存更改
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
