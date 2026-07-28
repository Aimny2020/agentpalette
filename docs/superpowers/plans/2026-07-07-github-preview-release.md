# AgentPalette GitHub Preview Release Implementation Plan

> The Tauri identifier is `com.lemon.agentpalette`, matching the AgentPalette product name.

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Prepare AgentPalette 0.1.0 for private, tag-triggered GitHub Draft Releases that build macOS ARM64/x64 DMGs and a Windows x64 NSIS installer.

**Architecture:** Keep product metadata in the three existing manifests and enforce consistency with one zero-dependency Node script. Separate ordinary CI from release packaging, and use Tauri platform override files for platform-specific bundle targets and macOS ad-hoc signing. Documentation describes the private-preview and unsigned-installer constraints.

**Tech Stack:** Tauri v2, Rust stable, React 19, TypeScript 5.8, Node.js 20, GitHub Actions, `tauri-apps/tauri-action`.

## Global Constraints

- Version remains exactly `0.1.0`; release tag is exactly `v0.1.0`.
- Product identifier is exactly `com.lemon.agentpalette`; author is exactly `Aimny2020`.
- Build only macOS ARM64, macOS x64, and Windows x64.
- Produce only macOS DMG and Windows NSIS `.exe` installers.
- macOS uses ad-hoc signing; Windows remains unsigned.
- Do not add updater, notarization, commercial signing, Linux, MSI, mobile, or LICENSE support.
- `.agents/` remains ignored and is not bundled.
- Do not commit, push, create tags, or trigger a release.

---

### Task 1: Version consistency guard

**Files:**
- Create: `scripts/check-version.mjs`
- Create: `scripts/check-version.test.mjs`
- Modify: `package.json`

**Interfaces:**
- Produces: `readVersions(rootDir) -> { packageVersion, cargoVersion, tauriVersion }`
- Produces: `validateVersions(versions, tag?) -> string`, returning the canonical version or throwing an `Error`.
- Produces: `npm run version:check`, using optional `RELEASE_TAG`.

- [ ] **Step 1: Write failing Node tests**

Create table-driven tests with `node:test` and `assert/strict`. Use temporary fixture directories containing minimal `package.json`, `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json`. Assert success for three `0.1.0` values and `v0.1.0`; assert throws for a Cargo mismatch and `v0.2.0`.

- [ ] **Step 2: Verify RED**

Run: `node --test scripts/check-version.test.mjs`

Expected: FAIL because `scripts/check-version.mjs` does not exist.

- [ ] **Step 3: Implement the minimal validator**

The module must parse JSON with `JSON.parse`, extract Cargo's first `[package]` `version = "..."`, compare all three values, validate tags against `/^v\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$/`, and expose a CLI that reads `process.env.RELEASE_TAG`.

- [ ] **Step 4: Wire npm scripts**

Add these keys without changing existing commands:

```json
"test:version": "node --test scripts/check-version.test.mjs",
"version:check": "node scripts/check-version.mjs"
```

- [ ] **Step 5: Verify GREEN**

Run: `npm run test:version && npm run version:check && RELEASE_TAG=v0.1.0 npm run version:check`

Expected: tests pass and both checks print `Version 0.1.0 is consistent.`

### Task 2: Release-safe metadata and Tauri configuration

