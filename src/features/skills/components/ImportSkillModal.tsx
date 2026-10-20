import React, { useState } from 'react';
import { AlertTriangle, Package, X, Folder, GitBranch } from 'lucide-react';
import { useQuery } from '@tanstack/react-query';
import { AppError, getGitEnvironment, inspectSkillImport } from '../../../shared/api/tauriClient';
import type {
  GitEnvironmentReport,
  GitProxySource,
  ImportInspection,
} from '../../../shared/api/types';
import { useTranslation } from 'react-i18next';

interface Props {
  onClose: () => void;
  onImport: (source: string, type: 'folder' | 'git') => void;
}

const proxySourceKeys: Record<GitProxySource, string> = {
  manual: 'skills.proxySourceManual',
  environment: 'skills.proxySourceEnvironment',
  system: 'skills.proxySourceSystem',
  none: 'skills.proxySourceNone',
  pac: 'skills.proxySourcePac',
};

function GitEnvironmentHint({
  report,
  loading,
}: {
  report?: GitEnvironmentReport;
  loading: boolean;
}) {
  const { t } = useTranslation();
  if (loading) return <p className="muted-copy">{t('skills.gitEnvChecking')}</p>;
  if (!report) return null;
  if (!report.available) {
    return (
      <p className="import-warning">
        <AlertTriangle size={14} />
        {t('skills.gitMissing')}{' '}
        <a href="https://git-scm.com/download/win" target="_blank" rel="noreferrer">
          {t('skills.gitInstall')}
        </a>
      </p>
    );
  }
  if (report.proxy) {
    return (
      <p className="muted-copy">
        {t('skills.gitProxy', {
          proxy: report.proxy,
          source: t(proxySourceKeys[report.proxySource]),
        })}
      </p>
    );
  }
  if (report.proxySource === 'pac') {
    return (
      <p className="import-warning">
        <AlertTriangle size={14} />
        {t('skills.gitPacWarning')}
      </p>
    );
  }
  return <p className="muted-copy">{t('skills.gitDirect')}</p>;
}

export function ImportSkillModal({ onClose, onImport }: Props) {
  const { t } = useTranslation();
  const [mode, setMode] = useState<'folder' | 'git'>('folder');
  const [source, setSource] = useState('');
  const [inspection, setInspection] = useState<ImportInspection | null>(null);
  const [isInspecting, setIsInspecting] = useState(false);
  const [error, setError] = useState('');
  const gitEnvironment = useQuery({
    queryKey: ['gitEnvironment'],
    queryFn: getGitEnvironment,
    enabled: mode === 'git',
  });

  const describeError = (inspectionError: unknown) => {
    if (inspectionError instanceof AppError) {
      return inspectionError.details
        ? `${inspectionError.message}\n${inspectionError.details}`
        : inspectionError.message;
    }
    return inspectionError instanceof Error ? inspectionError.message : t('skills.inspectFailed');
  };

  const handleImport = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!source.trim()) return;
    if (!inspection) {
      setIsInspecting(true);
      setError('');
      try {
        setInspection(await inspectSkillImport(source.trim(), mode));
      } catch (inspectionError) {
        setError(describeError(inspectionError));
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
            <h3>{t('skills.importTitle')}</h3>
            <p className="modal-subtitle">{t('skills.importDescription')}</p>
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
            <span>{t('skills.folderImport')}</span>
          </button>
          <button
            type="button"
            className={mode === 'git' ? 'active-tab' : ''}
            onClick={() => resetInspection('git')}
          >
            <GitBranch size={16} />
            <span>{t('skills.gitImport')}</span>
          </button>
        </div>
        <form onSubmit={handleImport} className="import-form">
          {mode === 'folder' ? (
            <div className="form-group">
              <label>{t('skills.folderPath')}</label>
              <input
                placeholder="/Users/dev/my-skill"
                value={source}
                onChange={(e) => { setSource(e.target.value); resetInspection(); }}
                required
              />
            </div>
          ) : (
            <div className="form-group">
              <label>{t('skills.gitUrl')}</label>
              <input
                placeholder="https://github.com/org/my-skill-repo"
                value={source}
                onChange={(e) => { setSource(e.target.value); resetInspection(); }}
                required
              />
              <GitEnvironmentHint
                report={gitEnvironment.data}
                loading={gitEnvironment.isLoading}
              />
            </div>
          )}
          {error && <p className="import-error" style={{ whiteSpace: 'pre-line' }}>{error}</p>}
          {inspection && (
            <div className="import-inspection">
              <div className="import-inspection__title"><Package size={16} /><strong>{inspection.name}</strong></div>
              {inspection.install_id && <p>{t('skills.installId', { id: inspection.install_id })}</p>}
              {inspection.normalized_source && <p>{t('skills.gitSource', { source: inspection.normalized_source })}</p>}
              <p>{inspection.kind === 'pack' ? t('skills.pack', { count: inspection.member_count }) : t('skills.standalone')}</p>
              {inspection.recommended_ref && <p>{t('skills.recommendedVersion', { version: inspection.recommended_ref })}</p>}
              {inspection.has_executable_content && <p className="import-warning"><AlertTriangle size={14} />{t('skills.executableWarning')}</p>}
              {inspection.duplicate_skill_id && <p className="import-warning">{t('skills.duplicate', { id: inspection.duplicate_skill_id })}</p>}
            </div>
          )}
          <div className="actions-footer">
            <button type="button" className="button button--secondary" onClick={onClose}>
              {t('common.cancel')}
            </button>
            <button type="submit" className="button button--primary" disabled={isInspecting || Boolean(inspection?.duplicate_skill_id)}>
              {isInspecting ? t('skills.inspecting') : inspection ? t('skills.confirmImport') : t('skills.inspect')}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
