# Harness Preset Registry Implementation Plan

> Harness templates use the AgentPalette user data root: `~/.agentpalette/`.

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the five ad-hoc Harness work types with four stable categories and a backend-owned, read-only registry of six built-in creation presets.

**Architecture:** Rust owns the preset registry, generated file skeletons, UUID generation, manifest provenance, and creation validation. React fetches preset definitions through IPC and uses them to drive the wizard; it contains no copied preset or default-file data. Templates stay user-owned after creation, while `work_type` and `created_from_preset` remain immutable provenance.

**Tech Stack:** Rust, Tauri v2 commands, serde/toml, rusqlite migrations, React 19, TypeScript, TanStack Query, Vitest, Testing Library.

## Global Constraints

- Work types are exactly `code`, `document`, `presentation`, and `custom`.
- Built-in preset IDs are `code-feature-development`, `code-review`, `code-technical-design`, `document-professional-report`, `document-academic-paper`, and `presentation-briefing`.
- `AGENTS.md` and `docs/harness.toml` are the only hard-required files.
- `work_type` and `created_from_preset` are immutable after creation; all template content files remain editable, addable, and deletable.
- The normal creation, import, extraction, and duplicate flows generate IDs in Rust; the UI must not request an ID.
- New template IDs use UUID v4 and are directory names beneath `~/.agentpalette/harnesses/`.
- Health checks remain advisory and must not modify user files.

---

### Task 1: Add domain types and a tested built-in preset registry

