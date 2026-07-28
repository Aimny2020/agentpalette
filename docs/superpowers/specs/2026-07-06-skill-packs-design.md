# Skill Packs Design

> Skill packages use the AgentPalette user data root: `~/.agentpalette/`.

## Goal

Extend the existing Skills catalog so AgentPalette can manage both standalone Skills and atomic Skill Packs such as `obra/superpowers`, without introducing a separate Plugins page or a Harness-specific runtime in this release.

## Product model

- A standalone Skill is a top-level directory whose `SKILL.md` is the only discovered Skill definition.
- A Skill Pack is a top-level directory containing more than one descendant `SKILL.md`. A directory with one nested `SKILL.md` is represented as a nested standalone Skill rather than a pack.
- A directory containing a root `SKILL.md` plus descendant Skills is a Skill Pack; the root Skill is one of its members.
- Packs are atomic for project enablement, updates, and removal. Members remain individually searchable and readable but cannot be enabled, categorized, updated, or removed independently.
- The parent pack owns category, notes, source, version, trust, and lifecycle metadata. Member Skills inherit the parent category.

## Discovery

AgentPalette recursively scans each immediate child of `~/.agentpalette/skills/` for `SKILL.md` files. Discovery does not follow symbolic links and ignores `.git`, `node_modules`, `target`, and hidden cache directories.

Each catalog entry has a stable top-level ID. Pack members have stable IDs derived from the pack ID and their relative path, while their source directory is preserved unchanged. Existing libraries are re-indexed automatically; no re-import or file movement is required.

Metadata resolution follows this order:

1. Recognized package manifest (`.codex-plugin/plugin.json`, `.claude-plugin/plugin.json`, then other supported manifests).
2. Root `SKILL.md` metadata.
3. Repository README title or top-level directory name.

Malformed member Skills are reported on the pack rather than causing valid siblings to disappear.

## Catalog experience

Normal browsing shows one card per standalone Skill or Skill Pack. A pack card displays its package badge, member count, source, installed version or commit, and update state. Opening it shows the member list and lets the user open each member's rendered instructions.

Search indexes the parent and all members. A member match appears as a direct result carrying a “from <pack>” label. Clearing search returns to the parent-only catalog. Categories and sidebar counts operate on parent entries only.

## Import and provenance

Folder and Git imports both accept standalone Skills and Skill Packs. Git imports clone into a temporary directory, inspect the contents, and present a summary before installation. The stored source is normalized so HTTPS, SSH, and optional `.git` spellings of the same repository deduplicate to one package.

Git installs record the normalized source, tracked ref, installed commit, display version, and last checked time. The preferred tracked ref is the highest stable semantic-version tag; repositories without stable tags track their default branch. Advanced selection of a branch or tag is supported by the backend contract even if the first UI exposes only the recommended default.

Existing cloned directories recover provenance from `.git`. Local folders are marked as local sources and do not offer remote updates.

## Update lifecycle

Opening the Skills page performs a non-blocking update check; the toolbar also exposes a manual check. A remote ref is compared with the installed commit. Updates are never installed automatically.

Before update, AgentPalette verifies that the global Git worktree is clean. A dirty package is blocked from automatic update. A confirmed update is staged in a temporary sibling directory, validated, and atomically swapped into place. Failure leaves the prior package intact.

Clean project installations managed by AgentPalette are synchronized during the confirmed global update. Modified project copies stay at their current commit and are marked for attention. Each project-package relation records its installed commit.

## Project enablement and removal

Skill Packs are copied to projects unchanged and remain atomic. Enabling or disabling applies to the entire top-level package directory. A globally installed package that is active in any project cannot be directly removed. The user must explicitly remove it from all projects and then uninstall it.

## Trust and executable content

Inspection identifies hooks, executable files, and common script extensions. These files are preserved, but enabling a package with executable content requires explicit trust. Trust is recorded against the package and installed commit. When an update adds or changes executable content, trust must be granted again.

This release detects and reports non-Skill components but does not configure Harness-specific Hooks, MCP servers, commands, agents, or adapters.

## Persistence

SQLite stores package-level user metadata and Git provenance. Project relations reference the top-level package ID and store the installed commit and synchronization state. Member Skills are derived from disk during indexing and do not require independent database rows.

The existing standalone Skill IDs remain valid. Existing category, notes, and project relations therefore survive re-indexing. Database migrations are additive.

## Failure behavior

- An import with no valid `SKILL.md` fails without modifying the library.
- A duplicate Git source returns an already-installed result instead of creating a numbered copy.
- Invalid members appear as warnings alongside valid members.
- Network failures set update state to unknown without hiding installed content.
- Dirty Git packages and dirty project copies are never overwritten silently.
- Filesystem changes and database lifecycle records are coordinated so failures retain the previous usable installation.

## Verification

Rust tests cover recursive discovery, ignored directories and symlinks, pack classification, malformed members, Git URL normalization, duplicate detection, dirty-worktree blocking, executable-content detection, and package-level project records. Frontend tests cover parent-only browsing, member search results, inherited categories, pack detail navigation, update badges, and atomic project toggles. The complete frontend and Rust suites, lint, build, formatting, and Clippy must pass before completion.

## Deferred

- Harness-specific installation adapters.
- Executing or translating Hooks, MCP, commands, agents, or subagents.
- Per-member enablement or category overrides.
- Automatic updates, semantic merges, or overwriting local modifications.
- Marketplace discovery and publisher signing.
