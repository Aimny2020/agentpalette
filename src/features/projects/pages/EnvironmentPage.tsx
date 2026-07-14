import { PageState } from '../../../shared/ui/PageState';

export function EnvironmentPage() {
  return (
    <div className="fixed-workspace-page project-placeholder-page">
      <PageState state="empty" title="环境检查尚未运行" description="后续阶段将在此展示 CLI、Git 与运行时检测结果。" />
    </div>
  );
}
