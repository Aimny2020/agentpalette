# 技能自定义说明隐藏功能说明书 (Hidden Features Record)

由于界面交互优化，我们隐藏了技能目录页面中关于自定义技能说明备份与维护的三个 UI 按钮（“导出说明”、“导入说明”、“清理未关联”）。

但这三个功能的底层逻辑在 Rust 核心与前端 API 客户端中均保持完整可用。

---

## 接口说明与触发方式

如果您日后需要使用这些功能，可以直接通过以下两种方式之一进行调用或二次接入：

### 1. 编程方式调用 (Frontend API)

前端客户端文件 [tauriClient.ts](file:///Users/lemon/Technology/00-chenkai-project/agent-forge/src/shared/api/tauriClient.ts) 中保留了以下已封装的方法：

```typescript
// 1. 备份：触发系统保存文件对话框，将所有自定义技能说明导出为 pretty JSON 文件
export async function exportCustomDescriptions(): Promise<string | null>;

// 2. 导入：触发系统选择文件对话框读取 JSON 备份，并返回冲突与有效性校验的预览信息
export async function previewCustomDescriptionsImport(): Promise<DescriptionsImportPreview | null>;

// 3. 确认导入：将上述校验后的有效记录写入数据库，支持 'keep_newer' | 'keep_local' | 'keep_import' 冲突处理策略
export async function confirmCustomDescriptionsImport(
  records: SkillDescriptionRecord[],
  conflictStrategy: 'keep_newer' | 'keep_local' | 'keep_import'
): Promise<void>;

// 4. 查询未关联数：计算数据库中已存在但当前未安装任何 Skill 的说明条数
export async function getUnassociatedDescriptionsCount(): Promise<number>;

// 5. 一键清理：清除那些已被彻底卸载的 Skill 残留的说明，返回成功删除的条数
export async function clearUnassociatedDescriptions(): Promise<number>;
```

### 2. Tauri IPC 直接触发 (Backend Commands)

后端的 Rust 逻辑声明在 [skills.rs](file:///Users/lemon/Technology/00-chenkai-project/agent-forge/src-tauri/src/commands/skills.rs)，已通过 `tauri::generate_handler` 注册到应用底座中。
可以直接使用 Tauri 的 `invoke` 机制调用：

```javascript
import { invoke } from '@tauri-apps/api/core';

// 导出备份
const savedPath = await invoke('export_custom_descriptions');

// 导入预览
const preview = await invoke('preview_custom_descriptions_import');

// 确认合并
await invoke('confirm_custom_descriptions_import', {
  records: preview.valid_records,
  conflictStrategy: 'keep_newer'
});

// 获取未关联残留数据量
const count = await invoke('get_unassociated_descriptions_count');

// 清理未关联残留数据
const deletedCount = await invoke('clear_unassociated_descriptions');
```

---

## 关联文件清单 (References)
- **后端命令实现**: [skills.rs](file:///Users/lemon/Technology/00-chenkai-project/agent-forge/src-tauri/src/commands/skills.rs)
- **前端 API 封装**: [tauriClient.ts](file:///Users/lemon/Technology/00-chenkai-project/agent-forge/src/shared/api/tauriClient.ts)
- **迁移数据库定义**: [004_skill_descriptions.sql](file:///Users/lemon/Technology/00-chenkai-project/agent-forge/src-tauri/migrations/004_skill_descriptions.sql)
