# AgentPalette Foundation Implementation Plan

> The application identity is `com.lemon.agentpalette` and its database filename is `agentpalette.db`.

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a runnable Tauri v2 + React foundation with six navigable pages, the approved Vivid Precision shell, SQLite initialization, and a typed health-check IPC flow.

**Architecture:** Use a modular monolith. React feature modules call a narrow typed IPC client; Rust commands delegate to application services, which depend on domain ports implemented by infrastructure adapters. The foundation implements only health/database status while the remaining domain capabilities are represented by focused interfaces and UI empty states.

**Tech Stack:** Tauri v2, Rust 2021, React 19, TypeScript 5.8, Vite 7, React Router 7, TanStack Query 5, Zustand 5, Tailwind CSS 3, Lucide React, rusqlite, Vitest, Testing Library.

---

## File Map

- `package.json`, `vite.config.ts`, `tsconfig*.json`, `index.html`: frontend toolchain.
- `src/app/`: router, providers, and application shell.
- `src/features/`: dashboard, projects, skills, MCP, tasks, and settings pages.
- `src/shared/`: IPC client, shared types, UI primitives, mock state, and styles.
- `src-tauri/src/domain/`: stable entities, errors, and ports.
- `src-tauri/src/application/`: health-check use case and service container.
- `src-tauri/src/infrastructure/`: SQLite and platform adapters.
- `src-tauri/src/commands/`: thin Tauri command handlers.
- `src-tauri/migrations/`: ordered database schema.

### Task 1: Scaffold the React and Tauri workspace

**Files:**
- Create: `package.json`, `index.html`, `vite.config.ts`, `tsconfig.json`, `tsconfig.node.json`
- Create: `src/main.tsx`, `src/app/App.tsx`, `src/shared/styles/index.css`, `src/vite-env.d.ts`
- Create: `src-tauri/Cargo.toml`, `src-tauri/build.rs`, `src-tauri/tauri.conf.json`
- Create: `src-tauri/src/main.rs`, `src-tauri/src/lib.rs`

- [ ] **Step 1: Add frontend scripts and dependencies**

Define scripts `dev`, `build`, `test`, `test:run`, `lint`, `tauri`, and `tauri:dev`. Pin React 19, Vite 7, Tauri 2, React Router 7, TanStack Query 5, Zustand 5, Tailwind 3, Vitest and Testing Library.

- [ ] **Step 2: Add the minimal React entry point**

```tsx
import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import { App } from './app/App';
import './shared/styles/index.css';

createRoot(document.getElementById('root')!).render(
  <StrictMode><App /></StrictMode>,
);
```

Add a temporary `App` that renders the AgentPalette name and “Foundation loading” status. Task 4 replaces this placeholder with the routed application shell.

- [ ] **Step 3: Add the minimal Tauri crate and configuration**

Use product name `AgentPalette`, identifier `com.agentpalette.desktop`, 1280×800 default window, 960×640 minimum size, `beforeDevCommand: npm run dev`, and `beforeBuildCommand: npm run build`.

- [ ] **Step 4: Install dependencies and verify the empty builds**

Run: `npm install && npm run build && cargo check --manifest-path src-tauri/Cargo.toml`

Expected: npm install succeeds; Vite build and Cargo check exit 0.

- [ ] **Step 5: Commit**

```bash
git add package.json package-lock.json index.html vite.config.ts tsconfig*.json src src-tauri
git commit -m "chore: scaffold AgentPalette desktop workspace"
```

If Git is still uninitialized, skip only the commit command and record that fact.

### Task 2: Implement Rust health and database foundation with TDD

