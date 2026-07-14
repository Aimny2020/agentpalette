import React from 'react';
import { useQuery } from '@tanstack/react-query';
import { getProjectSkills } from '../../../shared/api/tauriClient';
import { useProjectStore } from '../../../shared/store/projectStore';
import { Card } from '../../../shared/ui/Card';
import { StatusBadge } from '../../../shared/ui/StatusBadge';

export function ProjectOverview() {
  const { activeProjectId } = useProjectStore();

  // Query enabled skills for the active project
  const { data: enabledSkillIds = [] } = useQuery({
    queryKey: ['projectSkills', activeProjectId],
    queryFn: () => getProjectSkills(activeProjectId || ''),
    enabled: !!activeProjectId,
  });

  return (
    <div className="content-grid fixed-workspace-page project-overview-page">
      <Card>
        <p className="eyebrow">PROJECT HEALTH</p>
        <h2>工程状态</h2>
        <div className="health-score">
          {activeProjectId ? '100' : '0'}
          <span>/100</span>
        </div>
        <StatusBadge tone={activeProjectId ? 'success' : 'neutral'}>
          {activeProjectId ? '配置一致' : '未选择项目'}
        </StatusBadge>
      </Card>
      <Card>
        <p className="eyebrow">FOUNDATION</p>
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
    </div>
  );
}
