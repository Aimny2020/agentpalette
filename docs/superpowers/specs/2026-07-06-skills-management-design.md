# Skills Management Design Spec

> 全局 Skills 的持久化数据根目录为 `~/.agentpalette/`。

## 1. Goal and Scope

Skills Management 模块用于管理 AgentPalette 的全局可复用“技能库”（Capability Catalog）。技能用于配置、指导或扩展各种 Agent 的行为。本模块将实现：
- **全局文件存储**：所有技能存放在用户家目录的全局路径 `~/.agentpalette/skills/` 下，每个技能是一个独立的文件夹，内部包含 `SKILL.md`（内嵌 YAML Frontmatter 描述与正文说明）。
- **用户自定义数据持久化**：使用已有的 SQLite 数据库存储用户自定义的属性（包含技能所属分类、个人备注/使用说明）。
- **项目级启用机制**：技能的“启用/停用”状态属于项目级别，通过独立的 `project_skills` 关系表持久化。用户在项目详情页（Harness）中统一为当前项目启用或禁用技能。
- **完全自定义的分类**：支持用户显式新建、重命名、删除分类。新导入的技能默认属于“未分类”（Uncategorized）。
- **双重导入机制**：支持通过**选择本地文件夹**或**输入 Git 仓库链接**将技能导入至全局技能库。
- **物理删除**：删除技能时，物理删除对应的磁盘文件夹，并级联清理数据库记录。
- **平滑缩放详情弹窗**：点击技能卡片时，卡片以平滑的 Zoom 缩放动画展开为 3/4 屏幕的详情弹窗。
- **后端渲染 Markdown**：由 Rust 后端使用 `pulldown-cmark` 将 `SKILL.md` 渲染为安全的 HTML，前端直接渲染，减少前端打包体积。

本模块不涉及：
- 在应用内部直接创建空白技能。
- 在应用内部直接修改 `SKILL.md` 源码文件（仅支持修改用户个人备注及分类）。
- 与任何第三方社区平台的 API 绑定。

---

## 2. Data Architecture & Schema

### 2.1 物理文件结构
全局技能文件夹路径为：`~/.agentpalette/skills/`。
每个技能是一个子目录，子目录名称作为该技能的唯一 ID (`skill_id`)。

```text
~/.agentpalette/skills/
├── systematic-debugging/
│   ├── SKILL.md            # 必须包含，内有 YAML Frontmatter 和正文
│   └── scripts/            # 可选扩展文件夹
│       └── debug_helper.py
└── test-driven-development/
    └── SKILL.md
```

`SKILL.md` 文件格式模板：
```markdown
---
name: Systematic Debugging
description: Guidelines for fixing bugs systematically
author: Antigravity Team
version: 1.0.0
---

# Systematic Debugging

This skill provides step-by-step instructions for debugging code...
```

### 2.2 数据库表设计 (SQLite)
通过新增 `src-tauri/migrations/002_skills.sql` 建立以下表：

```sql
-- 分类表
CREATE TABLE IF NOT EXISTS categories (
  id TEXT PRIMARY KEY NOT NULL,
  name TEXT NOT NULL UNIQUE,
  created_at TEXT NOT NULL
);

-- 用户技能配置与备注表（移除了全局 is_enabled）
CREATE TABLE IF NOT EXISTS skills_user_meta (
  skill_id TEXT PRIMARY KEY NOT NULL,
  category_id TEXT,
  user_notes TEXT,
  updated_at TEXT NOT NULL,
  FOREIGN KEY (category_id) REFERENCES categories(id) ON DELETE SET NULL
);

-- 项目级技能启用表
CREATE TABLE IF NOT EXISTS project_skills (
  project_id TEXT NOT NULL,
  skill_id TEXT NOT NULL,
  enabled_at TEXT NOT NULL,
  PRIMARY KEY (project_id, skill_id),
  FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
  FOREIGN KEY (skill_id) REFERENCES skills_user_meta(skill_id) ON DELETE CASCADE
);
```

---

## 3. Rust Backend & Tauri IPC

### 3.1 领域模型 (Domain Types)
在 Rust 后端 `domain` 层定义：

```rust
pub struct SkillMetadata {
    pub name: String,
    pub description: String,
    pub author: Option<String>,
    pub version: Option<String>,
}

pub struct Skill {
    pub id: String,                      // 文件夹名称
    pub metadata: SkillMetadata,         // YAML 头部数据
    pub html_content: String,            // Markdown 编译后的 HTML 字符串
    pub category_id: Option<String>,     // 关联的用户分类 ID
    pub user_notes: Option<String>,      // 用户备注（作为技能的使用说明）
}

pub struct Category {
    pub id: String,
    pub name: String,
    pub created_at: String,
}
```

### 3.2 Tauri IPC Commands
前端调用的 Tauri Command 接口：

