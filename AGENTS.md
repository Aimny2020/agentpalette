# Repository Guidelines

## Project Structure & Module Organization

AgentPalette is a Tauri v2 desktop application. React and TypeScript live in `src/`: `app/` owns routing, providers, and the shared shell; `features/` contains page-level modules; `shared/` contains IPC clients, reusable UI, theme state, styles, and mock data. Rust lives in `src-tauri/src/`: `commands/` exposes thin Tauri handlers, `application/` coordinates use cases, `domain/` defines stable models and ports, and `infrastructure/` implements SQLite and platform adapters. Database migrations are in `src-tauri/migrations/`. Product specifications and plans are under `docs/superpowers/`.

## Build, Test, and Development Commands

- `npm install` installs frontend and Tauri CLI dependencies.
- `npm run tauri:dev` launches the complete desktop application.
- `npm run dev` runs only the Vite frontend at `127.0.0.1:1420`.
- `npm run build` type-checks TypeScript and creates the production frontend bundle.
- `npm run test:run` runs Vitest once; use `npm test` for watch mode.
- `cargo test --manifest-path src-tauri/Cargo.toml` runs Rust tests.
- `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings` enforces warning-free Rust.

## Coding Style & Naming Conventions

Use two-space indentation in TypeScript and four spaces in Rust. React components and files use `PascalCase`; hooks and utilities use `camelCase`; Rust modules and functions use `snake_case`. Keep Tauri commands small and move decisions into application/domain code. Format Rust with `cargo fmt`; TypeScript must pass `npm run lint`. Prefer semantic CSS classes and variables from `src/shared/styles/tokens.css`.

## Testing Guidelines

Follow red-green-refactor. Place frontend tests beside code as `*.test.ts(x)` and Rust unit tests in the relevant module. Test user-visible behavior with Testing Library and isolate only native Tauri boundaries. New IPC types, domain rules, routes, and error mappings require tests. Run both test suites before requesting review.

## Commit & Pull Request Guidelines

This workspace has no Git history yet; use Conventional Commits such as `feat: add project shell` or `test: cover health command`. Keep commits focused. Pull requests should explain scope, list verification commands, link relevant issues, and include screenshots for UI changes. Call out schema or IPC contract changes explicitly.

## Security & Configuration

Never commit secrets, `.env` files, local databases, or generated Agent credentials. Future project-shared settings belong in `.agentpalette/manifest.toml`; sensitive values must be referenced by name and stored in the system Keychain.
