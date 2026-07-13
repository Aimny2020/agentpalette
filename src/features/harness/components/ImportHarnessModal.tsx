import React, { useState } from 'react';
import { X, Search, FileText, CheckCircle, AlertCircle } from 'lucide-react';
import { useQuery } from '@tanstack/react-query';
import { getProjects, inspectHarnessImport } from '../../../shared/api/tauriClient';
import { useProjectStore } from '../../../shared/store/projectStore';
import { HarnessImportOptions, HarnessExtractOptions } from '../../../shared/api/types';

interface ImportHarnessModalProps {
  onClose: () => void;
  onImportFolder: (path: string, options: HarnessImportOptions) => void;
  onExtractProject: (projectId: string, options: HarnessExtractOptions) => void;
}

const EXTRACTABLE_FILES = [
  { path: 'AGENTS.md', label: 'AGENTS.md (Agent 主入口指令)' },
  { path: 'docs/architecture.md', label: 'docs/architecture.md (项目架构规范)' },
  { path: 'docs/feature_list.json', label: 'docs/feature_list.json (项目功能列表)' },
  { path: 'docs/task-status.md', label: 'docs/task-status.md (任务状态与记录)' },
  { path: 'docs/verification.md', label: 'docs/verification.md (测试与验证规范)' },
  { path: 'docs/risk-rules.md', label: 'docs/risk-rules.md (安全红线规则)' },
  { path: 'docs/agent-profile.md', label: 'docs/agent-profile.md (Agent 风格预设)' },
  { path: 'docs/harness.toml', label: 'docs/harness.toml (Harness 配置文件)' },
];

