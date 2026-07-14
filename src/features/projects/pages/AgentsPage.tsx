import { PageState } from '../../../shared/ui/PageState';

export function AgentsPage() {
  return (
    <div className="fixed-workspace-page project-placeholder-page">
      <PageState state="empty" title="尚未配置 Agent" description="后续阶段将在此管理 Agent 启动配置。" />
    </div>
  );
}