**Files:**
- Create: `src-tauri/migrations/001_initial.sql`
- Create: `src-tauri/src/domain/mod.rs`, `src-tauri/src/domain/error.rs`, `src-tauri/src/domain/health.rs`
- Create: `src-tauri/src/application/mod.rs`, `src-tauri/src/application/health_service.rs`
- Create: `src-tauri/src/infrastructure/mod.rs`, `src-tauri/src/infrastructure/database.rs`, `src-tauri/src/infrastructure/system.rs`
- Create: `src-tauri/src/commands/mod.rs`, `src-tauri/src/commands/health.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Write failing domain serialization and health-service tests**

```rust
#[test]
fn health_report_exposes_ready_database() {
    let report = HealthReport::new("0.1.0", "macos", DatabaseStatus::Ready);
    assert!(report.ready);
    assert_eq!(report.database, DatabaseStatus::Ready);
}
```

Add an application test using in-memory `SystemInfoPort` and `DatabasePort` fakes; assert version, platform, database status, and `ready == true`.

- [ ] **Step 2: Run the tests and verify failure**

Run: `cargo test --manifest-path src-tauri/Cargo.toml health`

Expected: FAIL because `HealthReport` and `HealthService` do not exist.

- [ ] **Step 3: Implement domain types and ports**

```rust
#[derive(Debug, Clone, serde::Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct HealthReport {
    pub version: String,
    pub platform: String,
    pub database: DatabaseStatus,
    pub ready: bool,
}

#[derive(Debug, Clone, serde::Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum DatabaseStatus { Ready, Unavailable }
```

Define `DatabasePort::status()` and `SystemInfoPort::{version, platform}`. Implement `HealthService` solely against those ports.

- [ ] **Step 4: Implement SQLite initialization and migration**

Create `projects`, `task_runs`, and `_migrations` tables in `001_initial.sql`. Open `${app_data_dir}/agentpalette.db`, enable foreign keys and WAL, and apply the migration once in a transaction.

- [ ] **Step 5: Implement stable IPC error mapping and command**

```rust
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandError { pub code: String, pub message: String, pub details: Option<String> }
```

Expose `health_check(state: State<AppState>) -> Result<HealthReport, CommandError>`; keep the handler free of database logic.

- [ ] **Step 6: Run Rust tests**

Run: `cargo test --manifest-path src-tauri/Cargo.toml`

Expected: all domain, application, migration, and error-mapping tests pass.

- [ ] **Step 7: Commit**

```bash
git add src-tauri
git commit -m "feat: add health and database foundation"
```

### Task 3: Add the typed frontend IPC boundary

**Files:**
- Create: `src/shared/api/types.ts`, `src/shared/api/tauriClient.ts`, `src/shared/api/tauriClient.test.ts`
- Create: `src/app/providers/AppProviders.tsx`

- [ ] **Step 1: Write failing IPC client tests**

```ts
it('returns a typed health report', async () => {
  invokeMock.mockResolvedValue({ version: '0.1.0', platform: 'macos', database: 'ready', ready: true });
  await expect(getHealth()).resolves.toMatchObject({ ready: true, database: 'ready' });
});
```

Also assert that rejected command errors become `AppError` with `code`, `message`, and optional `details`.

- [ ] **Step 2: Run the test and verify failure**

Run: `npm run test:run -- src/shared/api/tauriClient.test.ts`

Expected: FAIL because the client module does not exist.

- [ ] **Step 3: Implement types and client**

```ts
export type DatabaseStatus = 'ready' | 'unavailable';
export interface HealthReport { version: string; platform: string; database: DatabaseStatus; ready: boolean }
export class AppError extends Error { constructor(public code: string, message: string, public details?: string) { super(message); } }
```

Wrap `invoke<HealthReport>('health_check')` in `getHealth()` and normalize unknown rejections.

- [ ] **Step 4: Configure QueryClient and verify tests**

Use one retry for queries and disable refetch-on-focus. Run: `npm run test:run -- src/shared/api/tauriClient.test.ts`

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/shared/api src/app/providers
git commit -m "feat: add typed Tauri client"
```

### Task 4: Build the Vivid Precision application shell

