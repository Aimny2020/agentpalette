# Performance Optimization (Skills & Agents Load Latency) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Reduce Skills and Agents page load time from ~4-5 seconds to < 30ms for cached loads by eliminating blocking subprocess calls, implementing SQLite snapshot caching, thinning IPC payloads, and disabling auto-network checks on initial render.

**Architecture:** 
1. **Frontend Caching & Decoupling:** Configure React Query `staleTime: 5 min` and disable auto network check calls (`check_skill_updates`, `check_agent_updates`) on page mount.
2. **SQLite Snapshot & Payload Thinning:** Store scanned skill catalog metadata in SQLite. `get_skills` query returns thin `SkillSummary` (<10ms) directly from DB; full Markdown/HTML details are lazy-loaded on modal view via `get_skill_detail`.
3. **Background Indexer & Concurrency Control:** Perform disk scans and Git/CLI checks inside `tokio::task::spawn_blocking` background workers with concurrency limits and timeouts. Avoid redundant `git.exe` subprocess spawns on initial load.

**Tech Stack:** React 18, TypeScript, React Query (TanStack Query), Tauri v2 IPC, Rust (Tokio, SQLite, Serde).

---

## File Structure & Responsibilities

- **Frontend**:
  - Modify: [src/app/providers/AppProviders.tsx](file:///F:/00-chenkai-project/agentpalette/src/app/providers/AppProviders.tsx) (React Query client default staleTime)
  - Modify: [src/features/skills/SkillsPage.tsx](file:///F:/00-chenkai-project/agentpalette/src/features/skills/SkillsPage.tsx) (Disable auto update check on mount, adapt to SkillSummary & lazy detail loading)
  - Modify: [src/features/agents/AgentsPage.tsx](file:///F:/00-chenkai-project/agentpalette/src/features/agents/AgentsPage.tsx) (Disable auto npm update check on mount)
  - Modify: [src/shared/api/tauriClient.ts](file:///F:/00-chenkai-project/agentpalette/src/shared/api/tauriClient.ts) (Add IPC wrapper for `getSkillDetail`)

- **Rust Backend**:
  - Modify: [src-tauri/migrations/002_skills.sql](file:///F:/00-chenkai-project/agentpalette/src-tauri/migrations/002_skills.sql) or new migration for snapshot cache table
  - Modify: [src-tauri/src/domain/skill.rs](file:///F:/00-chenkai-project/agentpalette/src-tauri/src/domain/skill.rs) (Define `SkillSummary`)
  - Modify: [src-tauri/src/commands/skills.rs](file:///F:/00-chenkai-project/agentpalette/src-tauri/src/commands/skills.rs) (Expose `get_skills` as summaries & `get_skill_detail`)
  - Modify: [src-tauri/src/application/skill_service.rs](file:///F:/00-chenkai-project/agentpalette/src-tauri/src/application/skill_service.rs) (Implement DB snapshot caching, background scanner worker)
  - Modify: [src-tauri/src/application/agent_service.rs](file:///F:/00-chenkai-project/agentpalette/src-tauri/src/application/agent_service.rs) (Lightweight discovery without sync `--version` subprocess blocking)

---

## Tasks

### Task 1: Phase 1 Emergency Frontend Unblocking (Stop Auto Network & Re-fetch Triggers)

**Files:**
- Modify: `src/app/providers/AppProviders.tsx`
- Modify: `src/features/skills/SkillsPage.tsx`
- Modify: `src/features/agents/AgentsPage.tsx`

- [ ] **Step 1: Update AppProviders.tsx React Query default options**

Configure `staleTime` and `refetchOnWindowFocus` in `AppProviders.tsx`:

```tsx
const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 5 * 60 * 1000, // 5 minutes
      refetchOnWindowFocus: false,
      retry: 1,
    },
  },
});
```

- [ ] **Step 2: Disable auto update check on mount in SkillsPage.tsx**

Remove or comment out automatic `check_skill_updates()` in `useEffect` on page mount in `SkillsPage.tsx`, preserving explicit manual button click.

- [ ] **Step 3: Disable auto npm update check on mount in AgentsPage.tsx**

In `AgentsPage.tsx`, disable automatic `checkAgentUpdates()` execution on page mount, allowing user to manually trigger update checks.

- [ ] **Step 4: Verify frontend build and tests**

Run: `npm run build && npm run test:run`
Expected: TypeScript typecheck passes and tests succeed.

---

### Task 2: Phase 2 Database Snapshot Schema & Domain Models

**Files:**
- Modify: `src-tauri/src/domain/skill.rs`
- Create / Modify: SQLite database migration file if necessary for caching scanned skills.

- [ ] **Step 1: Define `SkillSummary` struct in domain/skill.rs**

Add a lightweight `SkillSummary` struct in `src-tauri/src/domain/skill.rs`:

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillSummary {
    pub id: String,
    pub kind: SkillKind,
    pub metadata: SkillMetadata,
    pub member_count: usize,
    pub category_id: Option<String>,
    pub user_notes: Option<String>,
    pub source: SkillSourceInfo,
    pub update_status: UpdateStatus,
    pub has_executable_content: bool,
    pub trusted: bool,
    pub warnings: Vec<String>,
    pub custom_description: Option<String>,
}
```

- [ ] **Step 2: Add repository methods for snapshot persistence**

Add methods in `SkillRepository` trait for saving and loading `SkillSummary` list from SQLite.

- [ ] **Step 3: Run Rust tests**

Run: `cargo test --manifest-path src-tauri/Cargo.toml`
Expected: All domain model tests pass.

---

### Task 3: Phase 2 Backend Snapshot Reading & Detail Command

**Files:**
- Modify: `src-tauri/src/commands/skills.rs`
- Modify: `src-tauri/src/application/skill_service.rs`
- Modify: `src/shared/api/tauriClient.ts`

- [ ] **Step 1: Implement instant DB read in `SkillService::get_skills`**

Modify `SkillService::get_skills(&self) -> DomainResult<Vec<SkillSummary>>` to read existing cache from SQLite DB if available, falling back to background scanner.

- [ ] **Step 2: Implement `get_skill_detail(skill_id)` command**

Add command `get_skill_detail` in `src-tauri/src/commands/skills.rs` to return full `Skill` (including `html_content` and member details) for a single skill ID.

- [ ] **Step 3: Update `src/shared/api/tauriClient.ts`**

Expose `getSkillDetail` in frontend Tauri client wrapper.

- [ ] **Step 4: Verify Rust backend build and test suite**

Run: `cargo test --manifest-path src-tauri/Cargo.toml`
Expected: Command and service tests pass.

---

### Task 4: Phase 3 Background Indexer & Agent Discovery Optimization

**Files:**
- Modify: `src-tauri/src/application/skill_service.rs`
- Modify: `src-tauri/src/application/agent_service.rs`

- [ ] **Step 1: Wrap disk scanning & Git status in `spawn_blocking` with Semaphore**

In `SkillService`, execute full disk scanning and Git worktree checking inside `tokio::task::spawn_blocking` worker threads with a concurrency limit (2-4 worker tasks).

- [ ] **Step 2: Optimize `AgentService::discover`**

In `src-tauri/src/application/agent_service.rs`, make initial `discover()` perform path checking (`find_executable`) only, skipping synchronous `--version` subprocess calls during initial discovery. Provide a background / on-demand version detection mechanism.

- [ ] **Step 3: Full end-to-end test verification**

Run: `npm run test:run && cargo test --manifest-path src-tauri/Cargo.toml`
Expected: Frontend and backend tests pass cleanly.

---

## Verification Plan

### Automated Verification
- Run `npm run build` to verify TypeScript types and bundle creation.
- Run `npm run test:run` to execute Vitest frontend tests.
- Run `cargo test --manifest-path src-tauri/Cargo.toml` to execute Rust backend tests.

### Manual Verification
1. Launch app with `npm run tauri:dev`.
2. Open Skills management page: verify immediate load without 4-5s spinner.
3. Open Agents page: verify immediate load without lag.
4. Click on a Skill item: verify full details load smoothly on demand.
