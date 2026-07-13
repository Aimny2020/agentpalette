# Harness Language Localization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add immutable Simplified Chinese and English language selection to all Harness creation sources, with independent localized system content and language-aware validation.

**Architecture:** Persist a `language` field in the manifest and in in-memory summaries/details while retaining the manifest as the source of truth. Make the backend registry accept a language parameter and return localized content from explicit bilingual template definitions. The frontend passes one explicit language through create, import, and extraction flows; saving a manifest compares immutable fields with its existing on-disk value.

**Tech Stack:** Rust, serde/toml, Tauri v2, React 19, TypeScript, TanStack Query, Vitest, Rust unit tests.

## Global Constraints

- Support only `zh-CN` and `en`; default every creation flow to `zh-CN`.
- Keep paths, IDs, TOML/JSON keys, JSON enum values, commands, and code identifiers unchanged.
- Localize human-readable generated instructions, labels, descriptions, Markdown content, and JSON sample strings.
- Treat old manifests without `language` as `en` without rewriting them.
- Lock `language`, `work_type`, `created_from_preset`, and `selected_modules` when saving `docs/harness.toml`.
- Do not auto-detect language, auto-translate, or upgrade existing template content.

---

### Task 1: Add language data contracts and legacy defaulting

**Files:**
- Modify: `src-tauri/src/domain/harness.rs`
- Modify: `src/shared/api/types.ts`
- Test: `src-tauri/src/application/harness_service.rs`

- [ ] Add `HarnessLanguage` validation for `zh-CN` and `en`; add `language` to manifests, creation input, import options, extraction options, summaries, and details.
- [ ] Use `#[serde(default = "default_harness_language")]` so legacy manifests deserialize as `en`.
- [ ] Add a Rust test parsing a manifest without `language` and asserting `language == "en"`.

### Task 2: Localize the backend-owned template registry

**Files:**
- Modify: `src-tauri/src/domain/harness_presets.rs`
- Modify: `src-tauri/src/domain/harness.rs`
- Test: `src-tauri/src/domain/harness_presets.rs`

- [ ] Change preset/module/shared-file registry functions to accept `language: &str`.
- [ ] Add explicit English and Simplified Chinese text for every system preset, Code module, shared file, and generated AGENTS phrase.
- [ ] Keep file paths and module IDs identical across languages.
- [ ] Add parity tests asserting both languages expose the same paths and IDs, while Chinese content contains Chinese localized text.

### Task 3: Create, import, extract, and validate localized templates

**Files:**
- Modify: `src-tauri/src/application/harness_service.rs`
- Test: `src-tauri/src/application/harness_service.rs`

- [ ] Pass language into all registry calls and AGENTS generators.
- [ ] Persist the selected language in every newly created, imported, or extracted manifest and return it in summaries/details.
- [ ] Preserve a valid existing imported manifest language; when the source lacks it, use the language selected in the import wizard.
- [ ] Generate Chinese role headings and validate localized headings based on manifest language.
- [ ] Add tests creating Chinese Code and English Document templates, importing/extracting with language, and reading a legacy manifest.

### Task 4: Protect immutable manifest fields

**Files:**
- Modify: `src-tauri/src/application/harness_service.rs`
- Test: `src-tauri/src/application/harness_service.rs`

- [ ] When `write_harness_file` receives `docs/harness.toml`, parse the existing and proposed manifests.
- [ ] Reject a change to `language`, `work_type`, `created_from_preset`, or `selected_modules`; allow editable metadata and file-list changes.
- [ ] Add tests proving language changes are rejected and description changes are saved.

### Task 5: Add language selectors to all Harness flows

**Files:**
- Modify: `src/features/harness/components/CreateHarnessModal.tsx`
- Modify: `src/features/harness/components/ImportHarnessModal.tsx`
- Modify: `src/features/harness/GlobalHarnessPage.tsx`
- Modify: `src/features/harness/components/CreateHarnessModal.test.tsx`
- Modify: `src/shared/api/tauriClient.test.ts`

- [ ] Add a `zh-CN` default selector to create, folder import, and project extraction forms.
- [ ] Include the selected language in each API payload.
- [ ] Show the read-only language in the Harness detail metadata panel.
- [ ] Add tests for default Chinese payloads and selecting English.

### Task 6: Verify and review

**Files:**
- Modify: `docs/superpowers/specs/2026-07-13-harness-language-localization-design.md` only for implementation-discovered corrections.

- [ ] Run `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`.
- [ ] Run `cargo test --manifest-path src-tauri/Cargo.toml` and `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings`.
- [ ] Run `npm run test:run`, `npm run build`, and `npm run lint`.
- [ ] Run `git diff --check` and inspect the diff against the localization design.
