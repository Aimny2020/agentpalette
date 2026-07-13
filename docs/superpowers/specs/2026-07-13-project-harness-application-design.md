# Project Harness Application Design

**Status:** Confirmed

## Goal

Let a registered project apply exactly one Harness template as a local project
instance. Applying a template copies its complete Harness file set into the
project. From that point, the project owns those files: users can edit them in
AgentForge or externally, and later template edits never change the project.

This feature belongs to project management. The global Harness page continues
to manage reusable templates; the project Harness page manages only the
instance in the active project.

## Core Model

### One Project, One Harness

- A project has zero or one managed Harness instance.
- A project without an instance may remain unconfigured.
- A project cannot replace its current Harness directly. It must first delete
  the current Harness configuration, then apply another template.
- Applying a Harness is only available from the global Harness template
  library. The project page does not provide a from-scratch creation flow.

### Instantiation, Not Binding

Applying a template is a one-time materialization operation:

1. Read the template's complete current file set.
2. Resolve project-file conflicts explicitly.
3. Create project-local Harness files.
4. Record only local management metadata.

Afterward, the instance has no runtime dependency on the source template. A
user may edit, add, or delete project Harness files freely. Editing or deleting
the template never edits, regenerates, or invalidates the project instance.

The product may retain source metadata for display only. The first release does
not implement template-to-project synchronization or merge.

## Project File Boundary

The project Harness file boundary is deliberately narrow:

```text
<project-root>/
  AGENTS.md
  docs/
    harness.toml
    ...Harness-managed files and folders
```

The Harness editor may read and write only:

- `<project-root>/AGENTS.md`
- `<project-root>/docs/**`

It must not use Harness actions to edit source code, package configuration, or
arbitrary project paths. External editors remain unrestricted; AgentForge
always refreshes from disk instead of treating its editor state as canonical.

## Required Project Instance Files

An applied Harness includes every file currently in its source template. The
application flow does not offer a selective-copy control because a template is
a coherent instruction set and partial copying can break references in
`AGENTS.md`.

`AGENTS.md` and `docs/harness.toml` are the only minimum validity requirements
for a managed project Harness:

- `AGENTS.md` must exist in the project root.
- `docs/harness.toml` must exist and parse successfully.

All other files under `docs/` are project-owned and optional after application.
They may diverge from, be added to, or be deleted independently of the source
template. Health checks report minimum-file and TOML parsing issues but never
auto-repair project files.

`docs/harness.toml` is written as a project-instance configuration. It must not
retain template identity, template name, or template path. After application it
is editable like every other project Harness file.

## Application Flow

The unconfigured project Harness page has one primary action: **Apply Harness**.
It opens a three-step wizard.

### 1. Select Template

The user selects one global template. The list supports filtering by name, work
type, and template language. A template preview shows its complete file list
and the generated `AGENTS.md` content.

### 2. Resolve Conflicts

For each source file that already exists in the project, the user must choose:

- **Keep project file:** do not write the template file.
- **Overwrite with template file:** replace the project file.
- **Skip template file:** do not write the template file.

There is no silent overwrite and no automatic content merge in the first
release. The confirmation state computes the final file set. If the final
`AGENTS.md` references a missing `docs/` file, application is blocked until the
user corrects the choices.

### 3. Confirm Application

The final screen shows the files that will be created, overwritten, retained,
and skipped. Confirmation performs one transactional application operation.

## Transaction, Backup, and Undo

The backend stages the resolved result before modifying the project. It verifies
that target paths are writable, then applies all writes as one operation. A
write failure rolls back any files already changed by that operation.

When a project file is overwritten, AgentForge stores a restorable local backup
for that application operation. After a successful application, the project
page provides a one-time undo action. Undo restores backed-up overwritten files
and removes files newly created by that operation, subject to a final conflict
check when they have since changed.

Backups are local application data. They are not stored in the project and are
removed when the related project registration is deleted.

## Project Harness Editing

Once configured, the project Harness page provides:

- Instance status and minimum health result.
- Source information: template name when available, application time, and a
  clear statement that it is an independent project copy.
- A file tree limited to `AGENTS.md` and `docs/**`.
- Content editor for text files, file creation, and file deletion.
- Refresh from disk and explicit health validation.
- Delete Harness action.

External changes are visible after refresh or page reload. AgentForge must not
overwrite an externally changed file merely because its in-memory editor has an
older version.

## Source Metadata and Template Lifecycle

The application database stores project-level provenance separately from the
project files:

```text
project_id
source_template_id
source_template_content_hash_at_apply
applied_at
applied_files[] { path, applied_content_hash }
```

This data supports instance display, safe deletion defaults, backups, and a
future explicit synchronization feature. It does not alter Agent behavior and
does not make the template authoritative.

When the source template changes, the first release may display that it has
changed but offers no merge or sync action. When the source template is deleted,
the project remains valid and displays **Source template deleted**. The project
instance continues to be fully editable.

## Deleting a Project Harness

The project Harness page exposes two explicit destructive paths:

### Unmanage Harness

Delete the local Harness association, provenance, and backups only. Leave every
project file unchanged. If the project is later registered again, the product
can detect the existing on-disk Harness.

### Delete Harness Files

Show a file-by-file deletion review based on the recorded application manifest:

- Files created during application whose current content still matches the
  recorded hash are selected by default.
- Applied files that changed afterward are marked **modified** and are not
  selected by default.
- Files created after application are not selected by default.
- `AGENTS.md` and `docs/harness.toml` follow the same rule.

Deletion never silently removes modified or newly added files. After file
deletion, local management metadata is removed.

Deleting or unregistering a project from AgentForge always follows the
unmanage behavior: it removes application metadata and backups but never
changes the physical project directory.

## Existing Unmanaged Harness Detection

When a registered project has both a root `AGENTS.md` and a parseable
`docs/harness.toml`, but no local project-Harness record, show it as **Existing
unmanaged Harness detected** rather than as unconfigured.

The user can:

- **Manage existing Harness:** create a local association without modifying any
  project file. Its source is recorded as unknown.
- **Ignore:** leave the project unmanaged and make no file changes.

An unmanaged Harness cannot be silently replaced by applying a template. The
user must first choose an explicit delete or unmanage path as appropriate.

## Data and API Shape

Introduce project-specific domain models rather than reusing global template
models:

```text
ProjectHarnessRecord
  project_id
  source_template_id: optional
  source_template_hash: optional
  applied_at
  applied_files: ProjectHarnessAppliedFile[]
  managed_state: managed | unmanaged-adopted

ProjectHarnessAppliedFile
  path
  applied_content_hash
  created_by_application

ProjectHarnessStatus
  state: absent | unmanaged_detected | managed | invalid
  required_files_present
  manifest_parseable
  source_status: unknown | available | changed | deleted
```

The backend owns all filesystem writes, path validation, hashing, backup,
rollback, and disk refresh logic. The frontend only renders state and submits
the user's explicit application or deletion choices.

## First-Release Non-Goals

- Multiple Harness instances per project.
- Direct replacement of one project Harness by another.
- Creating a Harness from scratch inside a project.
- Automatic template synchronization, automatic upgrades, or content merge.
- Writing template identity or source metadata into project Harness files.
- Managing files outside root `AGENTS.md` and `docs/**`.
- Auto-repairing invalid project Harness files.

## Acceptance Criteria

- A project can have at most one managed Harness instance.
- Applying a template writes a project-local copy only after explicit conflict
  resolution and final confirmation.
- No template update or deletion changes any project Harness file.
- Project Harness editing supports `AGENTS.md` and `docs/**`; external edits
  are read from disk on refresh.
- Application is transactional, rolls back on write failure, and stores backup
  information for overwritten files.
- Invalid final AGENTS references block application, while subsequent user
  edits only result in health warnings.
- Project Harness deletion never silently deletes modified or newly created
  project files.
- Existing on-disk Harness files can be adopted without being rewritten.
- Project unregistering never modifies physical project files.
