import { Card } from '../../shared/ui/Card';

export function SettingsPage() {
  return <div className="page-stack fixed-workspace-page settings-page-container"><header className="page-header"><div><p className="eyebrow">PREFERENCES</p><h1>设置</h1><p className="page-description">管理本机 Agent、数据安全、外观与更新。</p></div></header><div className="settings-grid"><Card><h2>Agent 检测</h2><p className="muted-copy">Claude Code、Codex、Gemini、OpenCode</p></Card><Card><h2>数据与安全</h2><p className="muted-copy">本地数据库与 Keychain 状态</p></Card></div></div>;
}
