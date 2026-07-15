import { useEffect, useState } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { CheckCircle2, Copy, MonitorCog, ShieldCheck, TerminalSquare } from 'lucide-react';

import { getHealth, getLaunchPreferences, saveLaunchPreferences } from '../../shared/api/tauriClient';
import type { LaunchPreferences, TerminalPreference } from '../../shared/api/types';
import { Card } from '../../shared/ui/Card';
import { PageState } from '../../shared/ui/PageState';
import { StatusBadge } from '../../shared/ui/StatusBadge';

const macosTerminals: Array<{ value: TerminalPreference; label: string; detail: string }> = [
  { value: 'auto', label: '自动选择', detail: '启动时选择首个可用终端。' },
  { value: 'terminal', label: 'Terminal', detail: '使用 macOS 自带终端。' },
  { value: 'iterm', label: 'iTerm2', detail: '适合多标签与开发工作流。' },
  { value: 'warp', label: 'Warp', detail: '使用 Warp 的原生工作区。' },
  { value: 'vscode', label: 'VS Code', detail: '将 CLI 交给集成终端。' },
];

const windowsTerminals: Array<{ value: TerminalPreference; label: string; detail: string }> = [
  { value: 'auto', label: '自动选择', detail: '启动时选择首个可用终端。' },
  { value: 'windows_terminal', label: 'Windows Terminal', detail: '使用 Windows Terminal 新标签页。' },
  { value: 'powershell', label: 'PowerShell', detail: '使用 PowerShell 启动 CLI。' },
  { value: 'git_bash', label: 'Git Bash', detail: '使用 Git Bash 的 POSIX 工作流。' },
  { value: 'vscode', label: 'VS Code', detail: '将 CLI 交给集成终端。' },
];

function PreferenceToggle({ checked, onChange, label, detail }: { checked: boolean; onChange: (checked: boolean) => void; label: string; detail: string }) {
  return (
    <label className="launch-preference-toggle">
      <input type="checkbox" checked={checked} onChange={(event) => onChange(event.target.checked)} />
      <span><strong>{label}</strong><small>{detail}</small></span>
    </label>
  );
}

export function SettingsPage() {
  const queryClient = useQueryClient();
  const health = useQuery({ queryKey: ['health'], queryFn: getHealth });
  const preferences = useQuery({ queryKey: ['launchPreferences'], queryFn: getLaunchPreferences });
  const [draft, setDraft] = useState<LaunchPreferences | null>(null);

  useEffect(() => {
    if (preferences.data) setDraft(preferences.data);
  }, [preferences.data]);

  const saveMutation = useMutation({
    mutationFn: saveLaunchPreferences,
    onSuccess: (saved) => {
      queryClient.setQueryData(['launchPreferences'], saved);
      setDraft(saved);
    },
  });

  if (preferences.isLoading || health.isLoading) return <PageState state="loading" label="正在读取启动偏好..." />;
  if (preferences.isError || health.isError || !health.data || !draft) return <PageState state="error" title="无法读取启动偏好" description="请确认 AgentForge 后端已启动，然后重试。" onRetry={() => { void preferences.refetch(); void health.refetch(); }} />;

  const isMac = health.data.platform === 'macos';
  const terminalOptions = isMac ? macosTerminals : windowsTerminals;
  const terminalKey = isMac ? 'macosTerminal' : 'windowsTerminal';
  const activeTerminal = draft[terminalKey];
  const update = (changes: Partial<LaunchPreferences>) => setDraft((current) => current ? { ...current, ...changes } : current);
  const dirty = JSON.stringify(draft) !== JSON.stringify(preferences.data);

  return (
    <div className="page-stack fixed-workspace-page settings-page-container">
      <header className="page-header">
        <div><h1>设置</h1><p className="page-description">管理本机 Agent、数据安全、外观与更新。</p></div>
        <StatusBadge tone="success">{isMac ? 'macOS' : 'Windows'} 启动偏好</StatusBadge>
      </header>

      <div className="settings-grid settings-grid--launch">
        <Card className="launch-preferences-card">
          <div className="settings-card-heading"><TerminalSquare size={18} /><div><h2>平台启动偏好</h2><p>为当前系统选择打开 Agent CLI 的默认终端。</p></div></div>
          <fieldset className="terminal-choice-grid"><legend className="sr-only">默认终端</legend>
            {terminalOptions.map((option) => <label className={`terminal-choice${activeTerminal === option.value ? ' is-selected' : ''}`} key={option.value}>
              <input type="radio" name="terminal" value={option.value} checked={activeTerminal === option.value} onChange={() => update({ [terminalKey]: option.value })} />
              <span><strong>{option.label}</strong><small>{option.detail}</small></span>
              {activeTerminal === option.value && <CheckCircle2 aria-hidden="true" size={18} />}
            </label>)}
          </fieldset>
          <div className="settings-select-row"><label htmlFor="launch-presentation"><strong>启动位置</strong><small>项目会以同一个目录交给所选终端，不会复制工程。</small></label><select id="launch-presentation" value={draft.launchPresentation} onChange={(event) => update({ launchPresentation: event.target.value as LaunchPreferences['launchPresentation'] })}><option value="new_tab">在新标签页启动</option><option value="new_window">在新窗口启动</option></select></div>
        </Card>

        <Card className="launch-preferences-card">
          <div className="settings-card-heading"><ShieldCheck size={18} /><div><h2>启动前检查</h2><p>在项目交接给 Agent 前，提前发现环境与权限风险。</p></div></div>
          <div className="launch-toggle-list">
            <PreferenceToggle checked={draft.showCommandPreview} onChange={(showCommandPreview) => update({ showCommandPreview })} label="显示命令预览" detail="启动前展示项目目录与安全处理后的命令。" />
            <PreferenceToggle checked={draft.checkEnvironment} onChange={(checkEnvironment) => update({ checkEnvironment })} label="检查项目环境" detail="后续将检查 Git、CLI 与项目运行时。" />
            <PreferenceToggle checked={draft.checkPermissions} onChange={(checkPermissions) => update({ checkPermissions })} label="检查权限摘要" detail="启动前展示文件、网络与 MCP 访问范围。" />
            <PreferenceToggle checked={draft.allowCopyCommandFallback} onChange={(allowCopyCommandFallback) => update({ allowCopyCommandFallback })} label="允许复制命令兜底" detail="终端交接失败时仍可复制完整命令继续工作。" />
          </div>
        </Card>

        <Card className="launch-preferences-card launch-preferences-card--summary">
          <div className="settings-card-heading"><MonitorCog size={18} /><div><h2>启动体验</h2><p>此设置会成为项目概览中 Agent 启动操作的全局默认值。</p></div></div>
          <div className="launch-preference-summary"><span>当前默认</span><strong>{terminalOptions.find((option) => option.value === activeTerminal)?.label}</strong><span>{draft.launchPresentation === 'new_tab' ? '新标签页' : '新窗口'} · {draft.showCommandPreview ? '先预览命令' : '直接启动'}</span></div>
          <p className="muted-copy"><Copy size={14} /> 项目级覆盖与受管运行会在 Agent Control Center 中提供。</p>
        </Card>
      </div>

      <div className="settings-save-bar" aria-live="polite">
        <span>{saveMutation.isSuccess ? '启动偏好已保存。' : dirty ? '你有未保存的启动偏好。' : '所有启动偏好已保存。'}</span>
        <button type="button" className="button button--primary" disabled={!dirty || saveMutation.isPending} onClick={() => saveMutation.mutate(draft)}>{saveMutation.isPending ? '正在保存...' : '保存启动偏好'}</button>
      </div>
    </div>
  );
}
