# Changelog

English | [简体中文](./CHANGELOG.zh-CN.md)

All notable changes to AgentPalette are documented here. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and this project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Planned

- Evaluate Linux builds, production code signing, and automatic updates after internal installer validation.

## [1.1.0] - 2026-07-28

### Added

- English user interface and a display-language preference for system default, Simplified Chinese, or English. Changes apply immediately and are stored locally.
- English-language support and template content for global Harness templates and project Harness workflows.
- English README and changelog, with language-matched documentation screenshots.

### Changed

- Standardized the AgentPalette application identifier, database file, global Skills and Harness data directories, and theme storage key.
- GitHub Releases now use bilingual English and Simplified Chinese release notes.

## [1.0.0] - 2026-07-21

### Added

- Public release of AgentPalette with macOS installers for Apple Silicon and Intel, plus a Windows x64 installer.
- Project management, local Agent detection and maintenance, global Skills management, global Harness template management, and project-level Skills and Harness configuration.
- Platform launch preferences, appearance settings, and workflows for starting local Agents inside a project.

### Changed

- Changed the project license to Apache License 2.0 and added contribution and brand-usage guidance.
- Changed the GitHub tag workflow to create public final Releases.

### Limitations

- macOS installers use ad-hoc signing and are not Apple-notarized; Windows installers are unsigned.
- Linux, MSI, mobile installers, and automatic updates are not currently available.

## [0.2.1] - 2026-07-13

### Changed

- Improved project Harness navigation, editing layout, file scanning scope, and switching-state management.

## [0.1.0] - 2026-07-07

### Added

- Initial desktop architecture based on Tauri v2, React, TypeScript, and Rust.
- Project navigation, appearance switching, shared UI state, and the application shell.
- SQLite migrations, domain ports, application services, and typed Tauri IPC.
- Local Skill and Skill-pack scanning, Markdown metadata parsing, and executable-content risk detection.
- Import inspection for local directories and Git sources, provenance records, and trust-state display.
- Skill details, Skill-pack member display, and dependency-protected deletion confirmation.
- Vitest frontend tests, Rust unit tests, Clippy checks, and a tag-driven macOS/Windows Release workflow.

### Limitations

- This version was for private internal preview only and was not authorized for public distribution.
- macOS used ad-hoc signing and was not notarized.
- Windows installers were unsigned.
- Linux, MSI, mobile installers, and automatic updates were not available.

[Unreleased]: https://github.com/Aimny2020/agentpalette/compare/v1.1.0...HEAD
[1.1.0]: https://github.com/Aimny2020/agentpalette/releases/tag/v1.1.0
[1.0.0]: https://github.com/Aimny2020/agentpalette/releases/tag/v1.0.0
[0.2.1]: https://github.com/Aimny2020/agentpalette/releases/tag/v0.2.1
[0.1.0]: https://github.com/Aimny2020/agentpalette/releases/tag/v0.1.0
