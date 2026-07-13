import { useMemo, useState } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { AlertTriangle, Check, FileText, RefreshCw, Save } from 'lucide-react';
import {
  applyProjectHarness,
  adoptProjectHarness,
  createProjectHarnessFile,
  deleteProjectHarnessFile,
  getHarnessTemplates,
  getProjectHarnessStatus,
  previewProjectHarnessApplication,
  readProjectHarnessFile,
  unmanageProjectHarness,
  writeProjectHarnessFile,
} from '../../../shared/api/tauriClient';
import type { ProjectHarnessFileDecision } from '../../../shared/api/types';
import { useProjectStore } from '../../../shared/store/projectStore';
import { Card } from '../../../shared/ui/Card';
import { PageState } from '../../../shared/ui/PageState';
import { StatusBadge } from '../../../shared/ui/StatusBadge';
import './project-harness.css';

export function HarnessPage() {
  const queryClient = useQueryClient();
  const { activeProjectId } = useProjectStore();
  const [selectedTemplate, setSelectedTemplate] = useState('');
  const [selectedFile, setSelectedFile] = useState('');
  const [draft, setDraft] = useState('');
  const [decisions, setDecisions] = useState<Record<string, ProjectHarnessFileDecision['action']>>({});
  const [newFilePath, setNewFilePath] = useState('docs/');

  const statusQuery = useQuery({
    queryKey: ['projectHarness', activeProjectId],
    queryFn: () => getProjectHarnessStatus(activeProjectId || ''),
    enabled: !!activeProjectId,
  });
  const templatesQuery = useQuery({
    queryKey: ['harnessTemplates'],
    queryFn: getHarnessTemplates,
    enabled: !!activeProjectId && statusQuery.data?.state !== 'managed',
  });
  const previewQuery = useQuery({
    queryKey: ['projectHarnessPreview', activeProjectId, selectedTemplate],
    queryFn: () => previewProjectHarnessApplication(activeProjectId || '', selectedTemplate),
    enabled: !!activeProjectId && !!selectedTemplate && statusQuery.data?.state !== 'managed',
  });

  const applyMutation = useMutation({
    mutationFn: () =>
      applyProjectHarness({
        projectId: activeProjectId || '',
        templateId: selectedTemplate,
        decisions: Object.entries(decisions).map(([path, action]) => ({ path, action })),
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['projectHarness', activeProjectId] });
      setSelectedTemplate('');
      setDecisions({});
    },
  });
  const saveMutation = useMutation({
    mutationFn: () => writeProjectHarnessFile(activeProjectId || '', selectedFile, draft),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['projectHarness', activeProjectId] }),
  });
  const unmanageMutation = useMutation({
    mutationFn: () => unmanageProjectHarness(activeProjectId || ''),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['projectHarness', activeProjectId] }),
  });
  const adoptMutation = useMutation({
    mutationFn: () => adoptProjectHarness(activeProjectId || ''),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['projectHarness', activeProjectId] }),
  });
  const createFileMutation = useMutation({
    mutationFn: () => createProjectHarnessFile(activeProjectId || '', newFilePath.trim()),
    onSuccess: (file) => {
      setNewFilePath('docs/');
      setSelectedFile(file.path);
      setDraft(file.content);
      queryClient.invalidateQueries({ queryKey: ['projectHarness', activeProjectId] });
    },
  });
  const deleteFileMutation = useMutation({
    mutationFn: (path: string) => deleteProjectHarnessFile(activeProjectId || '', path, true),
    onSuccess: () => {
      setSelectedFile('');
      setDraft('');
      queryClient.invalidateQueries({ queryKey: ['projectHarness', activeProjectId] });
    },
  });

  const files = statusQuery.data?.files || [];
  const activeFile = files.find((file) => file.path === selectedFile);
  const preview = previewQuery.data;
  const hasUnresolvedConflicts = !!preview?.conflicts.some((conflict) => !decisions[conflict.path]);

  const openFile = async (path: string) => {
    setSelectedFile(path);
    const file = await readProjectHarnessFile(activeProjectId || '', path);
    setDraft(file.content);
  };

  const setConflictAction = (path: string, action: ProjectHarnessFileDecision['action']) => {
    setDecisions((current) => ({ ...current, [path]: action }));
  };

  const statusLabel = useMemo(() => {
    switch (statusQuery.data?.state) {
      case 'managed': return '已配置';
      case 'invalid': return '需要检查';
      case 'unmanaged_detected': return '检测到未纳管 Harness';
      default: return '未配置';
    }
  }, [statusQuery.data?.state]);

  if (!activeProjectId) {
    return <PageState state="empty" title="尚未选择任何项目" description="请先在左侧选择一个项目，再配置项目 Harness。" />;
  }
  if (statusQuery.isLoading) {
    return <PageState state="loading" label="正在从项目目录读取最新 Harness 状态..." />;
  }
  if (statusQuery.isError || !statusQuery.data) {
    return <PageState state="error" title="无法读取项目 Harness" description="请确认项目目录仍然存在，然后刷新页面。" onRetry={() => void statusQuery.refetch()} />;
  }

  return (
    <div className="page-stack project-harness-page">
      <div className="project-harness-header">
        <div>
          <p className="eyebrow">PROJECT HARNESS</p>
          <h2>项目 Harness</h2>
          <p className="muted-copy">项目实例独立于模板，所有文件修改都直接写入当前项目。</p>
        </div>
        <StatusBadge tone={statusQuery.data.state === 'managed' ? 'success' : statusQuery.data.state === 'invalid' ? 'danger' : 'neutral'}>
          {statusLabel}
        </StatusBadge>
      </div>

      {statusQuery.data.state !== 'managed' && (
        <Card>
          <div className="project-harness-section-heading">
            <div>
              <h3>{statusQuery.data.state === 'unmanaged_detected' ? '检测到现有 Harness' : '应用 Harness 模板'}</h3>
              <p className="muted-copy">
                {statusQuery.data.state === 'unmanaged_detected'
                  ? '项目目录已有 Harness 文件，但 AgentForge 尚未建立管理记录。'
                  : '选择一套模板，将完整文件集复制为当前项目的独立实例。'}
              </p>
            </div>
          </div>
          {statusQuery.data.state === 'unmanaged_detected' ? (
            <div className="project-harness-actions"><p className="project-harness-warning"><AlertTriangle size={16} /> 纳管不会改写任何项目文件。</p><button type="button" className="button button--primary" onClick={() => adoptMutation.mutate()} disabled={adoptMutation.isPending}>{adoptMutation.isPending ? '正在纳管...' : '纳管现有 Harness'}</button></div>
          ) : (
            <>
              <label className="project-harness-label" htmlFor="project-harness-template">选择模板</label>
              <select id="project-harness-template" className="project-harness-template-select" value={selectedTemplate} onChange={(event) => { setSelectedTemplate(event.target.value); setDecisions({}); }}>
                <option value="">请选择 Harness 模板</option>
                {(templatesQuery.data || []).map((template) => (
                  <option key={template.id} value={template.id}>{template.name} · {template.workType} · {template.language}</option>
                ))}
              </select>
              {preview && (
                <div className="project-harness-application">
                  <div className="project-harness-file-summary">
                    <strong>将应用 {preview.templateFiles.length} 个文件</strong>
                    <span>{preview.conflicts.length ? `发现 ${preview.conflicts.length} 个冲突` : '没有文件冲突'}</span>
                  </div>
                  {preview.missingAgentsReferences.length > 0 && (
                    <p className="project-harness-warning"><AlertTriangle size={16} /> AGENTS.md 引用了模板中不存在的文件，无法应用。</p>
                  )}
                  {preview.conflicts.map((conflict) => (
                    <div key={conflict.path} className="project-harness-conflict">
                      <div><FileText size={15} /><code>{conflict.path}</code></div>
                      <div className="project-harness-conflict-actions">
                        {(['keep', 'overwrite', 'skip'] as const).map((action) => (
                          <button key={action} type="button" className={decisions[conflict.path] === action ? 'is-selected' : ''} onClick={() => setConflictAction(conflict.path, action)}>
                            {action === 'keep' ? '保留项目文件' : action === 'overwrite' ? '使用模板覆盖' : '跳过模板文件'}
                          </button>
                        ))}
                      </div>
                    </div>
                  ))}
                  <button type="button" className="button button--primary" disabled={hasUnresolvedConflicts || preview.missingAgentsReferences.length > 0 || applyMutation.isPending} onClick={() => applyMutation.mutate()}>
                    <Check size={16} /> {applyMutation.isPending ? '正在应用...' : '确认应用 Harness'}
                  </button>
                  {applyMutation.isError && <p className="form-error">{String(applyMutation.error)}</p>}
                </div>
              )}
            </>
          )}
        </Card>
      )}

      {(statusQuery.data.state === 'managed' || statusQuery.data.state === 'invalid') && (
        <>
          <Card>
            <div className="project-harness-meta">
              <div><span>来源模板</span><strong>{statusQuery.data.sourceTemplateId || '未知'}</strong></div>
              <div><span>来源状态</span><strong>{statusQuery.data.sourceStatus === 'changed' ? '模板已有变化' : statusQuery.data.sourceStatus === 'deleted' ? '原模板已删除' : '独立项目副本'}</strong></div>
              <div><span>应用时间</span><strong>{statusQuery.data.appliedAt || '未知'}</strong></div>
            </div>
            {statusQuery.data.warnings.map((warning) => <p key={warning} className="project-harness-warning"><AlertTriangle size={16} /> {warning}</p>)}
            <div className="project-harness-actions">
              <button type="button" className="button button--secondary" onClick={() => statusQuery.refetch()}><RefreshCw size={15} /> 刷新磁盘状态</button>
              <button type="button" className="button button--secondary" onClick={() => unmanageMutation.mutate()} disabled={unmanageMutation.isPending}>解除纳管</button>
            </div>
          </Card>
          <Card>
            <div className="project-harness-editor-layout">
              <div className="project-harness-file-list">
                <h3>Harness 文件</h3>
                {files.map((file) => <button type="button" key={file.path} className={selectedFile === file.path ? 'is-active' : ''} onClick={() => void openFile(file.path)}><FileText size={15} /><span>{file.path}</span>{file.changedSinceApply && <i title="已修改" />}</button>)}
              </div>
              <div className="project-harness-editor">
                {activeFile ? <>
                      <div className="project-harness-editor-toolbar"><code>{activeFile.path}</code><div className="project-harness-actions"><button type="button" className="button button--primary" onClick={() => saveMutation.mutate()} disabled={saveMutation.isPending}><Save size={15} /> 保存</button><button type="button" className="button button--secondary" disabled={deleteFileMutation.isPending} title="删除文件" onClick={() => { if (window.confirm(`确定删除 ${activeFile.path} 吗？`)) deleteFileMutation.mutate(activeFile.path); }}>删除</button></div></div>
                  <textarea value={draft} onChange={(event) => setDraft(event.target.value)} spellCheck={false} aria-label={`编辑 ${activeFile.path}`} />
                </> : <div className="project-harness-empty-editor">选择一个 Harness 文件开始编辑。</div>}
                <div className="project-harness-create-file"><input value={newFilePath} onChange={(event) => setNewFilePath(event.target.value)} aria-label="新 Harness 文件路径" /><button type="button" className="button button--secondary" onClick={() => createFileMutation.mutate()} disabled={!newFilePath.trim() || createFileMutation.isPending}>新增文件</button></div>
              </div>
            </div>
          </Card>
        </>
      )}
    </div>
  );
}
