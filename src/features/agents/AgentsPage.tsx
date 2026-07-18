import { useMemo, useState } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { AppWindow, Download, ExternalLink, RefreshCw, Search, TerminalSquare, Trash2, X } from 'lucide-react';

import {
  applyAgentMaintenance,
  checkAgentUpdates,
  getAgentMaintenancePlan,
  getLocalAgents,
  openDesktopAgent,
} from '../../shared/api/tauriClient';
import type { AgentMaintenanceAction, AgentMaintenancePlan, AgentUpdate, LocalAgent } from '../../shared/api/types';
import { PageState } from '../../shared/ui/PageState';
import { StatusBadge } from '../../shared/ui/StatusBadge';

function updateLabel(update: AgentUpdate | undefined, ready: boolean, isFetching: boolean) {
  if (!ready) return '未安装';
  if (isFetching && !update) return '正在检查更新';
  if (!update) return '暂时无法确认更新';
  if (update.status === 'current') return '已是最新版本';
  if (update.status === 'available') return `可更新至 v${update.latestVersion}`;
  return '暂时无法确认更新';
}

function AgentTile({
  agent,
  update,
  isFetching,
  onOpen,
  onMaintain,
  isOpening,
}: {
  agent: LocalAgent;
  update?: AgentUpdate;
  isFetching: boolean;
  onOpen: () => void;
  onMaintain: (action: AgentMaintenanceAction) => void;
  isOpening: boolean;
}) {
  const ready = agent.status === 'ready';
  const manageable = agent.surface === 'cli';
  const isDesktop = agent.surface === 'desktop';
  const label = isDesktop ? (ready ? '已发现' : '未发现') : updateLabel(update, ready, isFetching);
  const tone = ready ? 'success' : 'neutral';

  return (
    <article className="agent-tile">
      <div className="agent-tile__heading">
        <span className="agent-tile__icon">{isDesktop ? <AppWindow size={17} /> : <TerminalSquare size={17} />}</span>
        <div>
          <strong>{agent.displayName}</strong>
          <span>{isDesktop ? '桌面客户端' : '命令行 CLI'}</span>
        </div>
        <StatusBadge tone={tone}>{ready ? '已就绪' : '未安装'}</StatusBadge>
      </div>

      <div className="agent-tile__details">
        {ready && <span>版本 <b>v{agent.version || '未知'}</b></span>}
        {!isDesktop && <span className={update?.status === 'available' ? 'agent-tile__update-available' : ''}>{label}</span>}
        {isDesktop && <span>{ready ? '可直接打开' : '未在本机应用目录中发现'}</span>}
      </div>

      <div className="agent-tile__actions">
        {isDesktop && ready && <button type="button" className="button button--primary" onClick={onOpen} disabled={isOpening}><ExternalLink size={14} /> {isOpening ? '正在打开...' : '打开'}</button>}
        {manageable && !ready && agent.canInstall && <button type="button" className="button button--primary" onClick={() => onMaintain('install')}><Download size={14} /> 安装</button>}
        {manageable && ready && agent.canUpdate && update?.status !== 'current' && <button type="button" className="button button--primary" onClick={() => onMaintain('update')}><Download size={14} /> 更新</button>}
        {manageable && ready && update?.status === 'current' && <span className="agent-tile__current">已最新</span>}
        {manageable && ready && agent.canUninstall && <button type="button" className="button button--ghost-danger" onClick={() => onMaintain('uninstall')}><Trash2 size={14} /> 卸载</button>}
      </div>
    </article>
  );
}

function MaintenanceConfirmation({
  plan,
  isApplying,
  error,
  onClose,
  onConfirm,
}: {
  plan: AgentMaintenancePlan;
  isApplying: boolean;
  error: string | null;
  onClose: () => void;
  onConfirm: () => void;
}) {
  const label = plan.action === 'install' ? '安装' : plan.action === 'update' ? '更新' : '卸载';
  const dangerous = plan.action === 'uninstall';
  return (
    <div className="modal-overlay" onClick={isApplying ? undefined : onClose}>
      <div className="modal-body compact-modal agent-maintenance-modal" role="dialog" aria-modal="true" aria-labelledby="agent-maintenance-title" onClick={(event) => event.stopPropagation()}>
        <div className="modal-header">
          <h3 id="agent-maintenance-title">确认{label} Agent</h3>
          <button type="button" className="close-btn" onClick={onClose} disabled={isApplying} aria-label="关闭"><X size={20} /></button>
        </div>
        <div className="agent-maintenance-modal__content">
          <p>AgentPalette 将在本机执行以下受控命令：</p>
          <code>{plan.command}</code>
          {dangerous && <p className="agent-maintenance-modal__warning">卸载会移除该 CLI 的全局安装。项目和聊天记录不会被删除。</p>}
          {error && <p className="project-agent-error">{error}</p>}
        </div>
        <div className="actions-footer">
          <button type="button" className="button button--secondary" onClick={onClose} disabled={isApplying}>取消</button>
          <button type="button" className={dangerous ? 'button button--danger' : 'button button--primary'} onClick={onConfirm} disabled={isApplying}>{isApplying ? `正在${label}...` : `确认${label}`}</button>
        </div>
      </div>
    </div>
  );
}

