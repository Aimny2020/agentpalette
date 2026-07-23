import React from 'react';
import { AlertTriangle, X } from 'lucide-react';
import { useTranslation } from 'react-i18next';

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
  const { t } = useTranslation();
  return (
    <div className="modal-overlay" onClick={onClose} style={{ zIndex: 1100 }}>
      <div className="modal-body compact-modal" onClick={(e) => e.stopPropagation()}>
        <div className="modal-header">
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
            <AlertTriangle size={20} style={{ color: 'var(--color-danger)' }} />
            <h3>{t('harness.deleteTitle')}</h3>
          </div>
          <button className="close-btn" onClick={onClose} disabled={isDeleting} aria-label={t('common.close')}>
            <X size={20} />
          </button>
        </div>

        <div style={{ padding: 'var(--space-3)' }}>
          <p style={{ margin: '0 0 var(--space-2) 0', fontSize: '0.9rem', color: 'var(--color-ink)', lineHeight: '1.5' }}>
            {t('harness.deletePrompt', { name: templateName })}
          </p>
          <p style={{ margin: '0', fontSize: '0.85rem', color: 'var(--color-danger)', fontWeight: 700 }}>
            {t('harness.deleteWarning')}
          </p>
        </div>

        <div className="actions-footer" style={{ padding: 'var(--space-2) var(--space-3) var(--space-3)' }}>
          <button
            type="button"
            className="button button--secondary"
            onClick={onClose}
            disabled={isDeleting}
          >
            {t('common.cancel')}
          </button>
          <button
            type="button"
            className="button button--danger"
            onClick={onConfirm}
            disabled={isDeleting}
          >
            {isDeleting ? t('harness.deleting') : t('harness.deleteConfirm')}
          </button>
        </div>
      </div>
    </div>
  );
}
