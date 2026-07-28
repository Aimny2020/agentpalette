# Harness Template Management Design

> Harness templates use the AgentPalette user data root: `~/.agentpalette/`.

## Goal

Build the global Harness page as a local library of reusable templates for long-running AI work. A Harness template is a file package that gives an agent durable instructions, state, scope, verification, and session handoff artifacts.

This release focuses on creating, importing, editing, validating, and organizing templates. Project-level Harness selection, applying templates into projects, agent execution, and Skill/MCP binding remain separate work.

## Product Boundary

AgentPalette has two Harness surfaces:

- **Global Harness page**: manages reusable Harness templates.
- **Project Harness page**: a later surface that selects and applies one or more templates to a project.

The global page owns templates only. It must not imply that a template has already been applied to a project or is controlling an agent runtime.

## Harness Model

A Harness template is a directory-shaped file package:

```text
AGENTS.md
docs/
  harness.toml
  ...selected supporting files
```

Two files are hard requirements:

- `AGENTS.md`: the agent entrypoint and navigation page.
- `docs/harness.toml`: AgentPalette metadata and file manifest.

All other files are optional at creation and freely editable afterward. AgentPalette health checks are advisory: they warn about missing or invalid artifacts but never rewrite user-owned files.

### Five-Subsystem Standard

Every built-in preset is designed around the five subsystems needed for long-running work:

| Subsystem | Responsibility | Typical artifacts |
|---|---|---|
| Instructions | Startup path, non-negotiable rules, definition of done | `AGENTS.md` |
| State | Verified progress, active work, blockers, next step | `task-status.md`, `feature_list.json` |
| Verification | Acceptance conditions and evidence | `verification.md`, `quality-rubric.md` |
| Scope | Boundaries, risks, and decisions | `architecture.md`, `risk-rules.md`, `decision-record.md` |
| Lifecycle | Clean cross-session continuation | `session-handoff.md` |

## Identity and Manifest

Each template has a stable, system-generated ID. It is written to `docs/harness.toml`, used as the SQLite and IPC key, and used for the managed storage directory. Users do not type or edit the ID in the standard creation flow.

The display name and description remain user-editable. Renaming a template never changes its ID or directory. Duplicating a template always creates a new ID.

Example manifest:

```toml
id = "5c91f4d0-8da2-4d89-a469-fd2d8f1db0ad"
name = "Web Application Development"
version = "1.0.0"
description = "A long-running harness for verified feature development."
work_type = "code"
created_from_preset = "code-feature-development"
source = "local"

required_files = ["AGENTS.md", "docs/harness.toml"]

[[files]]
path = "docs/feature_list.json"
kind = "json"
standard = true
```

`work_type` and `created_from_preset` are immutable once a template is created. The user must create or duplicate a template to use a different category or preset. They may still edit, add, and delete all content files after creation.

## Work Types and Presets

The wizard exposes four stable work types:

- **Code Work**: implementation, fixes, refactors, tests, code review, and technical design.
- **Document Work**: professional reports, academic papers, and other evidence-based long-form deliverables.
- **Presentation Work**: deck-based briefings, narrative presentations, and speaker material.
- **Custom Work**: a minimal, unconstrained starting point.

Work types are categories for filtering and organization. The actual creation blueprint is the immutable built-in preset selected beneath a work type.

### Built-In Presets

Built-in presets are system-owned and read-only. Users do not create, edit, or delete presets. They create and own templates derived from them.

| Work type | Preset ID | Preset name | Default supporting files |
|---|---|---|---|
| Code Work | `code-feature-development` | Feature Development | `architecture.md`, `feature_list.json`, `task-status.md`, `session-handoff.md`, `verification.md`, `risk-rules.md` |
| Code Work | `code-review` | Code Review | `architecture.md`, `task-status.md`, `session-handoff.md`, `verification.md`, `risk-rules.md`, `review-rubric.md`, `review-findings.md` |
| Code Work | `code-technical-design` | Technical Design | `architecture.md`, `task-status.md`, `session-handoff.md`, `verification.md`, `risk-rules.md`, `decision-record.md` |
| Document Work | `document-professional-report` | Professional Report | `document-brief.md`, `outline.md`, `research-notes.md`, `evidence-matrix.md`, `quality-rubric.md`, `task-status.md`, `session-handoff.md`, `verification.md` |
| Document Work | `document-academic-paper` | Academic Paper | `research-question.md`, `paper-outline.md`, `literature-review.md`, `evidence-matrix.md`, `citation-register.md`, `quality-rubric.md`, `task-status.md`, `session-handoff.md`, `verification.md` |
| Presentation Work | `presentation-briefing` | Presentation Briefing | `presentation-brief.md`, `narrative-outline.md`, `slide-plan.md`, `speaker-notes.md`, `evidence-matrix.md`, `visual-direction.md`, `quality-rubric.md`, `task-status.md`, `session-handoff.md`, `verification.md` |

Custom Work has no preset. It starts with only `AGENTS.md` and `docs/harness.toml`, then lets the user select any standard file from the full library or leave the template minimal.

### File Responsibilities