**Files:**
- Create: `tailwind.config.ts`, `postcss.config.cjs`
- Create: `src/shared/styles/index.css`, `src/shared/styles/tokens.css`
- Create: `src/shared/ui/Button.tsx`, `src/shared/ui/Card.tsx`, `src/shared/ui/StatusBadge.tsx`
- Create: `src/app/layout/AppShell.tsx`, `src/app/layout/TopNavigation.tsx`, `src/app/layout/ProjectSidebar.tsx`
- Create: `src/app/layout/AppShell.test.tsx`
- Create: `src/app/router.tsx`, `src/app/App.tsx`

- [ ] **Step 1: Write the failing shell navigation test**

Render a memory router and assert visible links for `控制面板`, `项目管理`, `Skills`, `MCP`, `任务`, and `设置`, plus the selected project `AgentPalette Core`.

- [ ] **Step 2: Run the test and verify failure**

Run: `npm run test:run -- src/app/layout/AppShell.test.tsx`

Expected: FAIL because `AppShell` does not exist.

- [ ] **Step 3: Implement tokens and primitives**

Define semantic CSS variables for surface, text, outline, primary `#00e676`, danger, radii, and shadows. Configure Hanken Grotesk for headings, Inter for body text, 8px spacing, 16px cards, and pill controls.

- [ ] **Step 4: Implement responsive shell**

Use a 64px top navigation, 288px project sidebar on widths ≥1024px, collapsible project drawer below that breakpoint, and an outlet-based main region. Mark active navigation with both color and `aria-current="page"`.

- [ ] **Step 5: Run shell test and frontend build**

Run: `npm run test:run -- src/app/layout/AppShell.test.tsx && npm run build`

Expected: test passes and production build exits 0.

- [ ] **Step 6: Commit**

```bash
git add tailwind.config.ts postcss.config.cjs src/app src/shared/styles src/shared/ui
git commit -m "feat: build AgentPalette application shell"
```

### Task 5: Add six routed page modules and project tabs

**Files:**
- Create: `src/features/dashboard/DashboardPage.tsx`, `src/features/dashboard/DashboardPage.test.tsx`
- Create: `src/features/projects/ProjectsPage.tsx`, `src/features/projects/projectRoutes.tsx`
- Create: `src/features/projects/pages/ProjectOverview.tsx`, `HarnessPage.tsx`, `AgentsPage.tsx`, `EnvironmentPage.tsx`
- Create: `src/features/skills/SkillsPage.tsx`, `src/features/mcp/McpPage.tsx`
- Create: `src/features/tasks/TasksPage.tsx`, `src/features/settings/SettingsPage.tsx`
- Create: `src/shared/ui/PageState.tsx`, `src/shared/mocks/projects.ts`
- Modify: `src/app/router.tsx`

- [ ] **Step 1: Write failing route and state tests**

Assert every route renders its heading, project child routes render their tab labels, and `PageState` renders loading, empty, error with retry, and content states.

- [ ] **Step 2: Run tests and verify failure**

Run: `npm run test:run -- src/features src/shared/ui/PageState.test.tsx`

Expected: FAIL because route modules and `PageState` are absent.

- [ ] **Step 3: Implement reusable page states**

```tsx
type PageStateProps =
  | { state: 'loading'; label: string }
  | { state: 'empty'; title: string; description: string }
  | { state: 'error'; title: string; description: string; onRetry: () => void }
  | { state: 'content'; children: React.ReactNode };
```

Use accessible status roles and keyboard-focusable retry actions.

- [ ] **Step 4: Implement dashboard health flow and mock cards**

Fetch `getHealth()` with TanStack Query. Display version, platform, database status, three recent mock projects, and task summary. Map query loading/error/success into `PageState`.

- [ ] **Step 5: Implement remaining routes and project tabs**

Each page must have a descriptive heading, concise empty state, and one mock content section demonstrating the final layout without adding business actions.

- [ ] **Step 6: Run tests and build**

