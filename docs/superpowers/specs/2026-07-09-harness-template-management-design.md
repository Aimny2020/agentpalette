# Harness Template Management Design

## Goal

Build the global Harness page as a local template library for long-running AI project work. A Harness template is a reusable file package that defines how agents should understand, operate on, and verify work in a project.

This release focuses on creating, importing, editing, validating, and organizing Harness templates. Project-level Harness selection and applying templates into projects are separate follow-up work.

## Product Boundary

AgentForge has two Harness surfaces:

- **Global Harness page**: manages reusable Harness templates.
- **Project Harness page**: later selects which Harness template a project uses and writes the selected files into that project.

The first Harness release implements only the global template management page. It does not execute agents, bind Skills, configure MCP servers, or enforce runtime permissions.

## Harness Template Model

A Harness template is a directory-shaped file package. The root `AGENTS.md` is mandatory and acts as the main agent entrypoint. Supporting rule files live under `docs/`.

```text
AGENTS.md
docs/
  harness.toml
  architecture.md
  feature_list.json
  task-status.md
  verification.md
  risk-rules.md
  agent-profile.md
```

Required files:

- `AGENTS.md`: the primary instruction file agents must read first.
- `docs/harness.toml`: AgentForge metadata and file manifest for managing the template.

Standard optional files:

- `docs/architecture.md`: architecture boundaries, module responsibilities, and constraints.
- `docs/feature_list.json`: machine-readable feature list, status, priority, and ownership notes.
- `docs/task-status.md`: current task state, phase notes, known issues, and recent decisions.
- `docs/verification.md`: completion criteria, test commands, build checks, and manual review items.
- `docs/risk-rules.md`: risky operations, approval rules, blocked actions, and safety constraints.
- `docs/agent-profile.md`: preferred agent behavior, collaboration style, and tool/model guidance.

During template creation, AgentForge generates `AGENTS.md`, `docs/harness.toml`, and the selected optional files. After creation, users may freely edit every file, including `AGENTS.md` and generated references.

## Manifest

`docs/harness.toml` is required so AgentForge can identify, validate, display, and import templates reliably. It is software-managed during creation but editable after creation.

Example:

```toml
id = "code-work-standard"
name = "Code Work Standard"
version = "1.0.0"
description = "A standard harness for long-running coding work."
work_type = "code"
source = "local"

required_files = ["AGENTS.md", "docs/harness.toml"]

[[files]]
path = "docs/architecture.md"
kind = "markdown"
standard = true

[[files]]
path = "docs/feature_list.json"
kind = "json"
standard = true

[[files]]
path = "docs/verification.md"
kind = "markdown"
standard = true
```

If a local import contains `AGENTS.md` but no `docs/harness.toml`, AgentForge may generate a manifest after the user confirms name, description, and work type.

## Work Types

Creation starts from an AI work type, not a project type. A single project may later use multiple Harness templates, but each template should stay focused on one class of work.

Initial work types:

- **Code Work**: implementation, fixes, refactors, tests.
- **Documentation Work**: specs, design docs, knowledge base, long-form writing.
- **Presentation Work**: decks, briefings, narratives, slide content.
- **Review Work**: code review, plan review, document review.
- **Custom Work**: minimal starting point with only required files unless the user selects more.

Default optional file selections:

- Code Work: `architecture.md`, `feature_list.json`, `verification.md`, `risk-rules.md`.
- Documentation Work: `feature_list.json`, `task-status.md`, `verification.md`.
- Presentation Work: `feature_list.json`, `task-status.md`, `agent-profile.md`.
- Review Work: `architecture.md`, `verification.md`, `risk-rules.md`.
- Custom Work: no optional files selected by default.

The creation wizard exposes file purpose labels rather than raw filenames first, while still showing the target path.

## Creation Flow

The primary creation path is a guided wizard.

1. Select AI work type.
2. Enter template name and description.
3. Select optional standard files.
4. Review generated file tree.
5. Create template.
6. Open the template editor.

During creation, AgentForge maintains the `AGENTS.md` generated structure and references to selected supporting files. After creation, the template becomes user-owned and fully editable.

Users should not need to define custom filenames in the wizard. After creation, the editor allows arbitrary new files.

## Import Flow

The first release supports two import sources:

- **Import from local directory**
- **Create from current project**

Deferred import sources:

- Archive import.
- Git or URL import.
- Marketplace or remote template registry.
- Update checking.

Local directory import behavior:

- Inspect the selected directory.
- Require `AGENTS.md`.
- Detect `docs/harness.toml` if present.
- Detect standard optional files.
- List additional files.
- If `docs/harness.toml` is missing, offer to generate it.
- Confirm name, description, work type, and destination ID.
- Copy the directory into the global Harness template library.

