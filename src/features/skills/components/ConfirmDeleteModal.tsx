import React, { useState } from 'react';
import { AlertTriangle, ShieldAlert, X } from 'lucide-react';
import type { Skill } from '../../../shared/api/types';

interface Props {
  skill: Skill;
  onClose: () => void;
  onConfirm: (force: boolean) => Promise<void>;
}

export function ConfirmDeleteModal({ skill, onClose, onConfirm }: Props) {
  const [occupiedProjects, setOccupiedProjects] = useState<string[] | null>(null);
  const [isDeleting, setIsDeleting] = useState(false);
  const [errorMessage, setErrorMessage] = useState('');

  const handleDelete = async (force: boolean) => {
    setIsDeleting(true);
    setErrorMessage('');
    try {
      await onConfirm(force);
      onClose();
    } catch (err: any) {
      const msg = err.message || '';
      const details = err.details || '';
      if (msg.includes('enabled in projects') || details.includes('enabled in projects')) {
        const target = msg.includes('enabled in projects') ? msg : details;
        const match = target.match(/enabled in projects: (.*)/);
        const list = match ? match[1].split(', ') : [];
        setOccupiedProjects(list);
      } else {
        setErrorMessage(details || msg || '删除失败，请重试');
      }
    } finally {
      setIsDeleting(false);
    }
  };

  const isOccupied = occupiedProjects !== null;

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal-body compact-modal" onClick={(e) => e.stopPropagation()}>
        <div className="modal-header">
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
            {isOccupied ? (
              <ShieldAlert size={20} style={{ color: 'var(--color-danger)' }} />
            ) : (
              <AlertTriangle size={20} style={{ color: '#ff9800' }} />
            )}
            <h3>{isOccupied ? '该技能正在被项目使用' : '删除技能'}</h3>
          </div>
          <button className="close-btn" onClick={onClose} disabled={isDeleting}>
            <X size={20} />
          </button>
        </div>

        <div style={{ padding: 'var(--space-3)' }}>
          {isOccupied ? (
            <>
              <p style={{ margin: '0 0 var(--space-2) 0', fontSize: '0.9rem', color: 'var(--color-ink)', lineHeight: '1.5' }}>
                该技能包已在以下项目中启用，无法直接删除：
              </p>
              <div className="occupied-projects-list">
                <ul>
                  {occupiedProjects.map((proj) => (
                    <li key={proj} style={{ fontSize: '0.88rem', color: 'var(--color-ink)', marginBottom: '4px' }}>
                      {proj}
                    </li>
                  ))}
                </ul>
              </div>
              <p style={{ margin: 'var(--space-2) 0 0 0', fontSize: '0.85rem', color: 'var(--color-muted)', lineHeight: '1.5' }}>
                如果选择继续，系统将自动从以上项目中移除并禁用此技能，然后彻底删除本地源文件。
              </p>
            </>
          ) : (
            <>
              <p style={{ margin: '0 0 var(--space-2) 0', fontSize: '0.9rem', color: 'var(--color-ink)', lineHeight: '1.5' }}>
                你确定要永久删除技能 <strong style={{ color: 'var(--color-ink)' }}>{skill.metadata.name}</strong> 吗？
              </p>
              <p style={{ margin: '0', fontSize: '0.85rem', color: 'var(--color-muted)' }}>
                此操作将直接从磁盘中删除该技能的所有源文件，且不可逆。
              </p>
            </>
          )}

          {errorMessage && (
            <p style={{ color: 'var(--color-danger)', fontSize: '0.85rem', marginTop: '12px', margin: '12px 0 0 0' }}>
              {errorMessage}
            </p>
          )}
        </div>

        <div className="actions-footer" style={{ padding: 'var(--space-2) var(--space-3) var(--space-3)' }}>
          <button
            type="button"
            className="button button--secondary"
            onClick={onClose}
            disabled={isDeleting}
          >
            取消
          </button>
          {isOccupied ? (
            <button
              type="button"
              className="button button--danger"
              onClick={() => handleDelete(true)}
              disabled={isDeleting}
            >
              {isDeleting ? '正在移除并删除…' : '一键移除并彻底删除'}
            </button>
          ) : (
            <button
              type="button"
              className="button button--danger"
              onClick={() => handleDelete(false)}
              disabled={isDeleting}
            >
              {isDeleting ? '正在删除…' : '确认删除'}
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
