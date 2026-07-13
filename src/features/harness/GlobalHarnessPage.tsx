import React, { useState, useEffect, useRef } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { Plus, Download, ArrowLeft, Save, Copy, Trash, File, FolderOpen, RefreshCw, CheckCircle, AlertTriangle, AlertCircle } from 'lucide-react';
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
      alert(`创建模板失败: ${err.message || String(err)}`);
    },
  });

  const saveFileMut = useMutation({
    mutationFn: ({ templateId, path, content }: { templateId: string; path: string; content: string }) =>
      writeHarnessFile(templateId, path, content),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['harness-detail', selectedTemplateId] });
      queryClient.invalidateQueries({ queryKey: ['harness-summaries'] });
      setIsDirty(false);
      alert('文件保存成功！');
    },
    onError: (err: any) => {
      alert(`保存文件失败: ${err.message || String(err)}`);
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
      alert(`创建文件失败: ${err.message || String(err)}`);
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
      alert(`删除文件失败: ${err.message || String(err)}`);
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
      alert(`删除模板失败: ${err.message || String(err)}`);
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
      alert(`复制模板失败: ${err.message || String(err)}`);
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
      alert(`导入模板失败: ${err.message || String(err)}`);
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
      alert(`提取模板失败: ${err.message || String(err)}`);
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
    const targetName = prompt('请输入新副本的显示名称:');
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
      case 'code': return 'Code Work';
      case 'document': return 'Document Work';
      case 'presentation': return 'Presentation Work';
      default: return 'Custom';
    }
  };

  if (summariesLoading) {
    return (
      <div className="page-state">
        <div className="loading-dot" />
        <p>加载 Harness 模板库...</p>
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
                <Copy size={16} style={{ marginRight: '0.35rem' }} /> 复制副本
              </button>
              <button type="button" className="button button--secondary" onClick={handleDeleteTemplate} style={{ color: 'var(--color-danger)' }}>
                <Trash size={16} style={{ marginRight: '0.35rem' }} /> 删除模板
              </button>
            </div>
          </header>

          <div className="harness-editor-layout">
            {/* Column 1: File Tree (Left) */}
            <div className="harness-editor__tree">
              <div className="harness-tree__header">
                <h4>📁 模板文件树</h4>
                <div className="harness-tree__actions">
                  <button type="button" className="harness-tree__btn" title="新建文件" onClick={handleCreateFile}>
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
                          if (!confirm('当前文件有未保存的修改，切换文件将丢失修改。是否继续？')) {
                            return;
                          }
                        }
                        setActiveFilePath(file.path);
                      }}
                    >
                      <div className="harness-tree__item-left">
                        <File size={14} color={file.isStandard ? 'var(--color-primary-ink)' : 'var(--color-muted)'} />
                        <span className="harness-tree__item-name">{file.path}</span>
                      </div>
                      {file.path !== 'AGENTS.md' && file.path !== 'docs/harness.toml' && (
                        isDeletingConfirm ? (
                          <div className="harness-tree__confirm-actions" onClick={(e) => e.stopPropagation()}>
                            <button
                              type="button"
                              className="harness-tree__confirm-btn harness-tree__confirm-btn--yes"
                              title="确认删除"
                              onClick={() => {
                                deleteFileMut.mutate({ templateId: selectedTemplateId!, path: file.path });
                                setDeletingFilePath(null);
                              }}
                            >
                              确认
                            </button>
                            <button
                              type="button"
                              className="harness-tree__confirm-btn harness-tree__confirm-btn--no"
                              title="取消"
                              onClick={() => setDeletingFilePath(null)}
                            >
                              取消
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
                            title="删除文件"
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
                      <File size={14} color="var(--color-primary-ink)" />
                      <input
                        type="text"
                        className="harness-tree__input"
                        value={newFilePathInput}
                        onChange={(e) => setNewFilePathInput(e.target.value)}
                        placeholder="输入文件路径 (例: docs/rules.md)..."
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
                              alert('非法的相对文件路径！');
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
                          未保存
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
                        <Save size={14} style={{ marginRight: '0.35rem' }} /> 保存
                      </button>
                    </div>
                  </div>
                  <div className="harness-editor__textarea-container">
                    {fileLoading ? (
                      <div className="harness-editor__placeholder">
                        <div className="loading-dot" />
                        <span>正在读取文件...</span>
                      </div>
                    ) : (
                      <textarea
                        className="harness-editor__textarea"
                        value={editorContent}
                        onChange={(e) => {
                          setEditorContent(e.target.value);
                          setIsDirty(true);
                        }}
                        placeholder="在此处输入规约内容，支持 Markdown, JSON, TOML 等格式。"
                      />
                    )}
                  </div>
                </>
              ) : (
                <div className="harness-editor__placeholder">
                  <File size={32} />
                  <span>请从左侧文件树中选择一个规约文件进行编辑。</span>
                </div>
              )}
            </div>

            {/* Column 3: Meta & Health Checks (Right) */}
            <div className="harness-editor__meta">
              {/* Basic Meta Info */}
              <div className="harness-meta__section">
                <h4>模板属性</h4>
                <div className="harness-meta__kv">
                  <div className="harness-meta__kv-item">
                    <span className="harness-meta__kv-label">模板 ID</span>
                    <span className="harness-meta__kv-value">{detail.id}</span>
                  </div>
                  <div className="harness-meta__kv-item">
                    <span className="harness-meta__kv-label">工作类型</span>
                    <span className="harness-meta__kv-value">{getWorkTypeLabel(detail.workType)}</span>
                  </div>
                  <div className="harness-meta__kv-item">
                    <span className="harness-meta__kv-label">模板语言</span>
                    <span className="harness-meta__kv-value">{detail.language === 'zh-CN' ? '简体中文' : 'English'}</span>
                  </div>
                  {detail.workType === 'code' ? (
                    <div className="harness-meta__kv-item" style={{ flexDirection: 'column', gap: '0.25rem', alignItems: 'flex-start' }}>
                      <span className="harness-meta__kv-label">启用模块</span>
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
                          <span style={{ color: 'var(--color-muted)', fontSize: '0.8rem' }}>无模块</span>
                        )}
                      </div>
                    </div>
                  ) : (
                    <div className="harness-meta__kv-item">
                      <span className="harness-meta__kv-label">创建预设</span>
                      <span className="harness-meta__kv-value">{detail.createdFromPreset || 'Custom Work'}</span>
                    </div>
                  )}
                  <div className="harness-meta__kv-item">
                    <span className="harness-meta__kv-label">来源类型</span>
                    <span className="harness-meta__kv-value">{detail.sourceType === 'project' ? '项目提取' : '本地目录'}</span>
                  </div>
                  {detail.sourcePath && (
                    <div className="harness-meta__kv-item" style={{ flexDirection: 'column', gap: '0.15rem' }}>
                      <span className="harness-meta__kv-label">导入源路径</span>
                      <small style={{ wordBreak: 'break-all', color: 'var(--color-muted)', fontSize: '0.75rem' }}>
                        {detail.sourcePath}
                      </small>
                    </div>
                  )}
                  <div className="harness-meta__kv-item">
                    <span className="harness-meta__kv-label">创建时间</span>
                    <span className="harness-meta__kv-value" style={{ fontSize: '0.75rem' }}>
                      {new Date(detail.createdAt).toLocaleString()}
                    </span>
                  </div>
                  <div className="harness-meta__kv-item">
                    <span className="harness-meta__kv-label">更新时间</span>
                    <span className="harness-meta__kv-value" style={{ fontSize: '0.75rem' }}>
                      {new Date(detail.updatedAt).toLocaleString()}
                    </span>
                  </div>
                </div>
              </div>

              {/* Health checks panel */}
              <div className="harness-meta__section" style={{ flex: 1 }}>
                <h4>规约健康诊断</h4>
                <div className="harness-health-score-container">
                  <div className="harness-health-score-ring" data-valid={detail.validation.isValid}>
                    {detail.validation.isValid ? '合格' : '待修复'}
                  </div>
                  <div className="harness-health-score-text">
                    <h5>诊断状态</h5>
                    <p>{detail.validation.isValid ? '基础依赖完整且校验通过' : '有缺失依赖或语法错误'}</p>
                  </div>
                </div>

                <div className="harness-validation-list">
                  {/* Required Files Check */}
                  <div className="harness-validation-item" data-type={detail.validation.hasAgentsMd ? 'success' : 'error'}>
                    <div className="harness-validation-icon">
                      {detail.validation.hasAgentsMd ? <CheckCircle size={14} /> : <AlertCircle size={14} />}
                    </div>
                    <span>主指令入口 AGENTS.md</span>
                  </div>

                  <div className="harness-validation-item" data-type={detail.validation.hasManifest ? 'success' : 'error'}>
                    <div className="harness-validation-icon">
                      {detail.validation.hasManifest ? <CheckCircle size={14} /> : <AlertCircle size={14} />}
                    </div>
                    <span>清单配置 docs/harness.toml</span>
                  </div>

                  {detail.validation.hasManifest && (
                    <div className="harness-validation-item" data-type={detail.validation.manifestParses ? 'success' : 'error'}>
                      <div className="harness-validation-icon">
                        {detail.validation.manifestParses ? <CheckCircle size={14} /> : <AlertCircle size={14} />}
                      </div>
                      <span>docs/harness.toml 解析状态</span>
                    </div>
                  )}

                  {/* Missing required files warnings */}
                  {detail.validation.missingRequiredFiles.map((file: string) => (
                    <div key={file} className="harness-validation-item" data-type="error">
                      <div className="harness-validation-icon"><AlertCircle size={14} /></div>
                      <span>缺失必需文件: <code>{file}</code></span>
                    </div>
                  ))}

                  {/* Syntax errors warnings */}
                  {detail.validation.syntaxErrors.map((err: string) => (
                    <div key={err} className="harness-validation-item" data-type="error">
                      <div className="harness-validation-icon"><AlertCircle size={14} /></div>
                      <span>语法校验失败: {err}</span>
                    </div>
                  ))}

                  {/* Advisory warnings */}
                  {detail.validation.warnings.map((warn) => (
                    <div key={warn} className="harness-validation-item" data-type="warning">
                      <div className="harness-validation-icon"><AlertTriangle size={14} /></div>
                      <span>规约引流警告: {warn}</span>
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
              <h1 style={{ fontSize: '1.75rem', lineHeight: 1 }}>Harness 模板管理</h1>
              <span style={{ color: 'var(--color-muted)', fontSize: '0.85rem' }}>
                定义面向长周期 AI 协同项目的工程规约、验收标准与高风险边界。
              </span>
            </div>
          </header>

          <div className="harness-toolbar">
            <div className="harness-toolbar__left">
              <input
                className="harness-search-input"
                placeholder="搜索模板名称、ID或描述..."
                value={search}
                onChange={(e) => setSearch(e.target.value)}
              />
              <select
                className="harness-filter-select"
                value={filterWorkType}
                onChange={(e) => setFilterWorkType(e.target.value)}
              >
                <option value="all">全部工作类型</option>
                <option value="code">Code Work (代码编写)</option>
                <option value="document">Document Work (报告论文)</option>
                <option value="presentation">Presentation Work (演示汇报)</option>
                <option value="custom">Custom (自定义规格)</option>
              </select>
            </div>

            <div className="harness-toolbar__actions">
              <button className="button button--secondary" onClick={() => refetchSummaries()}>
                <RefreshCw size={16} /> 刷新
              </button>
              <button className="button button--secondary" onClick={() => setIsImportOpen(true)}>
                <Download size={16} style={{ marginRight: '0.35rem' }} /> 导入模板
              </button>
              <button className="button button--primary" onClick={() => setIsCreateOpen(true)}>
                <Plus size={16} /> 新建 Harness
              </button>
            </div>
          </div>

          {filteredSummaries.length === 0 ? (
            <div className="harness-empty-state">
              <h3>暂无 Harness 模板</h3>
              <p>
                模板是指导 Agent 工作的核心规范配置包，定义了核心指令（AGENTS.md）及验收标准、高风险检查等文件资产。
              </p>
              <div className="harness-empty-ctas">
                <button type="button" className="button button--primary" onClick={() => setIsCreateOpen(true)}>
                  <Plus size={16} /> 新建 Harness
                </button>
                <button type="button" className="button button--secondary" onClick={() => setIsImportOpen(true)}>
                  <Download size={16} style={{ marginRight: '0.35rem' }} /> 导入已有模板
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
                  <p className="harness-card__desc">{summary.description || '暂无详细描述。'}</p>
                  <div className="harness-card__footer">
                    <div className="harness-card__footer-left">
                      <span>文件: <strong>{summary.fileCount}</strong></span>
                      <span>•</span>
                      <span style={{ display: 'inline-flex', alignItems: 'center', gap: '0.2rem' }}>
                        {summary.isValid ? (
                          <span style={{ color: 'var(--color-primary-ink)' }}>● 状态良好</span>
                        ) : (
                          <span style={{ color: 'var(--color-danger)' }}>● 待修复</span>
                        )}
                      </span>
                    </div>
                    <span>更新于 {new Date(summary.updatedAt).toLocaleDateString()}</span>
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