Create-from-current-project behavior:

- Inspect the current project root for `AGENTS.md`.
- Inspect project `docs/` for standard Harness files.
- Let the user choose which discovered files to include.
- Generate `docs/harness.toml` if needed.
- Save a reusable copy into the global Harness template library.

Import should never mutate the source directory or current project.

## Editing Experience

The Harness template detail page is a file editor first and a settings page second.

Recommended layout:

- Left panel: template list or file tree.
- Center panel: text editor for the selected file.
- Right panel: metadata, health checks, and actions.

Editor behavior:

- Markdown files open as text, with preview optional in this release.
- JSON files validate syntax.
- TOML files validate syntax.
- Unknown file extensions open as plain text.
- Users can add, rename, delete, and edit files.
- Saves write back to the template directory.

Metadata panel:

- Name.
- Description.
- Work type.
- Version.
- Source type.
- File count.
- Last modified time.

Actions:

- Duplicate template.
- Export is deferred.
- Delete template with confirmation.
- Repair references is optional and may be deferred.

## Health Checks

Harness health checks are advisory. They warn users about template issues but do not forcibly rewrite user-edited files.

Checks:

- `AGENTS.md` exists.
- `docs/harness.toml` exists.
- `docs/harness.toml` parses as TOML.
- Manifest required files exist.
- Standard JSON files parse as JSON.
- Standard TOML files parse as TOML.
- `AGENTS.md` appears to reference selected standard supporting files.

Missing references in `AGENTS.md` should be warnings, not hard errors. The user may intentionally structure instructions differently after creation.

## Storage

Harness templates are stored as directories under a global AgentForge data path, for example:

```text
~/.agent-forge/harnesses/
  code-work-standard/
    AGENTS.md
    docs/
      harness.toml
      architecture.md
      verification.md
```

SQLite stores index and UI metadata:

- Template ID.
- Display name.
- Description.
- Work type.
- Source type.
- Source path for local imports when useful.
- Created time.
- Updated time.
- Favorite or pinned state if implemented.

The file package remains the source of truth for template content.

## Frontend Surface

Global Harness page:

- Empty state with `Create Harness` and `Import Harness`.
- Search by name and description.
- Filter by work type.
- Template cards or table rows showing name, work type, file count, source, and health state.
- Detail editor for selected template.

Creation modal or full-screen wizard:

- Work type cards.
- Metadata fields.
- Optional file checklist.
- Generated file preview.

Import modal:

- Import from local directory.
- Create from current project.
- Inspection result before import.

## Backend and IPC Direction

Future implementation should add a Harness domain model and thin Tauri commands around filesystem-backed operations.

Candidate commands:

```rust
get_harness_templates() -> Vec<HarnessTemplateSummary>
inspect_harness_import(source_path: String) -> HarnessImportInspection
import_harness_from_folder(source_path: String, options: HarnessImportOptions) -> HarnessTemplate
extract_harness_from_project(project_id: String, options: HarnessExtractOptions) -> HarnessTemplate
create_harness_template(input: CreateHarnessTemplateInput) -> HarnessTemplate
get_harness_template(template_id: String) -> HarnessTemplateDetail
read_harness_file(template_id: String, path: String) -> HarnessFile
write_harness_file(template_id: String, path: String, content: String) -> HarnessFile
create_harness_file(template_id: String, path: String, kind: String) -> HarnessFile
delete_harness_file(template_id: String, path: String) -> ()
delete_harness_template(template_id: String) -> ()
validate_harness_template(template_id: String) -> HarnessValidationReport
```

Commands should prevent path traversal and should only operate inside managed Harness template directories unless importing from an explicitly selected source.

## Deferred

- Applying a Harness template to a project.
- Project-level selection of active Harness.
- Writing `AGENTS.md` and `docs/` files into project repositories.
- Diff preview for project application.
- Agent runtime launch or task execution.
- Skills, MCP, or Agent profile binding.
- Git, URL, archive, and marketplace import.
- Remote updates and version synchronization.
- Runtime enforcement of permissions or verification commands.
- Markdown rich preview and collaborative editing.

## Acceptance Criteria

- Users can create a Harness template through the wizard.
- Created templates always include `AGENTS.md` and `docs/harness.toml`.
- Users can choose standard optional files during creation.
- Users can import a local directory containing `AGENTS.md`.
- Users can extract a template from the current project without mutating the project.
- Users can edit, add, delete, and save template files.
- JSON and TOML syntax errors are surfaced in the editor or health panel.
- The template list supports search and work-type filtering.
- Health checks identify missing required files and invalid manifest syntax.
- No first-release UI implies that templates are already applied to projects or used to run agents.
