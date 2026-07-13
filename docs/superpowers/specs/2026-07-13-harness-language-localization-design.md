# Harness Language Localization Design

**Status:** Confirmed

## Goal

Allow a user to create a Harness in Simplified Chinese or English without changing its stable file paths, module identifiers, structured-data schema, or project integration behavior.

This design adds two independently maintained system template languages. It does not translate a Harness at runtime and does not add project-level Harness application or execution behavior.

## Supported Languages

The first release supports exactly two language identifiers:

| ID | UI label | Default |
| --- | --- | --- |
| `zh-CN` | Simplified Chinese | Yes |
| `en` | English | No |

The create, local-directory import, and project-extraction flows show an explicit language selector. The product must not infer the language from the operating-system locale, project language, or imported file contents.

## What Language Controls

`language` controls only system-generated, human-readable content:

- Generated `AGENTS.md` content and headings.
- Generated Markdown templates under `docs/`.
- Human-readable sample values and descriptions in JSON templates.
- Display labels and descriptions returned by the system template registry.
- Health-check expectations for generated role headings and file references.

The following remain stable and are never translated:

- File paths such as `AGENTS.md`, `docs/harness.toml`, and `docs/feature_list.json`.
- Harness work types, module IDs, preset IDs, and database IDs.
- TOML keys, JSON keys, JSON enum values, command lines, code identifiers, URLs, and external tool parameters.

For example, a Chinese Code Work template may contain Chinese headings and instructions, while `selected_modules = ["feature-development"]` and the JSON key `"status": "not-started"` remain unchanged.

## Template Registry

The system owns two reviewed versions of each supported preset, Code Work module, shared Code file, and generated AGENTS document. They are equal in behavior and differ only in localized human-readable text.

English remains the structural reference. Simplified Chinese templates are maintained as a first-class system template set using `/Users/lemon/learn-harness-engineering` as a content and methodology reference, especially its Chinese resources for instructions, state, verification, scope, and session continuity.

Runtime machine translation is prohibited. A registry change must update both language variants in the same change and add or update parity tests for file paths, module IDs, and required behavioral sections.

## Creation Rules

All work types support language selection:

- **Code Work:** the chosen language generates the combined AGENTS document, shared Code files, and selected module files.
- **Document Work and Presentation Work:** the chosen language selects the corresponding language variant of the one selected system preset.
- **Custom Work:** the chosen language generates the minimal AGENTS document and localized contents for any selected standard files.

`AGENTS.md` and `docs/harness.toml` remain required. Other `docs/` files remain optional and are selected by default according to the existing work-type rules.

The language selector defaults to `zh-CN` and is independent of the application UI language.

## Manifest and Immutability

Every newly created, imported, or extracted Harness manifest records its selected language:

```toml
id = "5c91f4d0-8da2-4d89-a469-fd2d8f1db0ad"
name = "Full Software Delivery Harness"
work_type = "code"
language = "zh-CN"
selected_modules = ["technical-design", "feature-development"]
source = "local"
```

`language` is immutable after creation, alongside `work_type`, `created_from_preset`, and `selected_modules`. The Harness file editor still allows content editing, but the backend rejects a write to `docs/harness.toml` when any of those immutable fields changes. It allows edits to the name, description, version, required-file list, and standard-file list.

Changing language requires creating or duplicating a Harness, then intentionally migrating any user-authored changes. This protects user edits and prevents an English and Chinese instruction set from being mixed automatically.

## Import and Extraction

The import and extraction wizards require an explicit `zh-CN` or `en` selection. The selected value is written to the resulting manifest, but the product does not translate, rewrite, or normalize the imported file content.

Existing manifests without a `language` field remain readable. They are treated as `en` for display and validation only; no automatic migration or file rewrite occurs.

## Code Work and Health Checks

Code Work keeps its existing module semantics:

- Module IDs and canonical ordering remain `technical-design`, `feature-development`, `code-review`.
- `AGENTS.md` role headings and instructions are localized for the selected language.
- File paths linked by AGENTS remain stable.
- Health checks validate the localized role headings and selected dedicated file references corresponding to the manifest language.

Health checks remain advisory. User edits to AGENTS or supporting files may produce a warning but do not block editing or mark the template invalid unless an existing required-file or syntax rule fails.

## Preset Versioning

System template updates affect only new Harness instances. Existing templates are never automatically regenerated, translated, or upgraded because their files are user-editable.

The system records the template/preset version used at creation for traceability. A later template version is informational; adopting it requires creating or duplicating a template.

## Acceptance Criteria

- The create, import, and extraction flows offer only Simplified Chinese and English and default to Simplified Chinese.
- A Chinese and English instance from the same work type have identical stable file paths, manifest schema, work type, module IDs, preset IDs, and JSON keys/enums.
- Their generated human-readable content is respectively Chinese and English without runtime translation.
- Code Work health validation recognizes the selected language's expected role headings.
- A missing `language` field behaves as `en` without rewriting the existing template.
- Attempts to alter `language`, `work_type`, `created_from_preset`, or `selected_modules` through `docs/harness.toml` are rejected.
- System registry changes are covered by bilingual parity tests and do not alter existing Harness instances.

## Out of Scope

- Traditional Chinese or additional languages.
- Automatic language detection.
- Runtime machine translation.
- Automatic upgrades or translation of existing Harness files.
- Translating file paths, schemas, commands, code identifiers, or integration IDs.
- Project-level application, runtime execution, Skills/MCP binding, or automatic template upgrades.