Run: `npm run test:run && npm run build`

Expected: all frontend tests pass and build exits 0.

- [ ] **Step 7: Commit**

```bash
git add src/features src/shared/mocks src/shared/ui/PageState.tsx src/app/router.tsx
git commit -m "feat: add foundation page routes"
```

### Task 6: Add theme persistence and foundation interfaces

**Files:**
- Create: `src/shared/theme/themeStore.ts`, `src/shared/theme/ThemeToggle.tsx`, `src/shared/theme/themeStore.test.ts`
- Create: `src-tauri/src/domain/ports.rs`, `src-tauri/src/domain/agent.rs`, `src-tauri/src/domain/task.rs`
- Modify: `src/app/layout/TopNavigation.tsx`, `src-tauri/src/domain/mod.rs`

- [ ] **Step 1: Write failing theme tests**

Assert default system preference, explicit light/dark selection, localStorage persistence, and root `data-theme` updates.

- [ ] **Step 2: Run tests and verify failure**

Run: `npm run test:run -- src/shared/theme/themeStore.test.ts`

Expected: FAIL because the theme store is absent.

- [ ] **Step 3: Implement theme behavior and toggle**

Use Zustand persistence under key `agentpalette-theme`; support `system`, `light`, and `dark`; expose an accessible three-state control in the top navigation.

- [ ] **Step 4: Add compile-only Rust port interfaces**

Define focused `AgentAdapter` methods for metadata and command construction, plus `ProcessManager` methods for spawn/status/terminate. Do not add implementations or dynamic plugin ABI.

- [ ] **Step 5: Run full unit checks**

Run: `npm run test:run && cargo test --manifest-path src-tauri/Cargo.toml`

Expected: all tests pass.

- [ ] **Step 6: Commit**

```bash
git add src/shared/theme src/app/layout/TopNavigation.tsx src-tauri/src/domain
git commit -m "feat: add themes and domain extension ports"
```

### Task 7: Document and verify the foundation

**Files:**
- Create: `README.md`, `.gitignore`, `AGENTS.md`
- Modify: `docs/superpowers/plans/2026-07-06-agentpalette-foundation.md`

- [ ] **Step 1: Document setup and architecture**

README must include prerequisites (Node LTS, Rust stable, Tauri system dependencies), `npm install`, `npm run tauri:dev`, test/build commands, the four-layer dependency rule, and the deliberate first-stage exclusions.

- [ ] **Step 2: Add repository guidance**

Create the requested concise contributor guide with structure, commands, TypeScript/Rust naming, test placement, Conventional Commits, PR screenshots for UI changes, and security rules for `.agentpalette` and secrets.

- [ ] **Step 3: Run formatters and static checks**

Run: `npm run lint && npm run build && cargo fmt --manifest-path src-tauri/Cargo.toml -- --check && cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings`

Expected: all commands exit 0 with no warnings.

- [ ] **Step 4: Run all tests**

Run: `npm run test:run && cargo test --manifest-path src-tauri/Cargo.toml`

Expected: all tests pass.

- [ ] **Step 5: Launch smoke test**

Run: `npm run tauri:dev`

Expected: AgentPalette opens at 1280×800; all routes navigate; theme persists; Dashboard reports ready database. Stop cleanly with Ctrl-C.

- [ ] **Step 6: Commit**

```bash
git add README.md .gitignore AGENTS.md docs package.json package-lock.json src src-tauri
git commit -m "docs: finalize AgentPalette foundation"
```

## Plan Self-Review

- Spec coverage: application shell, six pages, four page states, light/dark themes, typed IPC, SQLite, health flow, errors, tests, and explicit exclusions each map to a task.
- Scope: no project scanning, config writing, PTY implementation, or real Agent launch is introduced.
- Type consistency: Rust serializes `DatabaseStatus` as `ready | unavailable`; TypeScript uses the same values and camelCase fields.
