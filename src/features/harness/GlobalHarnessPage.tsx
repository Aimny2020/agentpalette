import React, { useState, useEffect, useRef } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { Plus, Download, ArrowLeft, Save, Copy, Trash, File, FolderOpen, RefreshCw, CheckCircle, AlertTriangle, AlertCircle } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import {
  getHarnessTemplates,
  getHarnessTemplate,
  createHarnessTemplate,
  readHarnessFile,
  writeHarnessFile,
  createHarnessFile,
  deleteHarnessFile,
  deleteHarnessTemplate,
  duplicateHarnessTemplate,
  inspectHarnessImport,
  importHarnessFromFolder,
  extractHarnessFromProject,
  getHarnessPresets,
  getCodeWorkModules,
  getCodeWorkSharedFiles,
} from '../../shared/api/tauriClient';
import { CreateHarnessModal } from './components/CreateHarnessModal';
import { ImportHarnessModal } from './components/ImportHarnessModal';
import { ConfirmDeleteHarnessModal } from './components/ConfirmDeleteHarnessModal';
import { HarnessTemplateSummary, HarnessFileSummary, HarnessTemplateDetail } from '../../shared/api/types';
import './harness.css';

export function GlobalHarnessPage() {
  const { t, i18n } = useTranslation();
  const queryClient = useQueryClient();
  const [search, setSearch] = useState('');
  const [filterWorkType, setFilterWorkType] = useState<string>('all');

  // Modal toggle states
  const [isCreateOpen, setIsCreateOpen] = useState(false);
  const [isImportOpen, setIsImportOpen] = useState(false);
  const [isDeleteModalOpen, setIsDeleteModalOpen] = useState(false);

  // Active template detail states
  const [selectedTemplateId, setSelectedTemplateId] = useState<string | null>(null);
  const [activeFilePath, setActiveFilePath] = useState<string | null>(null);
  const [editorContent, setEditorContent] = useState('');
  const [isDirty, setIsDirty] = useState(false);

  // File tree operations states
  const [isCreatingFile, setIsCreatingFile] = useState(false);
  const [newFilePathInput, setNewFilePathInput] = useState('');
  const [deletingFilePath, setDeletingFilePath] = useState<string | null>(null);

  // Queries
  const { data: summaries = [], isLoading: summariesLoading, refetch: refetchSummaries } = useQuery({
    queryKey: ['harness-summaries'],
    queryFn: getHarnessTemplates,
  });

  const { data: presets = [], isLoading: presetsLoading } = useQuery({
    queryKey: ['harness-presets'],
    queryFn: getHarnessPresets,
  });

  const { data: codeModules = [], isLoading: codeModulesLoading } = useQuery({
    queryKey: ['code-work-modules'],
    queryFn: getCodeWorkModules,
  });

  const { data: codeSharedFiles = [], isLoading: codeSharedFilesLoading } = useQuery({
    queryKey: ['code-work-shared-files'],
    queryFn: getCodeWorkSharedFiles,
  });

  const { data: detail, isLoading: detailLoading } = useQuery({
    queryKey: ['harness-detail', selectedTemplateId],
    queryFn: () => selectedTemplateId ? getHarnessTemplate(selectedTemplateId) : null,
    enabled: !!selectedTemplateId,
  });

  // Load active file content
  const { data: fileData, isFetching: fileLoading } = useQuery({
    queryKey: ['harness-file', selectedTemplateId, activeFilePath],
    queryFn: () => (selectedTemplateId && activeFilePath) ? readHarnessFile(selectedTemplateId, activeFilePath) : null,
    enabled: !!selectedTemplateId && !!activeFilePath,
  });

  // Sync editor content when file data is loaded
  useEffect(() => {
    if (fileData) {
      setEditorContent(fileData.content);
      setIsDirty(false);
    }
  }, [fileData]);

  // Mutations
  const createMut = useMutation({
    mutationFn: createHarnessTemplate,
    onSuccess: (newDetail) => {
      queryClient.invalidateQueries({ queryKey: ['harness-summaries'] });
      setSelectedTemplateId(newDetail.id);
      setActiveFilePath('AGENTS.md');
      setIsCreateOpen(false);
    },
    onError: (err: any) => {
      alert(t('harness.createFailed', { error: err.message || String(err) }));
    },
  });

  const saveFileMut = useMutation({
    mutationFn: ({ templateId, path, content }: { templateId: string; path: string; content: string }) =>
      writeHarnessFile(templateId, path, content),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['harness-detail', selectedTemplateId] });
      queryClient.invalidateQueries({ queryKey: ['harness-summaries'] });
      setIsDirty(false);
      alert(t('harness.saveSucceeded'));
    },
    onError: (err: any) => {
      alert(t('harness.saveFailed', { error: err.message || String(err) }));
    },
  });

  const createFileMut = useMutation({
    mutationFn: ({ templateId, path, kind }: { templateId: string; path: string; kind: string }) =>
      createHarnessFile(templateId, path, kind),
    onSuccess: (newFile) => {
      queryClient.invalidateQueries({ queryKey: ['harness-detail', selectedTemplateId] });
      queryClient.invalidateQueries({ queryKey: ['harness-summaries'] });
      setActiveFilePath(newFile.path);
    },
    onError: (err: any) => {
      alert(t('harness.createFileFailed', { error: err.message || String(err) }));
    },
  });

  const deleteFileMut = useMutation({
    mutationFn: ({ templateId, path }: { templateId: string; path: string }) =>
      deleteHarnessFile(templateId, path),
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({ queryKey: ['harness-detail', selectedTemplateId] });
      queryClient.invalidateQueries({ queryKey: ['harness-summaries'] });
      if (activeFilePath === variables.path) {
        setActiveFilePath(null);
        setEditorContent('');
      }
    },
    onError: (err: any) => {
      alert(t('harness.deleteFileFailed', { error: err.message || String(err) }));
    },
  });

  const deleteTemplateMut = useMutation({
    mutationFn: deleteHarnessTemplate,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['harness-summaries'] });
      setSelectedTemplateId(null);
      setActiveFilePath(null);
      setEditorContent('');
    },
    onError: (err: any) => {
      alert(t('harness.deleteFailed', { error: err.message || String(err) }));
    },
  });

  const duplicateMut = useMutation({
    mutationFn: ({ templateId, targetName }: { templateId: string; targetName: string }) =>
      duplicateHarnessTemplate(templateId, targetName),
    onSuccess: (newDetail) => {
      queryClient.invalidateQueries({ queryKey: ['harness-summaries'] });
      setSelectedTemplateId(newDetail.id);
      setActiveFilePath('AGENTS.md');
    },
    onError: (err: any) => {
      alert(t('harness.duplicateFailed', { error: err.message || String(err) }));
    },
  });

  const importFolderMut = useMutation({
    mutationFn: ({ path, options }: { path: string; options: any }) =>
      importHarnessFromFolder(path, options),
    onSuccess: (newDetail) => {
      queryClient.invalidateQueries({ queryKey: ['harness-summaries'] });
      setSelectedTemplateId(newDetail.id);
      setActiveFilePath('AGENTS.md');
      setIsImportOpen(false);
    },
    onError: (err: any) => {
      alert(t('harness.importFailed', { error: err.message || String(err) }));
    },
  });

  const extractProjectMut = useMutation({
    mutationFn: ({ projectId, options }: { projectId: string; options: any }) =>
      extractHarnessFromProject(projectId, options),
    onSuccess: (newDetail) => {
      queryClient.invalidateQueries({ queryKey: ['harness-summaries'] });
      setSelectedTemplateId(newDetail.id);
      setActiveFilePath('AGENTS.md');
      setIsImportOpen(false);
    },
    onError: (err: any) => {
      alert(t('harness.extractFailed', { error: err.message || String(err) }));
    },
  });

  // Ctrl+S keybind for saving
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key === 's') {
        e.preventDefault();
        if (selectedTemplateId && activeFilePath && isDirty) {
          saveFileMut.mutate({
            templateId: selectedTemplateId,
            path: activeFilePath,
            content: editorContent,
          });
        }
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [selectedTemplateId, activeFilePath, editorContent, isDirty]);

  const handleSaveFile = () => {
    if (selectedTemplateId && activeFilePath) {
      saveFileMut.mutate({
        templateId: selectedTemplateId,
        path: activeFilePath,
        content: editorContent,
      });
    }
  };

  const handleCreateFile = () => {
    if (!selectedTemplateId) return;
    setIsCreatingFile(true);
    setNewFilePathInput('');
  };

  const handleDeleteTemplate = () => {
    if (!selectedTemplateId) return;
    setIsDeleteModalOpen(true);
  };

  const handleDuplicateTemplate = () => {
    if (!selectedTemplateId) return;
    const targetName = prompt(t('harness.duplicatePrompt'));
    if (!targetName) return;

    duplicateMut.mutate({
      templateId: selectedTemplateId,
      targetName,
    });
  };

  // Filter templates list
  const filteredSummaries = summaries.filter((s) => {
    const matchesSearch =
      s.name.toLowerCase().includes(search.toLowerCase()) ||
      s.id.toLowerCase().includes(search.toLowerCase()) ||
      s.description.toLowerCase().includes(search.toLowerCase());
    const matchesWorkType = filterWorkType === 'all' || s.workType === filterWorkType;
    return matchesSearch && matchesWorkType;
  });

  const getWorkTypeLabel = (wt: string) => {
    switch (wt) {
      case 'code': return t('harness.code');
      case 'document': return t('harness.document');
      case 'presentation': return t('harness.presentation');
      default: return t('harness.custom');
    }
  };

  if (summariesLoading) {
    return (
      <div className="page-state">
        <div className="loading-dot" />
        <p>{t('harness.loading')}</p>
      </div>
    );
  }

  return (
    <div className="harness-container">
      {selectedTemplateId && detail ? (
        /* Detail Page / Split Editor View */
        <div className="page-stack harness-editor-container">
          <header className="page-header" style={{ minHeight: 'auto', alignItems: 'center', marginBottom: 'var(--space-1)' }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: 'var(--space-2)' }}>
              <button
                type="button"
                className="harness-tree__btn"
                onClick={() => setSelectedTemplateId(null)}
                style={{ padding: '0.45rem', borderRadius: '50%', background: 'var(--color-surface-soft)' }}
              >
                <ArrowLeft size={18} />
              </button>
              <div>
                <h2 style={{ fontSize: '1.45rem', fontWeight: 700, margin: 0 }}>
                  {detail.name}
                  <span className={`harness-badge harness-badge--${detail.workType}`} style={{ marginLeft: 'var(--space-1)', fontSize: '0.75rem' }}>
                    {getWorkTypeLabel(detail.workType)}
                  </span>
                </h2>
                <small style={{ color: 'var(--color-muted)' }}>ID: {detail.id}</small>
              </div>
            </div>
            <div style={{ display: 'flex', gap: 'var(--space-1)' }}>
              <button type="button" className="button button--secondary" onClick={handleDuplicateTemplate}>
                <Copy size={16} style={{ marginRight: '0.35rem' }} /> {t('harness.duplicate')}
              </button>
              <button type="button" className="button button--secondary" onClick={handleDeleteTemplate} style={{ color: 'var(--color-danger)' }}>
                <Trash size={16} style={{ marginRight: '0.35rem' }} /> {t('harness.deleteTemplate')}
              </button>
            </div>
          </header>

          <div className="harness-editor-layout">
            {/* Column 1: File Tree (Left) */}
            <div className="harness-editor__tree">
              <div className="harness-tree__header">
                <h4>📁 {t('harness.templateFiles')}</h4>
                <div className="harness-tree__actions">
                  <button type="button" className="harness-tree__btn" title={t('harness.newFile')} aria-label={t('harness.newFile')} onClick={handleCreateFile}>
                    <Plus size={16} />
                  </button>
                </div>
              </div>
              <div className="harness-tree__list">
                {detail.files.map((file) => {
                  const isActive = activeFilePath === file.path;
                  const isDeletingConfirm = deletingFilePath === file.path;
                  return (
                    <div
                      key={file.path}
                      className="harness-tree__item"
                      data-active={isActive}
                      data-deleting={isDeletingConfirm}
                      onClick={() => {
                        if (isDeletingConfirm) return;
                        if (isDirty) {
                          if (!confirm(t('harness.discardChanges'))) {
                            return;
                          }
                        }
                        setActiveFilePath(file.path);
                      }}
                    >
                      <div className="harness-tree__item-left">
                        <File size={14} color={file.isStandard ? 'var(--color-primary-strong)' : 'var(--color-muted)'} />
                        <span className="harness-tree__item-name">{file.path}</span>
                      </div>
                      {file.path !== 'AGENTS.md' && file.path !== 'docs/harness.toml' && (
                        isDeletingConfirm ? (
                          <div className="harness-tree__confirm-actions" onClick={(e) => e.stopPropagation()}>
                            <button
                              type="button"
                              className="harness-tree__confirm-btn harness-tree__confirm-btn--yes"
                              title={t('harness.confirmDelete')}
                              onClick={() => {
                                deleteFileMut.mutate({ templateId: selectedTemplateId!, path: file.path });
                                setDeletingFilePath(null);
                              }}
                            >
                              {t('common.delete')}
                            </button>
                            <button
                              type="button"
                              className="harness-tree__confirm-btn harness-tree__confirm-btn--no"
                              title={t('common.cancel')}
                              onClick={() => setDeletingFilePath(null)}
                            >
                              {t('common.cancel')}
                            </button>
                          </div>
                        ) : (
                          <button
                            type="button"
                            className="harness-tree__item-delete"
                            onClick={(e) => {
                              e.stopPropagation();
                              setDeletingFilePath(file.path);
                            }}
                            title={t('harness.deleteFile')}
                          >
                            <Trash size={12} />
                          </button>
                        )
                      )}
                    </div>
                  );
                })}

                {isCreatingFile && (
                  <div className="harness-tree__item harness-tree__item--creating">
                    <div className="harness-tree__item-left" style={{ width: '100%' }}>
                      <File size={14} color="var(--color-primary-strong)" />
                      <input
                        type="text"
                        className="harness-tree__input"
                        value={newFilePathInput}
                        onChange={(e) => setNewFilePathInput(e.target.value)}
                        placeholder={t('harness.filePath')}
                        autoFocus
                        onBlur={() => {
                          setTimeout(() => {
                            setIsCreatingFile(false);
                            setNewFilePathInput('');
                          }, 150);
                        }}
                        onKeyDown={(e) => {
                          if (e.key === 'Enter') {
                            const relPath = newFilePathInput.trim();
                            if (!relPath) {
                              setIsCreatingFile(false);
                              return;
                            }
                            if (relPath.startsWith('/') || relPath.includes('..')) {
                              alert(t('harness.invalidPath'));
                              return;
                            }
                            const ext = relPath.split('.').pop() || '';
                            createFileMut.mutate({
                              templateId: selectedTemplateId!,
                              path: relPath,
                              kind: ext,
                            });
                            setIsCreatingFile(false);
                            setNewFilePathInput('');
                          } else if (e.key === 'Escape') {
                            setIsCreatingFile(false);
                            setNewFilePathInput('');
                          }
                        }}
                      />
                    </div>
                  </div>
                )}
              </div>
            </div>

            {/* Column 2: Code Editor (Center) */}
            <div className="harness-editor__content">
              {activeFilePath ? (
                <>
                  <div className="harness-editor__toolbar">
                    <div className="harness-editor__title-bar">
                      <span className="harness-editor__filename">{activeFilePath}</span>
                      {isDirty && (
                        <span style={{ fontSize: '0.72rem', color: '#ff9800', background: '#fff3e0', padding: '0.15rem 0.4rem', borderRadius: '4px', fontWeight: 700 }}>
                          {t('harness.unsaved')}
                        </span>
                      )}
                    </div>
                    <div className="harness-editor__actions">
                      <button
                        type="button"
                        className="button button--primary"
                        onClick={handleSaveFile}
                        disabled={!isDirty || saveFileMut.isPending}
                        style={{ padding: '0.45rem 0.95rem', fontSize: '0.85rem' }}
                      >
                        <Save size={14} style={{ marginRight: '0.35rem' }} /> {t('common.save')}
                      </button>
                    </div>
                  </div>
                  <div className="harness-editor__textarea-container">
                    {fileLoading ? (
                      <div className="harness-editor__placeholder">
                        <div className="loading-dot" />
                        <span>{t('harness.readingFile')}</span>
                      </div>
                    ) : (
                      <textarea
                        className="harness-editor__textarea"
                        value={editorContent}
                        onChange={(e) => {
                          setEditorContent(e.target.value);
                          setIsDirty(true);
                        }}
                        placeholder={t('harness.editorPlaceholder')}
                      />
                    )}
                  </div>
                </>
              ) : (
                <div className="harness-editor__placeholder">
                  <File size={32} />
                  <span>{t('harness.chooseFile')}</span>
                </div>
              )}
            </div>

            {/* Column 3: Meta & Health Checks (Right) */}
            <div className="harness-editor__meta">
              {/* Basic Meta Info */}
              <div className="harness-meta__section">
                <h4>{t('harness.properties')}</h4>
                <div className="harness-meta__kv">
                  <div className="harness-meta__kv-item">
                    <span className="harness-meta__kv-label">{t('harness.templateId')}</span>
                    <span className="harness-meta__kv-value">{detail.id}</span>
                  </div>
                  <div className="harness-meta__kv-item">
                    <span className="harness-meta__kv-label">{t('harness.workType')}</span>
                    <span className="harness-meta__kv-value">{getWorkTypeLabel(detail.workType)}</span>
                  </div>
                  <div className="harness-meta__kv-item">
                    <span className="harness-meta__kv-label">{t('harness.language')}</span>
                    <span className="harness-meta__kv-value">{detail.language === 'zh-CN' ? t('harness.chinese') : 'English'}</span>
                  </div>
                  {detail.workType === 'code' ? (
                    <div className="harness-meta__kv-item" style={{ flexDirection: 'column', gap: '0.25rem', alignItems: 'flex-start' }}>
                      <span className="harness-meta__kv-label">{t('harness.enabledModules')}</span>
                      <div className="harness-meta__modules-list">
                        {detail.selectedModules && detail.selectedModules.length > 0 ? (
                          detail.selectedModules.map((modId) => {
                            const mod = codeModules.find((m) => m.id === modId);
                            const displayName = mod ? mod.name : modId;
                            return (
                              <span key={modId} className="harness-meta__module-tag">
                                {displayName}
                              </span>
                            );
                          })
                        ) : (
                          <span style={{ color: 'var(--color-muted)', fontSize: '0.8rem' }}>{t('harness.noModules')}</span>
                        )}
                      </div>
                    </div>
                  ) : (
                    <div className="harness-meta__kv-item">
                      <span className="harness-meta__kv-label">{t('harness.createdPreset')}</span>
                      <span className="harness-meta__kv-value">{detail.createdFromPreset || t('harness.custom')}</span>
                    </div>
                  )}
                  <div className="harness-meta__kv-item">
                    <span className="harness-meta__kv-label">{t('harness.sourceType')}</span>
                    <span className="harness-meta__kv-value">{detail.sourceType === 'project' ? t('harness.sourceProject') : t('harness.sourceFolder')}</span>
                  </div>
                  {detail.sourcePath && (
                    <div className="harness-meta__kv-item" style={{ flexDirection: 'column', gap: '0.15rem' }}>
                      <span className="harness-meta__kv-label">{t('harness.sourcePath')}</span>
                      <small style={{ wordBreak: 'break-all', color: 'var(--color-muted)', fontSize: '0.75rem' }}>
                        {detail.sourcePath}
                      </small>
                    </div>
                  )}
                  <div className="harness-meta__kv-item">
                    <span className="harness-meta__kv-label">{t('harness.createdAt')}</span>
                    <span className="harness-meta__kv-value" style={{ fontSize: '0.75rem' }}>
                      {new Date(detail.createdAt).toLocaleString(i18n.language)}
                    </span>
                  </div>
                  <div className="harness-meta__kv-item">
                    <span className="harness-meta__kv-label">{t('harness.updatedOn')}</span>
                    <span className="harness-meta__kv-value" style={{ fontSize: '0.75rem' }}>
                      {new Date(detail.updatedAt).toLocaleString(i18n.language)}
                    </span>
                  </div>
                </div>
              </div>

              {/* Health checks panel */}
              <div className="harness-meta__section" style={{ flex: 1 }}>
                <h4>{t('harness.health')}</h4>
                <div className="harness-health-score-container">
                  <div className="harness-health-score-ring" data-valid={detail.validation.isValid}>
                    {detail.validation.isValid ? t('harness.valid') : t('harness.repair')}
                  </div>
                  <div className="harness-health-score-text">
                    <h5>{t('harness.diagnostic')}</h5>
                    <p>{detail.validation.isValid ? t('harness.validationPassed') : t('harness.validationFailed')}</p>
                  </div>
                </div>

                <div className="harness-validation-list">
                  {/* Required Files Check */}
                  <div className="harness-validation-item" data-type={detail.validation.hasAgentsMd ? 'success' : 'error'}>
                    <div className="harness-validation-icon">
                      {detail.validation.hasAgentsMd ? <CheckCircle size={14} /> : <AlertCircle size={14} />}
                    </div>
                    <span>{t('harness.agentsEntry')}</span>
                  </div>

                  <div className="harness-validation-item" data-type={detail.validation.hasManifest ? 'success' : 'error'}>
                    <div className="harness-validation-icon">
                      {detail.validation.hasManifest ? <CheckCircle size={14} /> : <AlertCircle size={14} />}
                    </div>
                    <span>{t('harness.manifest')}</span>
                  </div>

                  {detail.validation.hasManifest && (
                    <div className="harness-validation-item" data-type={detail.validation.manifestParses ? 'success' : 'error'}>
                      <div className="harness-validation-icon">
                        {detail.validation.manifestParses ? <CheckCircle size={14} /> : <AlertCircle size={14} />}
                      </div>
                      <span>{t('harness.manifestStatus')}</span>
                    </div>
                  )}

                  {/* Missing required files warnings */}
                  {detail.validation.missingRequiredFiles.map((file: string) => (
                    <div key={file} className="harness-validation-item" data-type="error">
                      <div className="harness-validation-icon"><AlertCircle size={14} /></div>
                      <span>{t('harness.missingFile', { file })}</span>
                    </div>
                  ))}

                  {/* Syntax errors warnings */}
                  {detail.validation.syntaxErrors.map((err: string) => (
                    <div key={err} className="harness-validation-item" data-type="error">
                      <div className="harness-validation-icon"><AlertCircle size={14} /></div>
                      <span>{t('harness.syntaxError', { error: err })}</span>
                    </div>
                  ))}

                  {/* Advisory warnings */}
                  {detail.validation.warnings.map((warn) => (
                    <div key={warn} className="harness-validation-item" data-type="warning">
                      <div className="harness-validation-icon"><AlertTriangle size={14} /></div>
                      <span>{t('harness.warning', { warning: warn })}</span>
                    </div>
                  ))}
                </div>
              </div>
            </div>
          </div>
        </div>
      ) : (
        /* List View */
        <div className="page-stack">
          <header className="page-header" style={{ minHeight: 'auto', alignItems: 'center' }}>
            <div style={{ display: 'flex', alignItems: 'baseline', gap: '1rem', flexWrap: 'wrap' }}>
              <h1 style={{ fontSize: '1.75rem', lineHeight: 1 }}>{t('harness.title')}</h1>
              <span style={{ color: 'var(--color-muted)', fontSize: '0.85rem' }}>
                {t('harness.description')}
              </span>
            </div>
          </header>

          <div className="harness-toolbar">
            <div className="harness-toolbar__left">
              <input
                className="harness-search-input"
                placeholder={t('harness.search')}
                aria-label={t('harness.search')}
                value={search}
                onChange={(e) => setSearch(e.target.value)}
              />
              <select
                className="harness-filter-select"
                value={filterWorkType}
                onChange={(e) => setFilterWorkType(e.target.value)}
              >
                <option value="all">{t('harness.allWorkTypes')}</option>
                <option value="code">{t('harness.code')}</option>
                <option value="document">{t('harness.document')}</option>
                <option value="presentation">{t('harness.presentation')}</option>
                <option value="custom">{t('harness.custom')}</option>
              </select>
            </div>

            <div className="harness-toolbar__actions">
              <button className="button button--secondary" onClick={() => refetchSummaries()}>
                <RefreshCw size={16} /> {t('harness.refresh')}
              </button>
              <button className="button button--secondary" onClick={() => setIsImportOpen(true)}>
                <Download size={16} style={{ marginRight: '0.35rem' }} /> {t('harness.import')}
              </button>
              <button className="button button--primary" onClick={() => setIsCreateOpen(true)}>
                <Plus size={16} /> {t('harness.create')}
              </button>
            </div>
          </div>

          {filteredSummaries.length === 0 ? (
            <div className="harness-empty-state">
              <h3>{t('harness.emptyTitle')}</h3>
              <p>
                {t('harness.emptyDescription')}
              </p>
              <div className="harness-empty-ctas">
                <button type="button" className="button button--primary" onClick={() => setIsCreateOpen(true)}>
                  <Plus size={16} /> {t('harness.create')}
                </button>
                <button type="button" className="button button--secondary" onClick={() => setIsImportOpen(true)}>
                  <Download size={16} style={{ marginRight: '0.35rem' }} /> {t('harness.importExisting')}
                </button>
              </div>
            </div>
          ) : (
            <div className="harness-grid">
              {filteredSummaries.map((summary) => (
                <div
                  key={summary.id}
                  className="harness-card"
                  onClick={() => {
                    setSelectedTemplateId(summary.id);
                    setActiveFilePath('AGENTS.md');
                  }}
                >
                  <div className="harness-card__header">
                    <h3 className="harness-card__title">{summary.name}</h3>
                    <span className={`harness-badge harness-badge--${summary.workType}`}>
                      {getWorkTypeLabel(summary.workType)}
                    </span>
                  </div>
                  <p className="harness-card__desc">{summary.description || t('harness.noDescription')}</p>
                  <div className="harness-card__footer">
                    <div className="harness-card__footer-left">
                      <span>{t('harness.files')}: <strong>{summary.fileCount}</strong></span>
                      <span>•</span>
                      <span style={{ display: 'inline-flex', alignItems: 'center', gap: '0.2rem' }}>
                        {summary.isValid ? (
                          <span style={{ color: 'var(--color-success-ink)' }}>● {t('harness.healthy')}</span>
                        ) : (
                          <span style={{ color: 'var(--color-danger)' }}>● {t('harness.needsRepair')}</span>
                        )}
                      </span>
                    </div>
                    <span>{t('harness.updatedAt', { date: new Date(summary.updatedAt).toLocaleDateString(i18n.language) })}</span>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      )}

      {/* Modals */}
      {isCreateOpen && (
        <CreateHarnessModal
          onClose={() => setIsCreateOpen(false)}
          onCreate={(input) => createMut.mutate(input)}
          presets={presets}
          isPresetsLoading={presetsLoading}
          codeModules={codeModules}
          isCodeModulesLoading={codeModulesLoading}
          codeSharedFiles={codeSharedFiles}
          isCodeSharedFilesLoading={codeSharedFilesLoading}
        />
      )}

      {isImportOpen && (
        <ImportHarnessModal
          onClose={() => setIsImportOpen(false)}
          onImportFolder={(path, options) => importFolderMut.mutate({ path, options })}
          onExtractProject={(projectId, options) => extractProjectMut.mutate({ projectId, options })}
        />
      )}

      {isDeleteModalOpen && detail && (
        <ConfirmDeleteHarnessModal
          templateName={detail.name}
          onClose={() => setIsDeleteModalOpen(false)}
          onConfirm={async () => {
            await deleteTemplateMut.mutateAsync(selectedTemplateId!);
            setIsDeleteModalOpen(false);
          }}
          isDeleting={deleteTemplateMut.isPending}
        />
      )}
    </div>
  );
}