export function ImportHarnessModal({ onClose, onImportFolder, onExtractProject }: ImportHarnessModalProps) {
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
      alert('请输入文件夹绝对路径！');
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
      alert(`检查失败: ${err.message || String(err)}`);
    } finally {
      setInspecting(false);
    }
  };

  const handleFolderImportSubmit = () => {
    if (!importName.trim()) {
      alert('请填写显示名称！');
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
      alert('请选择要提取的源项目！');
      return;
    }
    if (!extractName.trim()) {
      alert('请填写显示名称！');
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
          <h3>导入 Harness 模板</h3>
          <button type="button" className="close-btn" onClick={onClose}>
            <X size={20} />
          </button>
        </div>

        <div className="harness-import-tabs">
          <div
            className="harness-import-tab"
            data-active={tab === 'folder'}
            onClick={() => setTab('folder')}
          >
            从本地目录导入
          </div>
          <div
            className="harness-import-tab"
            data-active={tab === 'extract'}
            onClick={() => setTab('extract')}
          >
            从当前项目提取
          </div>
        </div>

        {tab === 'folder' ? (
          /* Tab 1: Local Folder Import */
          <div className="harness-modal-content" style={{ display: 'flex', flexDirection: 'column', gap: 'var(--space-2)' }}>
            <div className="harness-form-group">
              <label htmlFor="import-path">本地文件夹绝对路径</label>
              <div style={{ display: 'flex', gap: 'var(--space-1)' }}>
                <input
                  id="import-path"
                  style={{ flex: 1 }}
                  placeholder="例如: /Users/username/workspace/my-harness"
                  value={folderPath}
                  onChange={(e) => setFolderPath(e.target.value)}
                />
                <button
                  type="button"
                  className="button button--secondary"
                  onClick={handleInspect}
                  disabled={inspecting}
                >
                  <Search size={16} style={{ marginRight: '0.25rem' }} /> {inspecting ? '分析中...' : '检查'}
                </button>
              </div>
            </div>

            {inspectionResult && (
              <div className="harness-import-inspection-panel">
                <h5>🔍 目录分析结果</h5>
                <div style={{ display: 'flex', flexDirection: 'column', gap: '0.4rem', fontSize: '0.85rem', marginBottom: 'var(--space-2)' }}>
                  <div style={{ display: 'flex', alignItems: 'center', gap: '0.35rem' }}>
                    {inspectionResult.hasAgentsMd ? (
                      <CheckCircle size={14} color="var(--color-primary-ink)" />
                    ) : (
                      <AlertCircle size={14} color="var(--color-danger)" />
                    )}
                    <span>包含 AGENTS.md: {inspectionResult.hasAgentsMd ? '是' : '否 (导入后将自动生成默认主指令)'}</span>
                  </div>
                  <div style={{ display: 'flex', alignItems: 'center', gap: '0.35rem' }}>
                    {inspectionResult.hasManifest ? (
                      <CheckCircle size={14} color="var(--color-primary-ink)" />
                    ) : (
                      <AlertCircle size={14} color="var(--color-muted)" />
                    )}
                    <span>包含 docs/harness.toml: {inspectionResult.hasManifest ? '是' : '否 (导入后将自动生成默认清单文件)'}</span>
                  </div>
                  <div style={{ color: 'var(--color-muted)', fontSize: '0.8rem' }}>
                    发现可用文件: {inspectionResult.foundFiles.length} 个
                  </div>
                </div>

                <div style={{ display: 'flex', flexDirection: 'column', gap: 'var(--space-2)', borderTop: '1px solid var(--color-outline)', paddingTop: 'var(--space-2)' }}>
                  <div className="harness-form-group">
                    <label htmlFor="imp-name">显示名称</label>
                    <input
                      id="imp-name"
                      value={importName}
                      onChange={(e) => setImportName(e.target.value)}
                      placeholder="e.g. 自定义导入规范"
                    />
                  </div>
                  <div className="harness-form-group">
                    <label htmlFor="imp-desc">描述信息</label>
                    <textarea
                      id="imp-desc"
                      value={importDesc}
                      onChange={(e) => setImportDesc(e.target.value)}
                      placeholder="描述模板用途..."
                      rows={2}
                    />
                  </div>
                  <div className="harness-form-group">
                    <label htmlFor="imp-worktype">AI 工作类型</label>
                    <select
                      id="imp-worktype"
                      className="harness-filter-select"
                      style={{ borderRadius: 'var(--radius-sm)', width: '100%' }}
                      value={importWorkType}
                      onChange={(e) => setImportWorkType(e.target.value)}
                    >
                      <option value="code">Code Work (代码)</option>
                      <option value="document">Document Work (报告论文)</option>
                      <option value="presentation">Presentation Work (演示)</option>
                      <option value="custom">Custom (自定义)</option>
                    </select>
                  </div>
                  <div className="harness-form-group">
                    <label htmlFor="imp-language">模板语言</label>
                    <select id="imp-language" value={importLanguage} onChange={(e) => setImportLanguage(e.target.value as 'zh-CN' | 'en')}>
                      <option value="zh-CN">简体中文</option><option value="en">English</option>
                    </select>
                  </div>

                  <div style={{ display: 'flex', justifyContent: 'flex-end', marginTop: 'var(--space-1)' }}>
                    <button
                      type="button"
                      className="button button--primary"
                      onClick={handleFolderImportSubmit}
                    >
                      确认导入为全局模板
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
              <label htmlFor="extract-project-select">选择提取源项目</label>
              <select
                id="extract-project-select"
                className="harness-filter-select"
                style={{ borderRadius: 'var(--radius-sm)', width: '100%' }}
                value={selectedProjectId}
                onChange={(e) => setSelectedProjectId(e.target.value)}
              >
                <option value="">-- 请选择项目 --</option>
                {projects.map((p) => (
                  <option key={p.id} value={p.id}>
                    {p.name} ({p.path})
                  </option>
                ))}
              </select>
            </div>

            <div className="harness-form-group">
              <label>选择要提取的项目文件 (如果文件存在则会拷贝)</label>
              <div className="harness-checklist" style={{ maxHeight: '10rem' }}>
                {EXTRACTABLE_FILES.map((file) => (
                  <div
                    key={file.path}
                    className="harness-checklist-item"
                    onClick={() => handleToggleExtractFile(file.path)}
                  >
                    <input
                      type="checkbox"
                      checked={selectedFiles.includes(file.path)}
                      onChange={() => {}}
                    />
                    <span>{file.label}</span>
                  </div>
                ))}
              </div>
            </div>

            <div style={{ display: 'flex', flexDirection: 'column', gap: 'var(--space-2)', borderTop: '1px solid var(--color-outline)', paddingTop: 'var(--space-2)' }}>
              <div className="harness-form-group">
                <label htmlFor="ext-name">显示名称</label>
                <input
                  id="ext-name"
                  value={extractName}
                  onChange={(e) => setExtractName(e.target.value)}
                  placeholder="e.g. 提取项目规范"
                />
              </div>
              <div className="harness-form-group">
                <label htmlFor="ext-desc">描述信息</label>
                <textarea
                  id="ext-desc"
                  value={extractDesc}
                  onChange={(e) => setExtractDesc(e.target.value)}
                  placeholder="描述提取的模板用途..."
                  rows={2}
                />
              </div>
              <div className="harness-form-group">
                <label htmlFor="ext-worktype">AI 工作类型</label>
                <select
                  id="ext-worktype"
                  className="harness-filter-select"
                  style={{ borderRadius: 'var(--radius-sm)', width: '100%' }}
                  value={extractWorkType}
                  onChange={(e) => setExtractWorkType(e.target.value)}
                >
                  <option value="code">Code Work (代码)</option>
                  <option value="document">Document Work (报告论文)</option>
                  <option value="presentation">Presentation Work (演示)</option>
                  <option value="custom">Custom (自定义)</option>
                </select>
              </div>
              <div className="harness-form-group">
                <label htmlFor="ext-language">模板语言</label>
                <select id="ext-language" value={extractLanguage} onChange={(e) => setExtractLanguage(e.target.value as 'zh-CN' | 'en')}>
                  <option value="zh-CN">简体中文</option><option value="en">English</option>
                </select>
              </div>

              <div style={{ display: 'flex', justifyContent: 'flex-end', marginTop: 'var(--space-1)' }}>
                <button
                  type="button"
                  className="button button--primary"
                  onClick={handleExtractSubmit}
                >
                  提取并生成全局模板
                </button>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
