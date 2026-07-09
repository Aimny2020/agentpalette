import React, { useState } from 'react';
import { AlertTriangle, Package, X, Folder, GitBranch } from 'lucide-react';
import { inspectSkillImport } from '../../../shared/api/tauriClient';
import type { ImportInspection } from '../../../shared/api/types';

interface Props {
  onClose: () => void;
  onImport: (source: string, type: 'folder' | 'git') => void;
}

export function ImportSkillModal({ onClose, onImport }: Props) {
  const [mode, setMode] = useState<'folder' | 'git'>('folder');
  const [source, setSource] = useState('');
  const [inspection, setInspection] = useState<ImportInspection | null>(null);
  const [isInspecting, setIsInspecting] = useState(false);
  const [error, setError] = useState('');

  const handleImport = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!source.trim()) return;
    if (!inspection) {
      setIsInspecting(true);
      setError('');
      try {
        setInspection(await inspectSkillImport(source.trim(), mode));
      } catch (inspectionError) {
        setError(inspectionError instanceof Error ? inspectionError.message : '无法检查导入内容');
      } finally {
        setIsInspecting(false);
      }
      return;
    }
    if (!inspection.duplicate_skill_id) {
      onImport(source.trim(), mode);
      onClose();
    }
  };

  const resetInspection = (nextMode?: 'folder' | 'git') => {
    if (nextMode) setMode(nextMode);
    setInspection(null);
    setError('');
  };

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal-body compact-modal" onClick={(e) => e.stopPropagation()}>
        <div className="modal-header">
          <div>
            <h3>导入技能</h3>
            <p className="modal-subtitle">支持独立 Skill 与包含多个子 Skill 的扩展包</p>
          </div>
          <button className="close-btn" onClick={onClose}>
            <X size={20} />
          </button>
        </div>
        <div className="tabs-header">
          <button
            type="button"
            className={mode === 'folder' ? 'active-tab' : ''}
            onClick={() => resetInspection('folder')}
          >
            <Folder size={16} />
            <span>文件夹导入</span>
          </button>
          <button
            type="button"
            className={mode === 'git' ? 'active-tab' : ''}
            onClick={() => resetInspection('git')}
          >
            <GitBranch size={16} />
            <span>Git 仓库导入</span>
          </button>
        </div>
        <form onSubmit={handleImport} className="import-form">
          {mode === 'folder' ? (
            <div className="form-group">
              <label>本地文件夹路径</label>
              <input
                placeholder="/Users/dev/my-skill"
                value={source}
                onChange={(e) => { setSource(e.target.value); resetInspection(); }}
                required
              />
            </div>
          ) : (
            <div className="form-group">
              <label>Git Clone 仓库链接</label>
              <input
                placeholder="https://github.com/org/my-skill-repo.git"
                value={source}
                onChange={(e) => { setSource(e.target.value); resetInspection(); }}
                required
              />
            </div>
          )}
          {error && <p className="import-error">{error}</p>}
          {inspection && (
            <div className="import-inspection">
              <div className="import-inspection__title"><Package size={16} /><strong>{inspection.name}</strong></div>
              {inspection.install_id && <p>安装 ID：{inspection.install_id}</p>}
              {inspection.normalized_source && <p>Git 来源：{inspection.normalized_source}</p>}
              <p>{inspection.kind === 'pack' ? `技能扩展包 · ${inspection.member_count} 个 Skills` : '独立 Skill'}</p>
              {inspection.recommended_ref && <p>推荐稳定版本：{inspection.recommended_ref}</p>}
              {inspection.has_executable_content && <p className="import-warning"><AlertTriangle size={14} />包含脚本或可执行内容，启用前需要信任。</p>}
              {inspection.duplicate_skill_id && <p className="import-warning">已安装为 {inspection.duplicate_skill_id}，不会创建重复副本。</p>}
            </div>
          )}
          <div className="actions-footer">
            <button type="button" className="button button--secondary" onClick={onClose}>
              取消
            </button>
            <button type="submit" className="button button--primary" disabled={isInspecting || Boolean(inspection?.duplicate_skill_id)}>
              {isInspecting ? '正在检查…' : inspection ? '确认导入' : '检查内容'}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
