# Project Harness Application Implementation Plan

> **For agentic workers:** Execute this plan task-by-task with focused test cycles.

**Goal:** Allow each project to apply one independent Harness template and manage its project-owned Harness files.

**Architecture:** Add a project-Harness domain/service boundary separate from the global template repository. Persist only project provenance and applied file hashes in SQLite; keep all project files on disk. Expose project commands through Tauri and render the project Harness page from the active project.

**Tech Stack:** Rust, Tauri v2, SQLite/rusqlite, React, TypeScript, TanStack Query, Vitest, Testing Library.

## Global Constraints

- One project has zero or one managed Harness.
- Applying a template copies the complete template file set and never binds the project to future template changes.
- Project file operations are limited to root `AGENTS.md` and `docs/**`.
- `AGENTS.md` and parseable `docs/harness.toml` are the minimum validity requirements.
- No direct replacement, automatic sync, automatic merge, or silent overwrite.
- Project unregistering never modifies physical project files.

### Task 1: Project Harness Domain and Persistence

**Files:**
- Create: `src-tauri/src/domain/project_harness.rs`
- Modify: `src-tauri/src/domain/mod.rs`, `src-tauri/src/domain/ports.rs`
- Create: `src-tauri/migrations/007_project_harnesses.sql`
- Modify: `src-tauri/src/infrastructure/database.rs`
- Test: `src-tauri/src/infrastructure/database.rs`

- [ ] Add serializable records for project Harness status, provenance, applied files, and application conflict entries.
- [ ] Add a `ProjectHarnessRepository` port for get/save/delete records and applied-file hashes.
- [ ] Add migration tables keyed by `project_id`, with cascade deletion from projects.
- [ ] Implement SQLite read/write/delete methods and verify migration plus round-trip persistence.

### Task 2: Project Harness Service

**Files:**
- Create: `src-tauri/src/application/project_harness_service.rs`
- Modify: `src-tauri/src/application/mod.rs`, `src-tauri/src/lib.rs`
- Test: `src-tauri/src/application/project_harness_service.rs`

- [ ] Add status detection for absent, unmanaged, managed, and invalid states.
- [ ] Validate all paths against `AGENTS.md` or `docs/**`.
- [ ] Build a complete application preview from a global template and project directory, including conflicts and missing AGENTS references.
- [ ] Apply via staging/backup, write `docs/harness.toml` as an instance manifest without template identity, persist hashes and provenance, and reject application when final references are missing.
- [ ] Add refresh/read/write/create/delete project file operations with disk-based content.
- [ ] Add unmanage and hash-aware file deletion operations.

### Task 3: Tauri Commands and TypeScript IPC

**Files:**
- Modify: `src-tauri/src/commands/harnesses.rs`, `src-tauri/src/commands/health.rs`, `src-tauri/src/lib.rs`
- Modify: `src/shared/api/types.ts`, `src/shared/api/tauriClient.ts`
- Test: `src/shared/api/tauriClient.test.ts`

- [ ] Expose project status, application preview, apply, file read/write/create/delete, refresh validation, and unmanage commands.
- [ ] Keep template IDs as source metadata only; project file APIs accept project IDs and constrained relative paths.
- [ ] Add typed client functions and invoke coverage.

### Task 4: Project Harness UI

**Files:**
- Modify: `src/features/projects/pages/HarnessPage.tsx`
- Create or modify: `src/features/projects/pages/project-harness.css`
- Test: `src/features/routes.test.tsx` or a colocated Harness page test

- [ ] Render absent/unmanaged/managed/invalid states for the active project.
- [ ] Add template-selection, conflict-review, and confirmation flow.
- [ ] Add file tree/editor limited to `AGENTS.md` and `docs/**`, refresh, validation, unmanage, and safe deletion review.
- [ ] Invalidate project Harness queries after every mutation.

### Task 5: Verification

- [ ] Run `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`.
- [ ] Run `cargo test --manifest-path src-tauri/Cargo.toml`.
- [ ] Run `npm run test:run`.
- [ ] Run `npm run build` and `npm run lint`.
- [ ] Run `git diff --check`.