```rust
// 获取所有技能（扫描文件 + 查询 DB 合并）
#[tauri::command]
pub async fn get_skills(state: tauri::State<'_, AppState>) -> Result<Vec<Skill>, CommandError>;

// 导入技能（支持本地路径或 Git 链接）
#[tauri::command]
pub async fn import_skill(
    state: tauri::State<'_, AppState>,
    source: String,
    import_type: String, // "folder" | "git"
) -> Result<String, CommandError>;

// 删除技能（物理删除文件夹 + 清理 DB）
#[tauri::command]
pub async fn delete_skill(state: tauri::State<'_, AppState>, skill_id: String) -> Result<(), CommandError>;

// 更新技能用户数据（去掉了 is_enabled）
#[tauri::command]
pub async fn update_skill_meta(
    state: tauri::State<'_, AppState>,
    skill_id: String,
    category_id: Option<String>,
    user_notes: Option<String>,
) -> Result<(), CommandError>;

// 项目级启用接口
#[tauri::command]
pub async fn get_project_skills(state: tauri::State<'_, AppState>, project_id: String) -> Result<Vec<String>, CommandError>;

#[tauri::command]
pub async fn toggle_project_skill(
    state: tauri::State<'_, AppState>,
    project_id: String,
    skill_id: String,
    enabled: bool,
) -> Result<(), CommandError>;

// 分类管理
#[tauri::command]
pub async fn get_categories(state: tauri::State<'_, AppState>) -> Result<Vec<Category>, CommandError>;

#[tauri::command]
pub async fn create_category(state: tauri::State<'_, AppState>, name: String) -> Result<Category, CommandError>;

#[tauri::command]
pub async fn rename_category(state: tauri::State<'_, AppState>, id: String, name: String) -> Result<(), CommandError>;

#[tauri::command]
pub async fn delete_category(state: tauri::State<'_, AppState>, id: String) -> Result<(), CommandError>;
```

---

## 4. Frontend Architecture & UI Design

页面物理路径位于 `src/features/skills/`，拆分为以下组件：

- **`SkillsPage.tsx`**：主页面容器，使用 React Query 管理状态。
- **`SkillsSidebar.tsx`**：侧边栏分类展示。
  - “全部技能”、“未分类”为固定内置项。
  - 用户自定义分类列表，支持直接对分类进行重命名、删除。
  - 提供 “+ 新建分类” 按钮及弹窗。
- **`SkillsGrid.tsx`**：主要网格列表，具备搜索框（根据技能名称、描述、Tag 过滤）和“导入”按钮。
- **`SkillCard.tsx`**：卡片卡体。
  - 展示简短名称、描述、以及自定义分类标签。移去了全局启用/禁用开关。
  - 悬停触发上浮偏移与 `var(--color-primary)` 淡绿色外发光。
- **`SkillDetailModal.tsx`**：3/4 屏模态详情。
  - 采用平滑的 `scale` 缩放动画从点击处或中心放大。
  - 左侧/顶部：Markdown 编译的 HTML 内容滚动区域（只读，展示使用说明）。
  - 右侧/底部：交互设置。可直接修改下拉所属分类，文本框编辑“个人备注”（支持自动保存或点击保存）。移去了全局启用开关，使个人备注区域高度更大。
- **`ImportSkillModal.tsx`**：导入弹窗。
  - 提供单选/切换：“本地文件夹导入” (使用 Tauri `dialog` API 选择路径) 或 “Git 链接导入” (输入仓库 URL，Rust 后端执行 `git clone`)。
- **`HarnessPage.tsx`**：项目工程配置子页面。
  - 统一显示当前项目的工程规则。
  - 提供已导入技能的列表（附带 Checkbox 开关），让用户为当前项目启用或禁用特定的全局技能，状态持久化到 SQLite `project_skills` 中。

---

## 5. Visual System & Interactions

- **卡片动画**：
  ```css
  .skill-card {
    transition: transform 0.22s cubic-bezier(0.16, 1, 0.3, 1), border-color 0.22s ease, box-shadow 0.22s ease;
  }
  .skill-card:hover {
    transform: translateY(-4px);
    border-color: var(--color-primary);
    box-shadow: 0 8px 24px rgba(0, 230, 118, 0.15);
  }
  ```
- **Modal 动画**：
  点击卡片时，通过 React 状态控制 Modal 渲染，使用 `@keyframes` 触发 ZoomIn 缩放：
  ```css
  @keyframes modalZoom {
    from {
      opacity: 0;
      transform: scale(0.95);
    }
    to {
      opacity: 1;
      transform: scale(1);
    }
  }
  .modal-overlay {
    backdrop-filter: blur(8px);
    transition: opacity 0.3s ease;
  }
  .modal-body {
    animation: modalZoom 0.25s cubic-bezier(0.34, 1.56, 0.64, 1) forwards;
  }
  ```

---

## 6. Verification & Acceptance

### 6.1 自动化测试
- **Rust 单元测试**：
  - 测试 YAML 解析器是否能准确读取 Frontmatter。
  - 测试 Markdown 转 HTML 解析器，确保输出安全的 HTML 字符串。
  - 测试 SQLite 数据库迁移，并测试新增/删除/更新 SQL 语句。
- **React 单元测试**：
  - 测试 `SkillsPage` 搜索过滤、分类切换逻辑。
  - 测试 `HarnessPage` 中勾选技能开启/停用状态同步到数据库的逻辑。

### 6.2 手动验证
- 创建包含 `SKILL.md` 的本地文件夹，测试通过 Local Folder 导入，验证是否能成功复制至全局 `~/.agentpalette/skills/` 目录下。
- 输入一个公开的 Git Skill 仓库，测试通过 Git 链接克隆导入。
- 在页面中新建分类，将技能拖入（或下拉选择）到新分类，并在侧边栏中切换分类确认过滤正常。
- 物理删除技能，确认 `~/.agentpalette/skills/` 下该文件夹被完全删除，数据库元数据也被清空。