- `task-status.md`: current verified state, active work, blockers, decisions, evidence, and next step.
- `session-handoff.md`: compact current-objective handoff for the next session.
- `feature_list.json`: machine-readable work items, dependencies, statuses, and completion evidence.
- `verification.md`: runnable or inspectable acceptance criteria and evidence requirements.
- `review-rubric.md`: review dimensions, severity/score rules, and verdict conditions.
- `review-findings.md`: individual findings, evidence, status, and required follow-up.
- `decision-record.md`: decision, context, alternatives, rationale, and consequences.
- `evidence-matrix.md`: claim-to-evidence-to-source mapping with confidence and open questions.
- `quality-rubric.md`: explicit quality dimensions and delivery threshold.
- `citation-register.md`: source, citation key, usage location, and citation check status.
- `presentation-brief.md`: audience, desired decision, constraints, and duration.
- `narrative-outline.md`: conclusion-first presentation story.
- `slide-plan.md`: per-slide purpose, core message, content, assets, and completion state.
- `speaker-notes.md`: presenter guidance and transitions.
- `visual-direction.md`: visual principles, brand constraints, and prohibited treatments.

## Preset Registry

The Rust backend owns the complete, versioned built-in preset registry. Each registry entry contains:

- Preset ID, work type, name, and description.
- Default file list and file kinds.
- Content skeleton for every generated supporting file.
- `AGENTS.md` generation rules and per-file references.

The frontend requests this registry through IPC and renders the wizard from it. It must not maintain a copied file list or content definition. This prevents the UI selection from drifting from the backend-generated template.

## Creation Flow

The primary creation path is a guided wizard:

1. Select a work type.
2. Select an available system preset, unless the user selected Custom Work.
3. Enter template name and description. The system generates the stable ID.
4. Review the preset-recommended supporting files. They are preselected but can be deselected.
5. Review the generated file tree.
6. Create the template and open it in the editor.

The only enforced files are `AGENTS.md` and `docs/harness.toml`. Deselecting a recommended file removes it from the generated package and from the initial `AGENTS.md` references.

### Generated AGENTS.md

AgentPalette generates a preset-specific `AGENTS.md` that remains short and acts as a routing document rather than a manual. It includes:

1. Startup order: which selected status, plan, and evidence files to read before work.
2. Work rules: one clear active work item and no unrelated scope expansion.
3. File navigation: explicit references to every generated supporting file and its purpose.
4. Definition of done: required verification and evidence conditions.
5. End-of-session routine: update `task-status.md`, `session-handoff.md`, risks, and next step when those files exist.

After creation, users can edit or delete `AGENTS.md` and any supporting file. Missing AGENTS references are warnings only.

## Import and Extraction

The first release supports:

- Local directory import.
- Creating a reusable template by extracting files from the current project.

Imports never mutate the source directory. An imported template keeps its discovered files but uses the user-selected work type and optional built-in preset provenance when supplied. If no preset applies, it is stored as Custom Work.

Archive, Git/URL, marketplace, remote update, and export support remain deferred.

## Editing and Health

The detail page remains a file editor first:

- Left: template list and file tree.
- Center: text editor for the selected file.
- Right: mutable metadata, health, and destructive actions.

The settings panel shows immutable `work_type` and `created_from_preset` as read-only provenance. Name, description, version, and all files remain editable. Users can create and delete arbitrary files after creation.

Health checks warn about:

- Missing `AGENTS.md` or `docs/harness.toml`.
- Invalid TOML or JSON.
- Manifest-required file absence.
- Missing references from `AGENTS.md` to files initially generated by the preset.

## Storage

Templates are stored as managed directories under the global AgentPalette data path:

```text
~/.agentpalette/harnesses/
  5c91f4d0-8da2-4d89-a469-fd2d8f1db0ad/
    AGENTS.md
    docs/
      harness.toml
      task-status.md
      session-handoff.md
```

SQLite indexes template ID, display name, description, work type, preset provenance, source metadata, timestamps, and optional UI-only state. The file package remains the source of truth for generated content.

## Backend and IPC Direction

The implementation adds a read-only preset query and changes creation to accept a preset selection rather than user-supplied ID and arbitrary initial content:

```rust
get_harness_presets() -> Vec<HarnessPreset>
create_harness_template(input: CreateHarnessTemplateInput) -> HarnessTemplate
```

The creation input contains `name`, `description`, `work_type`, optional `preset_id`, and the final selected supporting-file paths. The backend validates that the preset belongs to the selected work type, generates the ID, creates files from the registry skeletons, and persists the immutable provenance.

Existing read, write, add file, delete file, import, extraction, duplicate, and validation commands remain, updated to preserve the new manifest fields.

## Acceptance Criteria

- The wizard exposes exactly four work types.
- Code Work offers Feature Development, Code Review, and Technical Design.
- Document Work offers Professional Report and Academic Paper.
- Presentation Work offers Presentation Briefing.
- Custom Work offers no preset and a full standard-file library.
- Built-in preset definitions come only from the backend registry.
- The wizard does not ask the user to enter a template ID.
- Created manifest files contain a system-generated stable ID, immutable work type, and optional preset provenance.
- The created package always includes `AGENTS.md` and `docs/harness.toml`.
- The generated AGENTS file references every selected supporting file and includes startup, scope, verification, and handoff rules.
- Users may edit all generated content and add or delete files after creation.
- Users cannot change a template's work type or preset provenance after creation.
- Health checks remain advisory and do not rewrite user-owned files.
