import React, { useState } from 'react';
import { X, Folder, GitBranch } from 'lucide-react';

interface Props {
  onClose: () => void;
  onImport: (source: string, type: 'folder' | 'git') => void;
}

export function ImportSkillModal({ onClose, onImport }: Props) {
  const [mode, setMode] = useState<'folder' | 'git'>('folder');
  const [source, setSource] = useState('');

  const handleImport = (e: React.FormEvent) => {
    e.preventDefault();
    if (source.trim()) {
      onImport(source.trim(), mode);
      onClose();
    }
  };

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal-body compact-modal" onClick={(e) => e.stopPropagation()}>
        <div className="modal-header">
          <h3>导入技能</h3>
          <button className="close-btn" onClick={onClose}>
            <X size={20} />
          </button>
        </div>
        <div className="tabs-header">
          <button
            type="button"
            className={mode === 'folder' ? 'active-tab' : ''}
            onClick={() => setMode('folder')}
          >
            <Folder size={16} />
            <span>文件夹导入</span>
          </button>
          <button
            type="button"
            className={mode === 'git' ? 'active-tab' : ''}
            onClick={() => setMode('git')}
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
                onChange={(e) => setSource(e.target.value)}
                required
              />
            </div>
          ) : (
            <div className="form-group">
              <label>Git Clone 仓库链接</label>
              <input
                placeholder="https://github.com/org/my-skill-repo.git"
                value={source}
                onChange={(e) => setSource(e.target.value)}
                required
              />
            </div>
          )}
          <div className="actions-footer">
            <button type="button" className="button button--secondary" onClick={onClose}>
              取消
            </button>
            <button type="submit" className="button button--primary">
              确认导入
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