export function AgentsPage() {
  const queryClient = useQueryClient();
  const [surface, setSurface] = useState<'cli' | 'desktop'>('cli');
  const [search, setSearch] = useState('');
  const [plan, setPlan] = useState<AgentMaintenancePlan | null>(null);
  const [actionError, setActionError] = useState<string | null>(null);
  const agents = useQuery({ queryKey: ['localAgents'], queryFn: getLocalAgents });
  const updates = useQuery({ queryKey: ['agentUpdates'], queryFn: checkAgentUpdates, staleTime: 5 * 60 * 1000 });
  const openMutation = useMutation({ mutationFn: openDesktopAgent, onSuccess: () => queryClient.invalidateQueries({ queryKey: ['localAgents'] }) });
  const applyMutation = useMutation({
    mutationFn: ({ agentId, action }: { agentId: string; action: AgentMaintenanceAction }) => applyAgentMaintenance(agentId, action),
    onSuccess: async () => {
      setPlan(null);
      setActionError(null);
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: ['localAgents'] }),
        queryClient.invalidateQueries({ queryKey: ['agentUpdates'] }),
      ]);
    },
    onError: (error: Error) => setActionError(error.message),
  });
  const visible = useMemo(() => (agents.data || []).filter((agent) => {
    const matchesSurface = agent.surface === surface;
    const keyword = search.trim().toLocaleLowerCase();
    return matchesSurface && (!keyword || `${agent.product} ${agent.displayName}`.toLocaleLowerCase().includes(keyword));
  }), [agents.data, search, surface]);
  const counts = useMemo(() => ({ cli: (agents.data || []).filter((agent) => agent.surface === 'cli').length, desktop: (agents.data || []).filter((agent) => agent.surface === 'desktop').length }), [agents.data]);
  const updatesByAgent = useMemo(() => new Map((updates.data || []).map((update) => [update.agentId, update])), [updates.data]);

  const refresh = async () => {
    await agents.refetch();
    await updates.refetch();
  };
  const requestMaintenance = async (agentId: string, action: AgentMaintenanceAction) => {
    setActionError(null);
    try {
      setPlan(await getAgentMaintenancePlan(agentId, action));
    } catch (error) {
      setActionError(error instanceof Error ? error.message : '无法准备该操作，请重试。');
    }
  };

  if (agents.isLoading) return <PageState state="loading" label="正在检测本机 Agent..." />;
  if (agents.isError) return <PageState state="error" title="无法检测本机 Agent" description="请检查系统环境后重试。" onRetry={() => void agents.refetch()} />;
  return <div className="page-stack agents-page">
    <header className="page-header agents-page__header"><div><h1>Agents 管理</h1><span>发现、启动并维护本机 Agent。</span></div><button type="button" className="button button--secondary" onClick={() => void refresh()} disabled={agents.isFetching || updates.isFetching}><RefreshCw size={15} /> {agents.isFetching ? '检测中...' : '重新检测'}</button></header>
    <div className="agents-toolbar">
      <label className="agent-search"><Search size={15} /><input placeholder="搜索 Agent" value={search} onChange={(event) => setSearch(event.target.value)} aria-label="搜索 Agent" /></label>
      <div className="agents-tabs" role="tablist" aria-label="Agent 类型"><button type="button" role="tab" aria-selected={surface === 'cli'} className={surface === 'cli' ? 'is-active' : ''} onClick={() => setSurface('cli')}>命令行 CLI <span>{counts.cli}</span></button><button type="button" role="tab" aria-selected={surface === 'desktop'} className={surface === 'desktop' ? 'is-active' : ''} onClick={() => setSurface('desktop')}>桌面客户端 <span>{counts.desktop}</span></button></div>
    </div>
    {updates.isFetching && surface === 'cli' && <p className="agent-auto-check">正在自动检查可维护 CLI 的版本...</p>}
    <section className="agent-tile-grid" aria-label={surface === 'cli' ? '命令行 CLI' : '桌面客户端'}>{visible.map((agent) => <AgentTile key={agent.id} agent={agent} update={updatesByAgent.get(agent.id)} isFetching={updates.isFetching} onOpen={() => openMutation.mutate(agent.id)} onMaintain={(action) => void requestMaintenance(agent.id, action)} isOpening={openMutation.isPending && openMutation.variables === agent.id} />)}</section>
    {!visible.length && <PageState state="empty" title="没有匹配的 Agent" description="尝试使用不同的搜索词。" />}
    {actionError && !plan && <p className="project-agent-error">{actionError}</p>}
    {openMutation.isError && <p className="project-agent-error">无法打开桌面客户端，请重新检测或检查系统应用权限。</p>}
    {plan && <MaintenanceConfirmation plan={plan} isApplying={applyMutation.isPending} error={actionError} onClose={() => { setPlan(null); setActionError(null); }} onConfirm={() => applyMutation.mutate({ agentId: plan.agentId, action: plan.action })} />}
  </div>;
}