**Files:**
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/src/domain/harness.rs`
- Create: `src-tauri/src/domain/harness_presets.rs`
- Modify: `src-tauri/src/domain/mod.rs`
- Test: `src-tauri/src/domain/harness_presets.rs`

**Interfaces:**
- Produces `HarnessPreset`, `HarnessPresetFile`, `built_in_harness_presets() -> Vec<HarnessPreset>`, `find_harness_preset(id: &str) -> Option<HarnessPreset>`.
- Produces `HarnessManifest.created_from_preset: Option<String>` and `CreateHarnessTemplateInput { name, description, work_type, preset_id, optional_files }`.
- Consumed by `HarnessService` in Task 3 and `get_harness_presets` in Task 4.

- [ ] **Step 1: Write failing domain tests for the preset catalogue**

Add tests in `src-tauri/src/domain/harness_presets.rs` asserting the registry has six IDs, each preset belongs to one of the three non-custom work types, and each preset includes the expected distinguishing files:

```rust
#[test]
fn built_in_presets_cover_the_confirmed_workflows() {
    let presets = built_in_harness_presets();

    assert_eq!(presets.len(), 6);
    assert!(find_harness_preset("code-feature-development")
        .unwrap()
        .files
        .iter()
        .any(|file| file.path == "docs/feature_list.json"));
    assert!(find_harness_preset("document-academic-paper")
        .unwrap()
        .files
        .iter()
        .any(|file| file.path == "docs/citation-register.md"));
    assert!(find_harness_preset("presentation-briefing")
        .unwrap()
        .files
        .iter()
        .any(|file| file.path == "docs/slide-plan.md"));
}
```

- [ ] **Step 2: Run the domain test to verify RED**

Run: `cargo test --manifest-path src-tauri/Cargo.toml domain::harness_presets::tests::built_in_presets_cover_the_confirmed_workflows`

Expected: compilation fails because `harness_presets` and its registry functions do not exist.

- [ ] **Step 3: Implement the registry and contract types**

Add direct `uuid` dependency to `src-tauri/Cargo.toml`:

```toml
uuid = { version = "1", features = ["v4"] }
```

Define serializable camelCase types in `domain/harness.rs`:

```rust
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct HarnessPresetFile {
    pub path: String,
    pub kind: String,
    pub label: String,
    pub content: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct HarnessPreset {
    pub id: String,
    pub work_type: String,
    pub name: String,
    pub description: String,
    pub files: Vec<HarnessPresetFile>,
}
```

Add `created_from_preset: Option<String>` with `#[serde(default)]` to `HarnessManifest`; remove `id` from `CreateHarnessTemplateInput` and add `preset_id: Option<String>`. Define `harness_presets.rs` with the six catalogue entries and complete Markdown/JSON skeletons, including `task-status.md`, `session-handoff.md`, `verification.md`, and preset-specific files. Build `AGENTS.md` later in the service so it can reflect files deselected in the wizard.

- [ ] **Step 4: Run the domain test to verify GREEN**

Run: `cargo test --manifest-path src-tauri/Cargo.toml domain::harness_presets::tests::built_in_presets_cover_the_confirmed_workflows`

Expected: PASS.

- [ ] **Step 5: Add a regression test for shared lifecycle files**

Add a test that every non-custom preset includes both `docs/task-status.md` and `docs/session-handoff.md`.

- [ ] **Step 6: Run the expanded domain preset tests**

Run: `cargo test --manifest-path src-tauri/Cargo.toml domain::harness_presets::tests`

Expected: PASS.

### Task 2: Persist and expose immutable preset provenance

**Files:**
- Create: `src-tauri/migrations/006_harness_template_preset.sql`
- Modify: `src-tauri/src/infrastructure/database.rs`
- Modify: `src-tauri/src/domain/harness.rs`
- Modify: `src-tauri/src/domain/ports.rs`
- Test: `src-tauri/src/infrastructure/database.rs`

**Interfaces:**
- Adds nullable `created_from_preset` to the `harness_templates` index.
- Extends `HarnessTemplateSummary` and `HarnessTemplateDetail` with `created_from_preset: Option<String>`.
- Existing import rows remain valid with `NULL` provenance.

- [ ] **Step 1: Write failing database persistence test**

Extend `saves_and_loads_harness_templates` to create a summary containing:

```rust
created_from_preset: Some("code-feature-development".into()),
```

and assert the loaded record preserves that field.

- [ ] **Step 2: Run the database test to verify RED**

Run: `cargo test --manifest-path src-tauri/Cargo.toml infrastructure::database::tests::saves_and_loads_harness_templates`

Expected: compilation fails because the summary does not yet expose `created_from_preset`.

- [ ] **Step 3: Implement migration and repository mapping**

Create migration `006_harness_template_preset.sql`:

```sql
ALTER TABLE harness_templates ADD COLUMN created_from_preset TEXT;
INSERT OR IGNORE INTO _migrations (version) VALUES (6);
```

Load it after migration 005 in `SqliteDatabase::initialize`. Update `HarnessRepository` implementations and SQL `SELECT`/`INSERT ... ON CONFLICT` statements to include `created_from_preset`. Include the value in summary/detail domain types while retaining `Option<String>` for imported or old templates.

- [ ] **Step 4: Run the database test to verify GREEN**

Run: `cargo test --manifest-path src-tauri/Cargo.toml infrastructure::database::tests::saves_and_loads_harness_templates`

Expected: PASS.

- [ ] **Step 5: Run all database tests**

Run: `cargo test --manifest-path src-tauri/Cargo.toml infrastructure::database::tests`

Expected: PASS.

### Task 3: Generate templates exclusively from the registry

**Files:**
- Modify: `src-tauri/src/application/harness_service.rs`
- Test: `src-tauri/src/application/harness_service.rs`

**Interfaces:**
- `HarnessService::get_harness_presets() -> Vec<HarnessPreset>` returns the immutable registry.
- `HarnessService::create_harness_template(input)` validates the type/preset relationship, creates a UUID v4 ID, writes selected registry skeletons, and returns template detail.
- `HarnessService::duplicate_harness_template(template_id, target_name)` creates a new UUID and preserves provenance.
- `HarnessImportOptions` and `HarnessExtractOptions` no longer accept IDs; they accept `preset_id: Option<String>` and receive an ID from the service.

- [ ] **Step 1: Write failing creation tests for generated identity and provenance**

Replace the current creation test input with a code preset input and add assertions:

```rust
let input = CreateHarnessTemplateInput {
    name: "Test Harness".into(),
    description: "Harness description".into(),
    work_type: "code".into(),
    preset_id: Some("code-feature-development".into()),
    optional_files: vec![
        "docs/feature_list.json".into(),
        "docs/task-status.md".into(),
        "docs/session-handoff.md".into(),
    ],
};

let detail = service.create_harness_template(input).unwrap();
assert!(uuid::Uuid::parse_str(&detail.id).is_ok());
assert_eq!(detail.created_from_preset.as_deref(), Some("code-feature-development"));
```

Read `AGENTS.md` and assert it names selected files but not a deselected preset file such as `docs/risk-rules.md`.

- [ ] **Step 2: Run the creation test to verify RED**

Run: `cargo test --manifest-path src-tauri/Cargo.toml application::harness_service::tests::test_create_and_get_harness_template`

Expected: compilation fails because creation still requires `input.id` and does not return provenance.

- [ ] **Step 3: Implement preset resolution and generation**

Replace the current file-name `match` blocks in `create_harness_template` with these rules:

1. Reject unsupported work types.
2. For non-custom work types, require a known preset whose `work_type` equals the input work type.
3. For Custom Work, require `preset_id == None` and allow any selected path from the flattened registry file library.
4. Reject selected paths absent from the chosen preset or standard file library.
5. Generate `Uuid::new_v4().to_string()` before creating the directory.
6. Write only selected supporting files using their registry skeletons.
7. Build `AGENTS.md` from the selected files with startup, WIP=1 scope, file navigation, completion evidence, and end-of-session sections.
8. Write `created_from_preset` to the manifest and index summary.

Extract small private helpers such as `validate_creation_input`, `selected_preset_files`, `generate_agents_content`, and `new_template_id` so creation, import, extraction, and duplication do not duplicate rules.

- [ ] **Step 4: Run the creation test to verify GREEN**

Run: `cargo test --manifest-path src-tauri/Cargo.toml application::harness_service::tests::test_create_and_get_harness_template`

Expected: PASS.

- [ ] **Step 5: Add failure tests for immutable creation provenance**

Add one test that rejects `work_type: "document"` with `preset_id: Some("code-review")`, and one that rejects Custom Work with a preset ID. Assert error text identifies the invalid preset/work type combination.

- [ ] **Step 6: Add duplicate and import/extraction coverage**

Update duplication to generate a UUID automatically, keep the source manifest's work type and provenance, and update its test. Update import/extraction tests to verify generated IDs and `created_from_preset: None` when no preset is chosen.

- [ ] **Step 7: Run all Harness service tests**

Run: `cargo test --manifest-path src-tauri/Cargo.toml application::harness_service::tests`

Expected: PASS.

### Task 4: Add preset-query IPC and synchronize shared API contracts

**Files:**
- Modify: `src-tauri/src/commands/harnesses.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src/shared/api/types.ts`
- Modify: `src/shared/api/tauriClient.ts`
- Test: `src/shared/api/tauriClient.test.ts`

**Interfaces:**
- Adds `get_harness_presets() -> Vec<HarnessPreset>` Tauri command.
- Adds TypeScript `HarnessPreset`, `HarnessPresetFile`, `HarnessTemplateSummary.createdFromPreset`, and `HarnessTemplateDetail.createdFromPreset`.
- Changes creation options to omit `id` and duplicate API to accept only `(templateId, targetName)`.

- [ ] **Step 1: Write failing Tauri client test for the registry invocation**

Add a test that mocks `invoke`, calls `getHarnessPresets()`, and expects:

```ts
expect(invoke).toHaveBeenCalledWith('get_harness_presets');
```

- [ ] **Step 2: Run the client test to verify RED**

Run: `npm run test:run -- src/shared/api/tauriClient.test.ts`

Expected: TypeScript or test failure because `getHarnessPresets` is absent.

- [ ] **Step 3: Implement the command and frontend client contract**

Expose `get_harness_presets` through `commands/harnesses.rs` and register it in `lib.rs`. Add matching TypeScript types and a `getHarnessPresets` wrapper. Update creation, import, extraction, and duplication wrappers so callers cannot provide an ID. Update the existing Tauri client tests for changed invoke payloads.

- [ ] **Step 4: Run the client test to verify GREEN**

Run: `npm run test:run -- src/shared/api/tauriClient.test.ts`

Expected: PASS.

- [ ] **Step 5: Run Rust command compilation checks**

Run: `cargo test --manifest-path src-tauri/Cargo.toml commands::harnesses --no-fail-fast`

Expected: PASS or zero matching tests with successful compilation.

### Task 5: Rebuild the creation wizard around work type and system presets

**Files:**
- Modify: `src/features/harness/components/CreateHarnessModal.tsx`
- Modify: `src/features/harness/GlobalHarnessPage.tsx`
- Modify: `src/features/harness/harness.css`
- Create: `src/features/harness/components/CreateHarnessModal.test.tsx`

**Interfaces:**
- `CreateHarnessModal` accepts `presets: HarnessPreset[]`, `isPresetsLoading`, `onClose`, and `onCreate`.
- Its submit payload is `CreateHarnessTemplateInput` with no user-provided ID.
- `GlobalHarnessPage` fetches presets once with TanStack Query and supplies them to the wizard.

- [ ] **Step 1: Write failing wizard tests**

Create `CreateHarnessModal.test.tsx` using a small fixture registry. Test that:

```tsx
await user.click(screen.getByRole('button', { name: /Code Work/i }));
expect(screen.getByRole('button', { name: /Feature Development/i })).toBeVisible();
expect(screen.getByRole('button', { name: /Code Review/i })).toBeVisible();

await user.click(screen.getByRole('button', { name: /Feature Development/i }));
await user.click(screen.getByRole('button', { name: /continue/i }));
// Fill name and complete the wizard.
expect(onCreate).toHaveBeenCalledWith(expect.objectContaining({
  workType: 'code',
  presetId: 'code-feature-development',
  optionalFiles: expect.arrayContaining(['docs/feature_list.json']),
}));
expect(onCreate.mock.calls[0][0]).not.toHaveProperty('id');
```

Add a second test that Custom Work bypasses preset selection and presents the full standard file library.

- [ ] **Step 2: Run the wizard tests to verify RED**

Run: `npm run test:run -- src/features/harness/components/CreateHarnessModal.test.tsx`

Expected: FAIL because the old wizard has five type cards, an ID field, and no preset input.

- [ ] **Step 3: Implement the data-driven wizard**

Replace the local `WORK_TYPES` and `OPTIONAL_FILES` constants with values derived from `presets`:

1. Step 1 displays four work-type cards only.
2. Step 2 displays preset cards for Code, Document, and Presentation; Custom Work skips this step.
3. Metadata collects only name and description.
4. File selection prechecks the selected preset's files; Custom Work presents the deduplicated full registry file library.
5. Preview displays `AGENTS.md`, `docs/harness.toml`, and selected files.
6. Submit sends the selected work type, optional preset ID, and selected file paths.

Use accessible buttons for selectable cards and standard checkbox labels. Keep the existing modal visual language, but make progress indicators reflect the conditional preset step.

In `GlobalHarnessPage`, add `useQuery({ queryKey: ['harness-presets'], queryFn: getHarnessPresets })`, pass its result to the modal, and disable or show a loading state until it resolves. Update `getWorkTypeLabel`, filters, badges, and copy from `documentation`/`review` to `document`; remove all legacy category choices.

- [ ] **Step 4: Run the wizard tests to verify GREEN**

Run: `npm run test:run -- src/features/harness/components/CreateHarnessModal.test.tsx`

Expected: PASS.

- [ ] **Step 5: Add an editor provenance presentation test**

Extend the page/component test coverage so a template detail with `createdFromPreset: 'code-review'` renders the read-only preset label and work type, while no control exists to mutate either field.

- [ ] **Step 6: Run relevant frontend tests**

Run: `npm run test:run -- src/features/harness/components/CreateHarnessModal.test.tsx src/features/routes.test.tsx src/shared/api/tauriClient.test.ts`

Expected: PASS.

### Task 6: Remove manual IDs from import, extraction, and duplication UX

**Files:**
- Modify: `src/features/harness/components/ImportHarnessModal.tsx`
- Modify: `src/features/harness/GlobalHarnessPage.tsx`
- Modify: `src/shared/api/types.ts`
- Test: `src/features/harness/components/ImportHarnessModal.test.tsx`

**Interfaces:**
- Import/extraction options carry name, description, work type, and optional preset ID only.
- Duplicate action accepts a name only; Rust creates the new ID.
- Import type choices use `code`, `document`, `presentation`, and `custom`.

- [ ] **Step 1: Write failing import modal test**

Add a test that walks local-directory import and asserts the confirmation payload has no `id` while its work type selector exposes `Code Work`, `Document Work`, `Presentation Work`, and `Custom Work`.

- [ ] **Step 2: Run the import modal test to verify RED**

Run: `npm run test:run -- src/features/harness/components/ImportHarnessModal.test.tsx`

Expected: FAIL because the current modal collects an ID and still displays legacy work types.

- [ ] **Step 3: Implement generated-ID UX**

Remove ID inputs and validation from import and extraction. Use the four canonical category values. Do not offer built-in preset provenance for imports in this iteration; send `presetId: undefined`, so imported files are never falsely described as generated from a system preset.

Replace the duplicate prompt pair with one prompt for the new display name, calling the changed client API without a target ID. Keep the resulting generated ID visible in the template detail header as read-only metadata.

- [ ] **Step 4: Run the import modal test to verify GREEN**

Run: `npm run test:run -- src/features/harness/components/ImportHarnessModal.test.tsx`

Expected: PASS.

- [ ] **Step 5: Run all Harness frontend tests**

Run: `npm run test:run -- src/features/harness`

Expected: PASS.

### Task 7: Verify the complete feature and update the design evidence

**Files:**
- Modify: `docs/superpowers/specs/2026-07-09-harness-template-management-design.md` only if implementation reveals a necessary decision correction.

**Interfaces:**
- No new interface; this task verifies Tasks 1-6 against the accepted specification.

- [ ] **Step 1: Format Rust**

Run: `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`

Expected: PASS. If it reports formatting changes, run `cargo fmt --manifest-path src-tauri/Cargo.toml`, then rerun the check.

- [ ] **Step 2: Run the full Rust test suite**

Run: `cargo test --manifest-path src-tauri/Cargo.toml`

Expected: PASS.

- [ ] **Step 3: Run Rust linting**

Run: `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings`

Expected: PASS.

- [ ] **Step 4: Run full frontend tests and build**

Run: `npm run test:run && npm run build && npm run lint`

Expected: all commands exit 0.

- [ ] **Step 5: Verify requirements from the accepted spec**

Check the implementation manually against these cases:

1. Code Work exposes exactly three presets; Document Work exposes two; Presentation Work exposes one; Custom Work exposes none.
2. The normal create flow never requests an ID and produces a UUID v4 directory/manifest ID.
3. A deselected file is absent from both the generated package and generated AGENTS references.
4. All selected files have meaningful preset skeletons, including `task-status.md` and `session-handoff.md`.
5. Created type and preset provenance are visible but not editable; files remain editable/addable/deletable.
6. Import/extraction/duplicate flows generate new IDs and use only canonical work-type values.

- [ ] **Step 6: Inspect final diff and commit**

Run: `git diff --check && git status --short`

Expected: no whitespace errors and only intended Harness, API, migration, test, and documentation files changed.

Commit after the user approves the final diff:

```bash
git add src src-tauri docs/superpowers
git commit -m "feat: add harness creation presets"
```
