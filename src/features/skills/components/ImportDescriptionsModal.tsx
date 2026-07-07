import React, { useState } from 'react';
import { X, ShieldAlert } from 'lucide-react';
import { DescriptionsImportPreview } from '../../../shared/api/types';
import { confirmCustomDescriptionsImport } from '../../../shared/api/tauriClient';

interface Props {
  preview: DescriptionsImportPreview;
  onClose: () => void;
  onSuccess: () => void;
}

export function ImportDescriptionsModal({ preview, onClose, onSuccess }: Props) {
  const [strategy, setStrategy] = useState<'keep_newer' | 'keep_local' | 'keep_import'>('keep_newer');
  const [isSaving, setIsSaving] = useState(false);

  const handleConfirm = async () => {
    setIsSaving(true);
    try {
      await confirmCustomDescriptionsImport(preview.valid_records, strategy);
      onSuccess();
      onClose();
    } catch (err) {
      alert(`导入失败: ${err instanceof Error ? err.message : String(err)}`);
    } finally {
      setIsSaving(false);
    }
  };

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal-body" onClick={(e) => e.stopPropagation()} style={{ maxWidth: '36rem' }}>
        <div className="modal-header">
          <h3>导入技能说明预览</h3>
          <button className="close-btn" onClick={onClose} disabled={isSaving}>
            <X size={20} />
          </button>
        </div>
        
        <div className="modal-grid-content" style={{ display: 'flex', flexDirection: 'column', gap: '1rem', maxHeight: '70vh', overflowY: 'auto' }}>
          <p style={{ fontSize: '0.9rem', color: 'var(--color-ink)' }}>
            文件路径: <code style={{ wordBreak: 'break-all' }}>{preview.file_path}</code>
          </p>

          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: '8px', textAlign: 'center' }}>
            <div style={{ padding: '8px', border: '1px solid var(--color-outline)', borderRadius: '6px', background: 'var(--color-surface-soft)' }}>
              <div style={{ fontSize: '1.2rem', fontWeight: 'bold' }}>{preview.new_count}</div>
              <div style={{ fontSize: '0.75rem', color: 'var(--color-muted)' }}>新增记录</div>
            </div>
            <div style={{ padding: '8px', border: '1px solid var(--color-outline)', borderRadius: '6px', background: 'var(--color-surface-soft)' }}>
              <div style={{ fontSize: '1.2rem', fontWeight: 'bold' }}>{preview.overwrite_count}</div>
              <div style={{ fontSize: '0.75rem', color: 'var(--color-muted)' }}>本地存在冲突</div>
            </div>
            <div style={{ padding: '8px', border: '1px solid var(--color-outline)', borderRadius: '6px', background: 'var(--color-surface-soft)' }}>
              <div style={{ fontSize: '1.2rem', fontWeight: 'bold' }}>{preview.unassociated_count}</div>
              <div style={{ fontSize: '0.75rem', color: 'var(--color-muted)' }}>当前未安装 Skill</div>
            </div>
          </div>

          {preview.invalid_records.length > 0 && (
            <div style={{ padding: '10px', border: '1px solid #ffccc7', background: '#fff2f0', borderRadius: '6px' }}>
              <h4 style={{ color: '#ff4d4f', display: 'flex', alignItems: 'center', gap: '4px', margin: '0 0 6px 0', fontSize: '0.85rem' }}>
                <ShieldAlert size={14} /> 忽略无效记录 ({preview.invalid_records.length})
              </h4>
              <div style={{ maxHeight: '6rem', overflowY: 'auto', fontSize: '0.75rem', color: '#8c8c8c' }}>
                {preview.invalid_records.map((r, i) => (
                  <div key={i} style={{ marginBottom: '4px', borderBottom: '1px dashed #ffa39e', paddingBottom: '4px' }}>
                    <strong>ID: {r.target_id || '未知'}</strong> ({r.reason})
                  </div>
                ))}
              </div>
            </div>
          )}

          <div style={{ borderTop: '1px solid var(--color-outline)', paddingTop: '10px' }}>
            <h4 style={{ margin: '0 0 8px 0', fontSize: '0.9rem' }}>冲突处理策略</h4>
            <div style={{ display: 'flex', flexDirection: 'column', gap: '6px', fontSize: '0.85rem' }}>
              <label style={{ display: 'flex', alignItems: 'center', gap: '6px', cursor: 'pointer' }}>
                <input
                  type="radio"
                  name="strategy"
                  checked={strategy === 'keep_newer'}
                  onChange={() => setStrategy('keep_newer')}
                  disabled={isSaving}
                />
                <span>保留较新记录 (比对本地和文件中的 updated_at 时间) - 推荐</span>
              </label>
              <label style={{ display: 'flex', alignItems: 'center', gap: '6px', cursor: 'pointer' }}>
                <input
                  type="radio"
                  name="strategy"
                  checked={strategy === 'keep_local'}
                  onChange={() => setStrategy('keep_local')}
                  disabled={isSaving}
                />
                <span>保留本地 (跳过冲突记录)</span>
              </label>
              <label style={{ display: 'flex', alignItems: 'center', gap: '6px', cursor: 'pointer' }}>
                <input
                  type="radio"
                  name="strategy"
                  checked={strategy === 'keep_import'}
                  onChange={() => setStrategy('keep_import')}
                  disabled={isSaving}
                />
                <span>使用导入文件 (全部覆盖)</span>
              </label>
            </div>
          </div>
          
          <div className="actions-footer" style={{ borderTop: '1px solid var(--color-outline)', paddingTop: '10px', marginTop: '10px', display: 'flex', justifyContent: 'flex-end', gap: '8px' }}>
            <button className="button button--secondary" onClick={onClose} disabled={isSaving}>
              取消
            </button>
            <button className="button button--primary" onClick={handleConfirm} disabled={isSaving || preview.valid_records.length === 0}>
              {isSaving ? '导入中...' : '确定导入'}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
