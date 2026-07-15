import { useMutation, useQuery } from '@tanstack/react-query';
import { Link } from 'react-router-dom';
import { AppWindow, CircleAlert, ExternalLink, TerminalSquare } from 'lucide-react';
import { getLocalAgents, getProjectSkills, launchAgent, openDesktopAgent } from '../../../shared/api/tauriClient';
import { useProjectStore } from '../../../shared/store/projectStore';
import { Card } from '../../../shared/ui/Card';

export function ProjectOverview() {
  const { activeProjectId } = useProjectStore();

  // Query enabled skills for the active project
  const { data: enabledSkillIds = [] } = useQuery({
    queryKey: ['projectSkills', activeProjectId],
    queryFn: () => getProjectSkills(activeProjectId || ''),
    enabled: !!activeProjectId,
  });
  const agents = useQuery({ queryKey: ['localAgents'], queryFn: getLocalAgents });
  const launchMutation = useMutation({ mutationFn: (agentId: string) => launchAgent(activeProjectId || '', agentId) });
  const openDesktopMutation = useMutation({ mutationFn: openDesktopAgent });
  const availableAgents = (agents.data || []).filter((agent) => agent.status === 'ready');

  return (
    <div className="content-grid fixed-workspace-page project-overview-page">
      <Card>
        <h2>配置摘要</h2>
        <dl className="definition-list">
          <div>
            <dt>Harness</dt>
            <dd>0</dd>
          </div>
          <div>
            <dt>Agents</dt>
            <dd>0</dd>
          </div>
          <div>
            <dt>Skills</dt>
            <dd>{activeProjectId ? enabledSkillIds.length : 0}</dd>
          </div>
          <div>
            <dt>MCP</dt>
            <dd>0</dd>
          </div>
        </dl>
      </Card>
      <Card className="project-launch-card">
        <h2>在此项目中启动 Agent</h2>
        {!activeProjectId ? <p className="muted-copy">选择项目后，可直接在该项目目录启动本机 Agent。</p> : agents.isLoading ? <p className="muted-copy">正在检测本机 Agent...</p> : availableAgents.length ? <div className="project-agent-list">
          {availableAgents.map((agent) => <div className="project-agent-row" key={agent.id}>
            <div><strong>{agent.displayName}</strong><small>{agent.version || (agent.surface === 'desktop' ? '桌面应用' : '命令行 CLI')}</small></div>
            {agent.surface === 'cli' ? <button type="button" className="button button--primary project-agent-row__launch" disabled={launchMutation.isPending} onClick={() => launchMutation.mutate(agent.id)}><ExternalLink size={15} /> 打开</button> : <button type="button" className="button button--primary project-agent-row__launch" disabled={openDesktopMutation.isPending} onClick={() => openDesktopMutation.mutate(agent.id)}><AppWindow size={15} /> 打开</button>}
          </div>)}
        </div> : <p className="muted-copy">未发现可启动的 Agent。请前往 Agents 管理页安装或检测。</p>}
        {agents.isError && <p className="project-agent-error"><CircleAlert size={15} /> 无法检测本机 Agent，请重试。</p>}
        {launchMutation.isError && <p className="project-agent-error"><CircleAlert size={15} /> {launchMutation.error instanceof Error ? launchMutation.error.message : '无法启动 Agent，请检查启动偏好。'}</p>}
        {openDesktopMutation.isError && <p className="project-agent-error"><CircleAlert size={15} /> 无法打开桌面应用，请重新检测或检查系统权限。</p>}
        {launchMutation.isSuccess && <p className="project-agent-success">已交给终端启动；项目目录不会被复制。</p>}
        <Link className="button button--secondary project-launch-card__action" to="/settings"><TerminalSquare size={16} /> 配置平台启动偏好</Link>
      </Card>
    </div>
  );
}
