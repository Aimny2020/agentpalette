import React, { useState } from 'react';
import { AlertTriangle, ShieldAlert, X } from 'lucide-react';
import type { Skill } from '../../../shared/api/types';
import { useTranslation } from 'react-i18next';

interface Props {
  skill: Skill;
  onClose: () => void;
  onConfirm: (force: boolean) => Promise<void>;
}

export function ConfirmDeleteModal({ skill, onClose, onConfirm }: Props) {
  const { t } = useTranslation();
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
        setErrorMessage(details || msg || t('skills.deleteFailed'));
      }
    } finally {
      setIsDeleting(false);
    }
  };

  const isOccupied = occupiedProjects !== null;

  return (
    <div className="modal-overlay" onClick={isDeleting ? undefined : onClose}>
      <div
        className="modal-body compact-modal confirm-delete-modal"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="modal-header">
          <div className="confirm-delete-heading">
            <span
              className={`confirm-delete-heading__icon ${isOccupied ? 'is-danger' : 'is-warning'}`}
            >
              {isOccupied ? <ShieldAlert size={18} /> : <AlertTriangle size={18} />}
            </span>
            <h3>{isOccupied ? t('skills.occupiedTitle') : t('skills.deleteTitle')}</h3>
          </div>
          <button className="close-btn" onClick={onClose} disabled={isDeleting}>
            <X size={20} />
          </button>
        </div>

        <div className="confirm-delete-body">
          {isOccupied ? (
            <>
              <p className="confirm-delete-copy">
                {t('skills.occupiedIntro', { name: skill.metadata.name })}
              </p>
              <div className="occupied-projects-list">
                <ul>
                  {occupiedProjects.map((project) => (
                    <li key={project}>{project}</li>
                  ))}
                </ul>
              </div>
              <p className="confirm-delete-detail">{t('skills.occupiedDetail')}</p>
            </>
          ) : (
            <>
              <p className="confirm-delete-copy">
                {t('skills.deletePrompt', { name: skill.metadata.name })}
              </p>
              <p className="confirm-delete-detail">{t('skills.deleteWarning')}</p>
            </>
          )}

          {errorMessage && (
            <p className="confirm-delete-error" role="alert">{errorMessage}</p>
          )}
        </div>

        <div className="actions-footer confirm-delete-footer">
          <button
            type="button"
            className="button button--secondary"
            onClick={onClose}
            disabled={isDeleting}
          >
            {t('common.cancel')}
          </button>
          {isOccupied ? (
            <button
              type="button"
              className="button button--danger"
              onClick={() => handleDelete(true)}
              disabled={isDeleting}
            >
              {isDeleting ? t('skills.removingAndDeleting') : t('skills.removeAndDelete')}
            </button>
          ) : (
            <button
              type="button"
              className="button button--danger"
              onClick={() => handleDelete(false)}
              disabled={isDeleting}
            >
              {isDeleting ? t('skills.deleting') : t('skills.confirmDelete')}
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
