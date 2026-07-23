import React, { useState } from 'react';
import { X, Search, FileText, CheckCircle, AlertCircle } from 'lucide-react';
import { useQuery } from '@tanstack/react-query';
import { useTranslation } from 'react-i18next';
import { getProjects, inspectHarnessImport } from '../../../shared/api/tauriClient';
import { useProjectStore } from '../../../shared/store/projectStore';
import { HarnessImportOptions, HarnessExtractOptions } from '../../../shared/api/types';

interface ImportHarnessModalProps {
  onClose: () => void;
  onImportFolder: (path: string, options: HarnessImportOptions) => void;
  onExtractProject: (projectId: string, options: HarnessExtractOptions) => void;
}

const EXTRACTABLE_FILES = ['AGENTS.md', 'docs/architecture.md', 'docs/feature_list.json', 'docs/task-status.md', 'docs/verification.md', 'docs/risk-rules.md', 'docs/agent-profile.md', 'docs/harness.toml'];

export function ImportHarnessModal({ onClose, onImportFolder, onExtractProject }: ImportHarnessModalProps) {
  const { t } = useTranslation();
  const { activeProjectId } = useProjectStore();
  const [tab, setTab] = useState<'folder' | 'extract'>('folder');

  // Folder Import states
  const [folderPath, setFolderPath] = useState('');
  const [inspecting, setInspecting] = useState(false);
  const [inspectionResult, setInspectionResult] = useState<any>(null);
  const [importName, setImportName] = useState('');
  const [importDesc, setImportDesc] = useState('');
  const [importWorkType, setImportWorkType] = useState('code');
  const [importLanguage, setImportLanguage] = useState<'zh-CN' | 'en'>('zh-CN');

  // Extract states
  const [selectedProjectId, setSelectedProjectId] = useState(activeProjectId || '');
  const [selectedFiles, setSelectedFiles] = useState<string[]>(['AGENTS.md', 'docs/harness.toml']);
  const [extractName, setExtractName] = useState('');
  const [extractDesc, setExtractDesc] = useState('');
  const [extractWorkType, setExtractWorkType] = useState('code');
  const [extractLanguage, setExtractLanguage] = useState<'zh-CN' | 'en'>('zh-CN');

  // Query projects for extraction dropdown
  const { data: projects = [] } = useQuery({
    queryKey: ['projects'],
    queryFn: getProjects,
  });

  const handleInspect = async () => {
    if (!folderPath.trim()) {
      alert(t('harness.enterFolderPath'));
      return;
    }
    setInspecting(true);
    setInspectionResult(null);
    try {
      const res = await inspectHarnessImport(folderPath.trim());
      setInspectionResult(res);
      // Auto fill form based on inspection metadata
      setImportName(res.name || '');
      setImportDesc(res.description || '');
      const discoveredWorkType = res.workType ?? '';
      setImportWorkType(['code', 'document', 'presentation', 'custom'].includes(discoveredWorkType) ? discoveredWorkType : 'custom');
    } catch (err: any) {
      alert(t('harness.inspectFailed', { error: err.message || String(err) }));
    } finally {
      setInspecting(false);
    }
  };

  const handleFolderImportSubmit = () => {
    if (!importName.trim()) {
      alert(t('harness.enterDisplayName'));
      return;
    }
    // NOTE: Module selectors are not presented in the import modal in this iteration
    // as legacy templates or imports without modules are allowed to retain an empty list.
    onImportFolder(folderPath.trim(), {
      name: importName.trim(),
      description: importDesc.trim(),
      workType: importWorkType,
      language: importLanguage,
    });
  };

  const handleToggleExtractFile = (path: string) => {
    setSelectedFiles((prev) =>
      prev.includes(path) ? prev.filter((p) => p !== path) : [...prev, path]
    );
  };

  const handleExtractSubmit = () => {
    if (!selectedProjectId) {
      alert(t('harness.selectSourceProject'));
      return;
    }
    if (!extractName.trim()) {
      alert(t('harness.enterDisplayName'));
      return;
    }
    onExtractProject(selectedProjectId, {
      name: extractName.trim(),
      description: extractDesc.trim(),
      workType: extractWorkType,
      language: extractLanguage,
      selectedFiles,
    });
  };

  return (
    <div className="modal-overlay" onClick={onClose} style={{ zIndex: 1000 }}>
      <div className="modal-body" onClick={(e) => e.stopPropagation()} style={{ width: '38rem', maxWidth: '90vw', height: 'auto' }}>
        <div className="modal-header">
          <h3>{t('harness.importTitle')}</h3>
          <button type="button" className="close-btn" onClick={onClose} aria-label={t('common.close')}>
            <X size={20} />
          </button>
        </div>

        <div className="harness-import-tabs">
          <div
            className="harness-import-tab"
            data-active={tab === 'folder'}
            onClick={() => setTab('folder')}
          >
            {t('harness.folderImport')}
          </div>
          <div
            className="harness-import-tab"
            data-active={tab === 'extract'}
            onClick={() => setTab('extract')}
          >
            {t('harness.projectExtract')}
          </div>
        </div>

        {tab === 'folder' ? (
          /* Tab 1: Local Folder Import */
          <div className="harness-modal-content" style={{ display: 'flex', flexDirection: 'column', gap: 'var(--space-2)' }}>
            <div className="harness-form-group">
              <label htmlFor="import-path">{t('harness.folderPath')}</label>
              <div style={{ display: 'flex', gap: 'var(--space-1)' }}>
                <input
                  id="import-path"
                  style={{ flex: 1 }}
                  placeholder={t('harness.folderPathPlaceholder')}
                  value={folderPath}
                  onChange={(e) => setFolderPath(e.target.value)}
                />
                <button
                  type="button"
                  className="button button--secondary"
                  onClick={handleInspect}
                  disabled={inspecting}
                >
                  <Search size={16} style={{ marginRight: '0.25rem' }} /> {inspecting ? t('harness.inspecting') : t('harness.inspect')}
                </button>
              </div>
            </div>

            {inspectionResult && (
              <div className="harness-import-inspection-panel">
                <h5>🔍 {t('harness.inspection')}</h5>
                <div style={{ display: 'flex', flexDirection: 'column', gap: '0.4rem', fontSize: '0.85rem', marginBottom: 'var(--space-2)' }}>
                  <div style={{ display: 'flex', alignItems: 'center', gap: '0.35rem' }}>
                    {inspectionResult.hasAgentsMd ? (
                      <CheckCircle size={14} color="var(--color-success-ink)" />
                    ) : (
                      <AlertCircle size={14} color="var(--color-danger)" />
                    )}
                    <span>{t('harness.containsAgents', { value: inspectionResult.hasAgentsMd ? t('harness.yes') : t('harness.noAgents') })}</span>
                  </div>
                  <div style={{ display: 'flex', alignItems: 'center', gap: '0.35rem' }}>
                    {inspectionResult.hasManifest ? (
                      <CheckCircle size={14} color="var(--color-success-ink)" />
                    ) : (
                      <AlertCircle size={14} color="var(--color-muted)" />
                    )}
                    <span>{t('harness.containsManifest', { value: inspectionResult.hasManifest ? t('harness.yes') : t('harness.noManifest') })}</span>
                  </div>
                  <div style={{ color: 'var(--color-muted)', fontSize: '0.8rem' }}>
                    {t('harness.foundFiles', { count: inspectionResult.foundFiles.length })}
                  </div>
                </div>

                <div style={{ display: 'flex', flexDirection: 'column', gap: 'var(--space-2)', borderTop: '1px solid var(--color-outline)', paddingTop: 'var(--space-2)' }}>
                  <div className="harness-form-group">
                    <label htmlFor="imp-name">{t('harness.displayName')}</label>
                    <input
                      id="imp-name"
                      value={importName}
                      onChange={(e) => setImportName(e.target.value)}
                      placeholder={t('harness.namePlaceholder')}
                    />
                  </div>
                  <div className="harness-form-group">
                    <label htmlFor="imp-desc">{t('harness.descriptionLabel')}</label>
                    <textarea
                      id="imp-desc"
                      value={importDesc}
                      onChange={(e) => setImportDesc(e.target.value)}
                      placeholder={t('harness.descriptionPlaceholder')}
                      rows={2}
                    />
                  </div>
                  <div className="harness-form-group">
                    <label htmlFor="imp-worktype">{t('harness.workTypeLabel')}</label>
                    <select
                      id="imp-worktype"
                      className="harness-filter-select"
                      style={{ borderRadius: 'var(--radius-sm)', width: '100%' }}
                      value={importWorkType}
                      onChange={(e) => setImportWorkType(e.target.value)}
                    >
                      <option value="code">{t('harness.code')}</option>
                      <option value="document">{t('harness.document')}</option>
                      <option value="presentation">{t('harness.presentation')}</option>
                      <option value="custom">{t('harness.custom')}</option>
                    </select>
                  </div>
                  <div className="harness-form-group">
                    <label htmlFor="imp-language">{t('harness.language')}</label>
                    <select id="imp-language" value={importLanguage} onChange={(e) => setImportLanguage(e.target.value as 'zh-CN' | 'en')}>
                      <option value="zh-CN">{t('harness.chinese')}</option><option value="en">English</option>
                    </select>
                  </div>

                  <div style={{ display: 'flex', justifyContent: 'flex-end', marginTop: 'var(--space-1)' }}>
                    <button
                      type="button"
                      className="button button--primary"
                      onClick={handleFolderImportSubmit}
                    >
                      {t('harness.importConfirm')}
                    </button>
                  </div>
                </div>
              </div>
            )}
          </div>
        ) : (
          /* Tab 2: Extract from Project */
          <div className="harness-modal-content" style={{ display: 'flex', flexDirection: 'column', gap: 'var(--space-2)' }}>
            <div className="harness-form-group">
              <label htmlFor="extract-project-select">{t('harness.selectProject')}</label>
              <select
                id="extract-project-select"
                className="harness-filter-select"
                style={{ borderRadius: 'var(--radius-sm)', width: '100%' }}
                value={selectedProjectId}
                onChange={(e) => setSelectedProjectId(e.target.value)}
              >
                <option value="">{t('harness.selectProjectPlaceholder')}</option>
                {projects.map((p) => (
                  <option key={p.id} value={p.id}>
                    {p.name} ({p.path})
                  </option>
                ))}
              </select>
            </div>

            <div className="harness-form-group">
              <label>{t('harness.selectFiles')}</label>
              <div className="harness-checklist" style={{ maxHeight: '10rem' }}>
                {EXTRACTABLE_FILES.map((path) => (
                  <div
                    key={path}
                    className="harness-checklist-item"
                    onClick={() => handleToggleExtractFile(path)}
                  >
                    <input
                      type="checkbox"
                      checked={selectedFiles.includes(path)}
                      onChange={() => {}}
                    />
                    <span>{path}</span>
                  </div>
                ))}
              </div>
            </div>

            <div style={{ display: 'flex', flexDirection: 'column', gap: 'var(--space-2)', borderTop: '1px solid var(--color-outline)', paddingTop: 'var(--space-2)' }}>
              <div className="harness-form-group">
                <label htmlFor="ext-name">{t('harness.displayName')}</label>
                <input
                  id="ext-name"
                  value={extractName}
                  onChange={(e) => setExtractName(e.target.value)}
                  placeholder={t('harness.namePlaceholder')}
                />
              </div>
              <div className="harness-form-group">
                <label htmlFor="ext-desc">{t('harness.descriptionLabel')}</label>
                <textarea
                  id="ext-desc"
                  value={extractDesc}
                  onChange={(e) => setExtractDesc(e.target.value)}
                  placeholder={t('harness.descriptionPlaceholder')}
                  rows={2}
                />
              </div>
              <div className="harness-form-group">
                <label htmlFor="ext-worktype">{t('harness.workTypeLabel')}</label>
                <select
                  id="ext-worktype"
                  className="harness-filter-select"
                  style={{ borderRadius: 'var(--radius-sm)', width: '100%' }}
                  value={extractWorkType}
                  onChange={(e) => setExtractWorkType(e.target.value)}
                >
                  <option value="code">{t('harness.code')}</option>
                  <option value="document">{t('harness.document')}</option>
                  <option value="presentation">{t('harness.presentation')}</option>
                  <option value="custom">{t('harness.custom')}</option>
                </select>
              </div>
              <div className="harness-form-group">
                <label htmlFor="ext-language">{t('harness.language')}</label>
                <select id="ext-language" value={extractLanguage} onChange={(e) => setExtractLanguage(e.target.value as 'zh-CN' | 'en')}>
                  <option value="zh-CN">{t('harness.chinese')}</option><option value="en">English</option>
                </select>
              </div>

              <div style={{ display: 'flex', justifyContent: 'flex-end', marginTop: 'var(--space-1)' }}>
                <button
                  type="button"
                  className="button button--primary"
                  onClick={handleExtractSubmit}
                >
                  {t('harness.extractConfirm')}
                </button>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