**Files:**
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/tauri.conf.json`
- Create: `src-tauri/tauri.macos.conf.json`
- Create: `src-tauri/tauri.windows.conf.json`
- Modify: `src-tauri/src/application/skill_service.rs`

**Interfaces:**
- Consumes: version guard from Task 1.
- Produces: common Tauri config with explicit CSP and platform override configs selecting `dmg` or `nsis`.

- [ ] **Step 1: Fix the existing Rust formatter failure**

Run: `cargo fmt --manifest-path src-tauri/Cargo.toml`

Expected: whitespace and wrapping changes only in `skill_service.rs`.

- [ ] **Step 2: Update stable package metadata**

Set Cargo authors to `Aimny2020`; set Tauri identifier to `com.lemon.agentpalette`. Replace common bundle target `all` with an empty platform-neutral target list only if accepted by the schema; otherwise omit `targets` and make every release command pass an explicit `--bundles` value.

- [ ] **Step 3: Configure explicit CSP**

Set the policy to:

```text
default-src 'self'; connect-src ipc: http://ipc.localhost; img-src 'self' asset: http://asset.localhost data:; style-src 'self' 'unsafe-inline'; font-src 'self'; script-src 'self'
```

This permits Tauri IPC and bundled/data images while rejecting external network content.

- [ ] **Step 4: Add platform overrides**

Create a macOS override whose `bundle.targets` is `["dmg"]` and `bundle.macOS.signingIdentity` is `"-"`. Create a Windows override whose `bundle.targets` is `["nsis"]`.

- [ ] **Step 5: Validate configuration and formatting**

Run: `npm run version:check && cargo fmt --manifest-path src-tauri/Cargo.toml -- --check && npm run tauri -- build --bundles app`

Expected: metadata check and formatting pass; a macOS `.app` bundle is produced without invoking the known-failing local DMG script.

### Task 3: GitHub CI and Draft Release pipelines

**Files:**
- Create: `.github/workflows/ci.yml`
- Create: `.github/workflows/release.yml`

**Interfaces:**
- Consumes: `npm run version:check`, existing frontend/Rust commands, Tauri platform configs.
- Produces: read-only CI on push/PR and tag-triggered Draft Release with `contents: write`.

- [ ] **Step 1: Add CI workflow**

Use `actions/checkout@v4`, `actions/setup-node@v4` with Node 20 and npm cache, and `dtolnay/rust-toolchain@stable` with `rustfmt` and `clippy`. On Ubuntu install `libwebkit2gtk-4.1-dev`, `libappindicator3-dev`, `librsvg2-dev`, and `patchelf`; run `npm ci`, all frontend checks, version check, Cargo fmt/test/clippy.

- [ ] **Step 2: Add release preflight job**

Trigger on tags `v*.*.*`, set `RELEASE_TAG: ${{ github.ref_name }}`, and run the same version/quality checks. Export no user-managed secrets.

- [ ] **Step 3: Add release build matrix**

Make the matrix depend on preflight and include the exact three platform/target/bundle combinations. Use `tauri-apps/tauri-action@v1`, `releaseDraft: true`, `prerelease: true`, `uploadUpdaterJson: false`, `tagName: v__VERSION__`, and target-specific `args: --target ... --bundles dmg|nsis`.

- [ ] **Step 4: Statically validate YAML semantics**

Load both files with an available YAML parser and assert the CI triggers, release tag trigger, three matrix entries, `releaseDraft`, `uploadUpdaterJson`, permissions, and exact bundle args. If no local YAML parser is installed, parse with Ruby's standard `Psych` library.

Expected: both files parse and all assertions succeed.

### Task 4: Repository hygiene and release documentation

**Files:**
- Modify: `.gitignore`
- Modify: `README.md`
- Create: `CHANGELOG.md`
- Preserve deletion: `stitch_agentpalette_project_manager (1)/DESIGN.md`
- Preserve deletion: `stitch_agentpalette_project_manager (1)/code.html`
- Preserve deletion: `stitch_agentpalette_project_manager (1)/screen.png`

**Interfaces:**
- Consumes: exact release behavior established by Tasks 1–3.
- Produces: contributor-facing setup/release instructions and private-preview warnings.

- [ ] **Step 1: Extend ignore rules narrowly**

Add `.idea/`, `.vscode/`, `coverage/`, `*.log`, `*.p12`, `*.pfx`, `*.cer`, `*.key`, and `*.mobileprovision`. Keep `.agents/`, `.env*`, databases, build outputs, and lockfiles handled as specified; do not ignore `.github/` or `Cargo.lock`.

- [ ] **Step 2: Rewrite README in Chinese**

Document private-preview status, current skill management functionality, architecture, prerequisites, development/testing, supported packages, the `v0.1.0` Draft Release workflow, ad-hoc/unsigned installation warnings, disabled updater, and security rules. Link to `CHANGELOG.md` and the Tauri prerequisite guide.

- [ ] **Step 3: Add changelog**

Create Keep a Changelog sections `Unreleased` and `0.1.0 - 2026-07-07`. Record the application shell, SQLite/domain architecture, skill scanning/import/detail/delete flows, theme, typed IPC, tests, and preview distribution limitations.

- [ ] **Step 4: Verify repository hygiene**

Run: `git status --short --ignored` and `git check-ignore -v .agents/ .env dist src-tauri/target`

Expected: build/local directories are ignored, `.github/workflows/*.yml` and both lockfiles are not ignored, and the three old design files remain recorded as deletions.

### Task 5: Full verification and handoff

**Files:**
- Review all changed files from Tasks 1–4.

**Interfaces:**
- Consumes: every prior task.
- Produces: fresh evidence and a reviewable, uncommitted working tree.

- [ ] **Step 1: Run complete frontend and version verification**

Run: `npm run test:version && npm run version:check && npm run lint && npm run test:run && npm run build`

Expected: all commands exit 0 and Vitest reports all test files passing.

- [ ] **Step 2: Run complete Rust verification**

Run: `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check && cargo test --manifest-path src-tauri/Cargo.toml && cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings`

Expected: all commands exit 0, all Rust tests pass, and Clippy emits no warnings.

- [ ] **Step 3: Build the local macOS app bundle**

Run: `npm run tauri -- build --bundles app`

Expected: `src-tauri/target/release/bundle/macos/AgentPalette.app` is produced. Do not claim local DMG success because the local DMG script has already failed in this environment.

- [ ] **Step 4: Review diff and remote**

Run: `git diff --check && git diff --stat && git status --short && git remote -v`

Expected: no whitespace errors; all intended files appear; no unrelated content is modified. Add `origin https://github.com/Aimny2020/agentpalette.git` only if no origin exists, then re-run `git remote -v`. Do not commit or push.
