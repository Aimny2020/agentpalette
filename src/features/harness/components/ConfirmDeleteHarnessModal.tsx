import React from 'react';
import { AlertTriangle, X } from 'lucide-react';

interface Props {
  templateName: string;
  onClose: () => void;
  onConfirm: () => Promise<void>;
  isDeleting: boolean;
}

export function ConfirmDeleteHarnessModal({
  templateName,
  onClose,
  onConfirm,
  isDeleting,
}: Props) {
  return (
    <div className="modal-overlay" onClick={onClose} style={{ zIndex: 1100 }}>
      <div className="modal-body compact-modal" onClick={(e) => e.stopPropagation()}>
        <div className="modal-header">
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
            <AlertTriangle size={20} style={{ color: 'var(--color-danger)' }} />
            <h3>删除 Harness 模板</h3>
          </div>
          <button className="close-btn" onClick={onClose} disabled={isDeleting}>
            <X size={20} />
          </button>
        </div>

        <div style={{ padding: 'var(--space-3)' }}>
          <p style={{ margin: '0 0 var(--space-2) 0', fontSize: '0.9rem', color: 'var(--color-ink)', lineHeight: '1.5' }}>
            确定要永久删除 Harness 模板 <strong style={{ color: 'var(--color-ink)' }}>{templateName}</strong> 吗？
          </p>
          <p style={{ margin: '0', fontSize: '0.85rem', color: 'var(--color-danger)', fontWeight: 700 }}>
            警告：此操作会完全移除磁盘上的该模板目录及其中所有的规约文件（如 AGENTS.md 等），此操作无法撤销。
          </p>
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
          <button
            type="button"
            className="button button--danger"
            onClick={onConfirm}
            disabled={isDeleting}
          >
            {isDeleting ? '正在删除…' : '确认删除'}
          </button>
        </div>
      </div>
    </div>
  );
}
