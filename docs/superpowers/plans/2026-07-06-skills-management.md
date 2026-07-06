# Skills Management Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 实现全局 Skills（技能库）的文件管理、SQLite 用户属性（备注、自定义分类、开关状态）持久化，并展示支持 3/4 屏 Zoom 缩放详情与 Git/本地文件夹导入的精美 UI。

**Architecture:** 
- **数据层**：全局技能存在本地 `~/.agent-forge/skills/` 目录下；用户自定义备注、分类与开启状态存储于本地 SQLite 数据库中。
- **业务层**：Rust 后端提供 `SkillService` 和 `CategoryService` 负责文件扫描、`pulldown-cmark` Markdown 渲染为 HTML、Git 仓库克隆、物理文件删除以及 SQLite 数据持久化；Tauri Commands 暴露类型化的 IPC 接口。
- **展示层**：前端 React + Vanilla CSS 实现自适应网格展示，支持搜索过滤、左侧自定义分类侧边栏，以及具有 Zoom 缩放进入动画的详情 Modal。

**Tech Stack:**
- Frontend: React 19, react-router-dom, Zustand, @tanstack/react-query, lucide-react, Vanilla CSS.
- Backend: Rust, Tauri v2, rusqlite, pulldown-cmark, git2 (or command execution of `git clone`), serde.

## Global Constraints
- Two-space indentation in TypeScript, four-space indentation in Rust.
- React components and files use `PascalCase`; hooks and utilities use `camelCase`; Rust modules and functions use `snake_case`.
- Prefer semantic CSS classes and variables from `src/shared/styles/tokens.css`.
- New IPC types, domain rules, and database migrations require tests. Run both test suites (`npm run test:run` and `cargo test`) before completing tasks.

---

### Task 1: Database Migration for Skills & Categories

**Files:**
- Create: `src-tauri/migrations/002_skills.sql`
- Test: `src-tauri/src/infrastructure/database.rs` (adding database tests for new tables)

**Interfaces:**
- Consumes: None (initial db is ready)
- Produces: Database schemas for `categories` and `skills_user_meta` tables in SQLite.

- [ ] **Step 1: Create the SQL migration file**
  Create `src-tauri/migrations/002_skills.sql`:
  ```sql
  CREATE TABLE IF NOT EXISTS categories (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL UNIQUE,
    created_at TEXT NOT NULL
  );

  CREATE TABLE IF NOT EXISTS skills_user_meta (
    skill_id TEXT PRIMARY KEY NOT NULL,
    category_id TEXT,
    user_notes TEXT,
    is_enabled INTEGER NOT NULL DEFAULT 1,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (category_id) REFERENCES categories(id) ON DELETE SET NULL
  );
  ```

- [ ] **Step 2: Read migrations dynamically in `database.rs`**
  Modify `src-tauri/src/infrastructure/database.rs` to execute migration files or update the `INITIAL_MIGRATION` batch string.
  Change:
  ```rust
  const INITIAL_MIGRATION: &str = include_str!("../../migrations/001_initial.sql");
  ```
  To:
  ```rust
  const INITIAL_MIGRATION: &str = include_str!("../../migrations/001_initial.sql");
  const SKILLS_MIGRATION: &str = include_str!("../../migrations/002_skills.sql");
  ```
  Update `initialize` in `src-tauri/src/infrastructure/database.rs` (around line 29):
  ```rust
  fn initialize(connection: Connection) -> DomainResult<Self> {
      connection
          .execute_batch("PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL;")
          .map_err(database_error)?;
      connection
          .execute_batch(INITIAL_MIGRATION)
          .map_err(database_error)?;
      connection
          .execute_batch(SKILLS_MIGRATION)
          .map_err(database_error)?;
      Ok(Self {
          connection: Mutex::new(connection),
      })
  }
  ```

- [ ] **Step 3: Write tests verifying schema creation**
  Modify `src-tauri/src/infrastructure/database.rs` test module (around line 83):
  ```rust
  #[test]
  fn in_memory_database_applies_schema_and_reports_ready() {
      let database = SqliteDatabase::open_in_memory().expect("database should initialize");

      assert_eq!(database.status().unwrap(), DatabaseStatus::Ready);
      assert!(database.has_table("_migrations").unwrap());
      assert!(database.has_table("projects").unwrap());
      assert!(database.has_table("task_runs").unwrap());
      assert!(database.has_table("categories").unwrap());
      assert!(database.has_table("skills_user_meta").unwrap());
  }
  ```

- [ ] **Step 4: Run tests to verify they pass**
  Run: `cargo test --manifest-path src-tauri/Cargo.toml`
  Expected: PASS

- [ ] **Step 5: Commit**
  ```bash
  git add src-tauri/migrations/002_skills.sql src-tauri/src/infrastructure/database.rs
  git commit -m "feat: add SQLite migration for categories and skills metadata"
  ```

---

### Task 2: Domain Layer Models for Skills

**Files:**
- Create: `src-tauri/src/domain/skill.rs`
- Modify: `src-tauri/src/domain/mod.rs`
- Modify: `src-tauri/src/domain/ports.rs`

**Interfaces:**
- Consumes: Database schema from Task 1.
- Produces: `Skill`, `SkillMetadata`, `Category` domain types.

- [ ] **Step 1: Create the domain module for Skills**
  Create `src-tauri/src/domain/skill.rs`:
  ```rust
  use serde::{Deserialize, Serialize};

  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct SkillMetadata {
      pub name: String,
      pub description: String,
      pub author: Option<String>,
      pub version: Option<String>,
  }

  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct Skill {
      pub id: String,
      pub metadata: SkillMetadata,
      pub html_content: String,
      pub category_id: Option<String>,
      pub user_notes: Option<String>,
      pub is_enabled: bool,
  }

  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct Category {
      pub id: String,
      pub name: String,
      pub created_at: String,
  }
  ```

- [ ] **Step 2: Register domain module in `domain/mod.rs`**
  Modify `src-tauri/src/domain/mod.rs`:
  ```rust
  pub mod agent;
  pub mod error;
  pub mod health;
  pub mod ports;
  pub mod task;
  pub mod skill;
  ```

- [ ] **Step 3: Define repository ports in `domain/ports.rs`**
  Add ports for Categories and Skills persistence.
  Modify `src-tauri/src/domain/ports.rs` to append:
  ```rust
  use crate::domain::skill::{Category, Skill};
  use crate::domain::error::DomainResult;

  pub trait SkillRepository: Send + Sync {
      fn get_user_meta(&self, skill_id: &str) -> DomainResult<Option<(Option<String>, Option<String>, bool)>>;
      fn save_user_meta(&self, skill_id: &str, category_id: Option<&str>, user_notes: Option<&str>, is_enabled: bool) -> DomainResult<()>;
      fn delete_user_meta(&self, skill_id: &str) -> DomainResult<()>;
      
      fn get_categories(&self) -> DomainResult<Vec<Category>>;
      fn create_category(&self, id: &str, name: &str, created_at: &str) -> DomainResult<Category>;
      fn rename_category(&self, id: &str, name: &str) -> DomainResult<()>;
      fn delete_category(&self, id: &str) -> DomainResult<()>;
  }
  ```

- [ ] **Step 4: Run clippy and check build**
  Run: `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings`
  Expected: PASS

- [ ] **Step 5: Commit**
  ```bash
  git add src-tauri/src/domain/skill.rs src-tauri/src/domain/mod.rs src-tauri/src/domain/ports.rs
  git commit -m "feat: define Skill and Category domain models and repository ports"
  ```

---

### Task 3: Infrastructure Persistence and Markdown Engine

**Files:**
- Modify: `src-tauri/Cargo.toml` (add `pulldown-cmark`, `yaml-front-matter`)
- Modify: `src-tauri/src/infrastructure/database.rs` (implement `SkillRepository`)
- Create: `src-tauri/src/infrastructure/markdown.rs` (implement parser)
- Modify: `src-tauri/src/infrastructure/mod.rs`

**Interfaces:**
- Consumes: `SkillRepository` trait from Task 2.
- Produces: Database persistent methods for skills & markdown rendering engine.

- [ ] **Step 1: Add dependencies to Cargo.toml**
  Modify `src-tauri/Cargo.toml` to add to `dependencies`:
  ```toml
  pulldown-cmark = "0.10"
  yaml-front-matter = "0.1"
  chrono = "0.4"
  ```

- [ ] **Step 2: Implement Markdown & YAML parser**
  Create `src-tauri/src/infrastructure/markdown.rs`:
  ```rust
  use serde::Deserialize;
  use yaml_front_matter::YamlFrontMatter;
  use pulldown-cmark::{Parser, Options, html};
  use crate::domain::error::{DomainError, DomainResult};
  use crate::domain::skill::SkillMetadata;

  #[derive(Deserialize)]
  struct FrontMatterRaw {
      name: String,
      description: String,
      author: Option<String>,
      version: Option<String>,
  }

  pub fn parse_skill_markdown(content: &str) -> DomainResult<(SkillMetadata, String)> {
      let document = YamlFrontMatter::parse::<FrontMatterRaw>(content)
          .map_err(|e| DomainError::Database(format!("Failed to parse Frontmatter: {}", e)))?;
      
      let raw_meta = document.metadata;
      let metadata = SkillMetadata {
          name: raw_meta.name,
          description: raw_meta.description,
          author: raw_meta.author,
          version: raw_meta.version,
      };

      let markdown_body = document.content;
      let mut options = Options::empty();
      options.insert(Options::ENABLE_TABLES);
      options.insert(Options::ENABLE_STRIKETHROUGH);
      let parser = Parser::new_ext(&markdown_body, options);
      
      let mut html_output = String::new();
      html::push_html(&mut html_output, parser);

      Ok((metadata, html_output))
  }
  ```

- [ ] **Step 3: Register `markdown` module**
  Modify `src-tauri/src/infrastructure/mod.rs` to add:
  ```rust
  pub mod markdown;
  ```

- [ ] **Step 4: Implement `SkillRepository` in `SqliteDatabase`**
  Modify `src-tauri/src/infrastructure/database.rs` to implement `SkillRepository` for `SqliteDatabase`:
  ```rust
  use crate::domain::ports::SkillRepository;
  use crate::domain::skill::{Category, Skill};
  use crate::domain::error::DomainResult;

  impl SkillRepository for SqliteDatabase {
      fn get_user_meta(&self, skill_id: &str) -> DomainResult<Option<(Option<String>, Option<String>, bool)>> {
          let connection = self.connection.lock().map_err(|e| DomainError::Database(e.to_string()))?;
          let mut stmt = connection.prepare(
              "SELECT category_id, user_notes, is_enabled FROM skills_user_meta WHERE skill_id = ?1"
          ).map_err(database_error)?;
          
          let row = stmt.query_row([skill_id], |r| {
              let category_id: Option<String> = r.get(0)?;
              let user_notes: Option<String> = r.get(1)?;
              let is_enabled_int: i32 = r.get(2)?;
              Ok((category_id, user_notes, is_enabled_int != 0))
          }).optional().map_err(database_error)?;
          
          Ok(row)
      }

      fn save_user_meta(&self, skill_id: &str, category_id: Option<&str>, user_notes: Option<&str>, is_enabled: bool) -> DomainResult<()> {
          let connection = self.connection.lock().map_err(|e| DomainError::Database(e.to_string()))?;
          let is_enabled_int = if is_enabled { 1 } else { 0 };
          let now = chrono::Utc::now().to_rfc3339();
          connection.execute(
              "INSERT INTO skills_user_meta (skill_id, category_id, user_notes, is_enabled, updated_at)
               VALUES (?1, ?2, ?3, ?4, ?5)
               ON CONFLICT(skill_id) DO UPDATE SET
                 category_id = excluded.category_id,
                 user_notes = excluded.user_notes,
                 is_enabled = excluded.is_enabled,
                 updated_at = excluded.updated_at",
              rusqlite::params![skill_id, category_id, user_notes, is_enabled_int, now]
          ).map_err(database_error)?;
          Ok(())
      }

      fn delete_user_meta(&self, skill_id: &str) -> DomainResult<()> {
          let connection = self.connection.lock().map_err(|e| DomainError::Database(e.to_string()))?;
          connection.execute("DELETE FROM skills_user_meta WHERE skill_id = ?1", [skill_id]).map_err(database_error)?;
          Ok(())
      }

      fn get_categories(&self) -> DomainResult<Vec<Category>> {
          let connection = self.connection.lock().map_err(|e| DomainError::Database(e.to_string()))?;
          let mut stmt = connection.prepare("SELECT id, name, created_at FROM categories ORDER BY created_at ASC").map_err(database_error)?;
          let iter = stmt.query_map([], |r| {
              Ok(Category {
                  id: r.get(0)?,
                  name: r.get(1)?,
                  created_at: r.get(2)?,
              })
          }).map_err(database_error)?;
          
          let mut list = Vec::new();
          for c in iter {
              list.push(c.map_err(database_error)?);
          }
          Ok(list)
      }

      fn create_category(&self, id: &str, name: &str, created_at: &str) -> DomainResult<Category> {
          let connection = self.connection.lock().map_err(|e| DomainError::Database(e.to_string()))?;
          connection.execute(
              "INSERT INTO categories (id, name, created_at) VALUES (?1, ?2, ?3)",
              [id, name, created_at]
          ).map_err(database_error)?;
          Ok(Category {
              id: id.to_string(),
              name: name.to_string(),
              created_at: created_at.to_string(),
          })
      }

      fn rename_category(&self, id: &str, name: &str) -> DomainResult<()> {
          let connection = self.connection.lock().map_err(|e| DomainError::Database(e.to_string()))?;
          connection.execute(
              "UPDATE categories SET name = ?1 WHERE id = ?2",
              [name, id]
          ).map_err(database_error)?;
          Ok(())
      }

      fn delete_category(&self, id: &str) -> DomainResult<()> {
          let connection = self.connection.lock().map_err(|e| DomainError::Database(e.to_string()))?;
          connection.execute("DELETE FROM categories WHERE id = ?1", [id]).map_err(database_error)?;
          Ok(())
      }
  }
  ```

- [ ] **Step 5: Write tests verifying markdown and DB operations**
  Create a test block in `src-tauri/src/infrastructure/markdown.rs` to verify Parsing.
  Run: `cargo test --manifest-path src-tauri/Cargo.toml`
  Expected: PASS

- [ ] **Step 6: Commit**
  ```bash
  git add src-tauri/Cargo.toml src-tauri/src/infrastructure/
  git commit -m "feat: implement markdown parser and SqliteDatabase skill repository methods"
  ```

---

### Task 4: Skill Application Services (Git/Local Import)

**Files:**
- Create: `src-tauri/src/application/skill_service.rs`
- Modify: `src-tauri/src/application/mod.rs`

**Interfaces:**
- Consumes: `SkillRepository` trait and `parse_skill_markdown`.
- Produces: `SkillService` to scan directory, delete, update meta, copy local files, and run Git Clone.

- [ ] **Step 1: Implement SkillService**
  Create `src-tauri/src/application/skill_service.rs` with `get_skills()`, `import_local_folder()`, `import_git_url()`, and `delete_skill()`:
  ```rust
  use std::path::{Path, PathBuf};
  use std::sync::Arc;
  use std::fs;
  use crate::domain::ports::SkillRepository;
  use crate::domain::skill::{Skill, Category};
  use crate::domain::error::{DomainError, DomainResult};
  use crate::infrastructure::markdown::parse_skill_markdown;

  pub struct SkillService {
      repo: Arc<dyn SkillRepository>,
      skills_dir: PathBuf,
  }

  impl SkillService {
      pub fn new(repo: Arc<dyn SkillRepository>) -> Self {
          let home = dirs::home_dir().expect("Failed to locate home directory");
          let skills_dir = home.join(".agent-forge").join("skills");
          if !skills_dir.exists() {
              fs::create_dir_all(&skills_dir).expect("Failed to create skills directory");
          }
          Self { repo, skills_dir }
      }

      pub fn get_skills(&self) -> DomainResult<Vec<Skill>> {
          let mut list = Vec::new();
          if !self.skills_dir.exists() {
              return Ok(list);
          }
          for entry in fs::read_dir(&self.skills_dir).map_err(|e| DomainError::Database(e.to_string()))? {
              let entry = entry.map_err(|e| DomainError::Database(e.to_string()))?;
              let path = entry.path();
              if path.is_dir() {
                  let skill_id = path.file_name().and_then(|s| s.to_str()).unwrap_or("").to_string();
                  let skill_md_path = path.join("SKILL.md");
                  if skill_md_path.exists() {
                      let content = fs::read_to_string(&skill_md_path).map_err(|e| DomainError::Database(e.to_string()))?;
                      if let Ok((metadata, html)) = parse_skill_markdown(&content) {
                          let (cat_id, notes, enabled) = match self.repo.get_user_meta(&skill_id)? {
                              Some((c, n, e)) => (c, n, e),
                              None => (None, None, true),
                          };
                          list.push(Skill {
                              id: skill_id,
                              metadata,
                              html_content: html,
                              category_id: cat_id,
                              user_notes: notes,
                              is_enabled: enabled,
                          });
                      }
                  }
              }
          }
          Ok(list)
      }

      pub fn import_local_folder(&self, source_path: &str) -> DomainResult<String> {
          let src = Path::new(source_path);
          if !src.exists() || !src.is_dir() {
              return Err(DomainError::Database("Source directory does not exist".into()));
          }
          let md_path = src.join("SKILL.md");
          if !md_path.exists() {
              return Err(DomainError::Database("SKILL.md not found in source directory".into()));
          }
          let id = src.file_name().and_then(|s| s.to_str()).ok_or_else(|| DomainError::Database("Invalid folder name".into()))?;
          let dest = self.skills_dir.join(id);
          if dest.exists() {
              fs::remove_dir_all(&dest).map_err(|e| DomainError::Database(e.to_string()))?;
          }
          self.copy_dir_all(src, &dest)?;
          Ok(id.to_string())
      }

      pub fn import_git_url(&self, url: &str) -> DomainResult<String> {
          let repo_name = url.split('/').last().and_then(|s| s.strip_suffix(".git").or(Some(s))).ok_or_else(|| DomainError::Database("Invalid Git URL".into()))?;
          let dest = self.skills_dir.join(repo_name);
          if dest.exists() {
              fs::remove_dir_all(&dest).map_err(|e| DomainError::Database(e.to_string()))?;
          }
          // Run command git clone
          let status = std::process::Command::new("git")
              .args(["clone", url, dest.to_str().unwrap()])
              .status()
              .map_err(|e| DomainError::Database(format!("Git clone execution error: {}", e)))?;
          
          if !status.success() {
              return Err(DomainError::Database("git clone command exited with error".into()));
          }
          let md_path = dest.join("SKILL.md");
          if !md_path.exists() {
              return Err(DomainError::Database("Imported git repository does not contain a SKILL.md".into()));
          }
          Ok(repo_name.to_string())
      }

      pub fn delete_skill(&self, skill_id: &str) -> DomainResult<()> {
          let path = self.skills_dir.join(skill_id);
          if path.exists() && path.is_dir() {
              fs::remove_dir_all(&path).map_err(|e| DomainError::Database(e.to_string()))?;
          }
          self.repo.delete_user_meta(skill_id)?;
          Ok(())
      }

      fn copy_dir_all(&self, src: impl AsRef<Path>, dst: impl AsRef<Path>) -> std::io::Result<()> {
          fs::create_dir_all(&dst)?;
          for entry in fs::read_dir(src)? {
              let entry = entry?;
              let ty = entry.file_type()?;
              if ty.is_dir() {
                  self.copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
              } else {
                  fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
              }
          }
          Ok(())
      }
  }
  ```

- [ ] **Step 2: Register `skill_service` in `application/mod.rs`**
  Modify `src-tauri/src/application/mod.rs` to add:
  ```rust
  pub mod health_service;
  pub mod skill_service;
  ```

- [ ] **Step 3: Run unit tests for skill services**
  Verify everything builds and tests pass.
  Run: `cargo test --manifest-path src-tauri/Cargo.toml`
  Expected: PASS

- [ ] **Step 4: Commit**
  ```bash
  git add src-tauri/src/application/
  git commit -m "feat: implement SkillService backend business logic"
  ```

---

### Task 5: Tauri IPC Commands for Skills & AppState Hookup

**Files:**
- Create: `src-tauri/src/commands/skills.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs`

**Interfaces:**
- Consumes: `SkillService` and `SkillRepository`
- Produces: Command handlers registered in Tauri builder.

- [ ] **Step 1: Create commands module**
  Create `src-tauri/src/commands/skills.rs` and hook up commands. Implement proper mapping from `DomainError` to `CommandError`.
  ```rust
  use serde::Serialize;
  use crate::commands::health::AppState;
  use crate::domain::skill::{Skill, Category};
  use tauri::State;

  #[derive(Serialize)]
  pub struct CommandError {
      pub code: String,
      pub message: String,
  }

  fn to_cmd_err(err: crate::domain::error::DomainError) -> CommandError {
      CommandError {
          code: "skill_error".into(),
          message: err.to_string(),
      }
  }

  #[tauri::command]
  pub async fn get_skills(state: State<'_, AppState>) -> Result<Vec<Skill>, CommandError> {
      state.skills.get_skills().map_err(to_cmd_err)
  }

  #[tauri::command]
  pub async fn import_skill(state: State<'_, AppState>, source: String, import_type: String) -> Result<String, CommandError> {
      if import_type == "git" {
          state.skills.import_git_url(&source).map_err(to_cmd_err)
      } else {
          state.skills.import_local_folder(&source).map_err(to_cmd_err)
      }
  }

  #[tauri::command]
  pub async fn delete_skill(state: State<'_, AppState>, skill_id: String) -> Result<(), CommandError> {
      state.skills.delete_skill(&skill_id).map_err(to_cmd_err)
  }

  #[tauri::command]
  pub async fn update_skill_meta(
      state: State<'_, AppState>,
      skill_id: String,
      category_id: Option<String>,
      user_notes: Option<String>,
      is_enabled: bool,
  ) -> Result<(), CommandError> {
      state.repo.save_user_meta(
          &skill_id,
          category_id.as_deref(),
          user_notes.as_deref(),
          is_enabled
      ).map_err(to_cmd_err)
  }

  #[tauri::command]
  pub async fn get_categories(state: State<'_, AppState>) -> Result<Vec<Category>, CommandError> {
      state.repo.get_categories().map_err(to_cmd_err)
  }

  #[tauri::command]
  pub async fn create_category(state: State<'_, AppState>, name: String) -> Result<Category, CommandError> {
      let id = uuid::Uuid::new_v4().to_string();
      let created_at = chrono::Utc::now().to_rfc3339();
      state.repo.create_category(&id, &name, &created_at).map_err(to_cmd_err)
  }

  #[tauri::command]
  pub async fn rename_category(state: State<'_, AppState>, id: String, name: String) -> Result<(), CommandError> {
      state.repo.rename_category(&id, &name).map_err(to_cmd_err)
  }

  #[tauri::command]
  pub async fn delete_category(state: State<'_, AppState>, id: String) -> Result<(), CommandError> {
      state.repo.delete_category(&id).map_err(to_cmd_err)
  }
  ```

- [ ] **Step 2: Add `uuid` to Cargo.toml dependencies**
  Modify `src-tauri/Cargo.toml` to add to `dependencies`:
  ```toml
  uuid = { version = "1.0", features = ["v4"] }
  ```

- [ ] **Step 3: Register commands in `commands/mod.rs`**
  Modify `src-tauri/src/commands/mod.rs` to add:
  ```rust
  pub mod health;
  pub mod skills;
  ```

- [ ] **Step 4: Hook up services in AppState inside `lib.rs`**
  Modify `AppState` in `src-tauri/src/commands/health.rs`:
  ```rust
  use crate::application::health_service::HealthService;
  use crate::application::skill_service::SkillService;
  use crate::domain::ports::SkillRepository;
  use std::sync::Arc;

  pub struct AppState {
      pub health: HealthService,
      pub skills: SkillService,
      pub repo: Arc<dyn SkillRepository>,
  }
  ```
  Modify `lib.rs` setup logic:
  ```rust
  use std::sync::Arc;
  use commands::skills::*;
  // ...
  let database = Arc::new(SqliteDatabase::open(&app_data_dir.join("agentforge.db"))?);
  let system = Arc::new(PlatformSystem::current());
  
  app.manage(AppState {
      health: HealthService::new(Arc::clone(&database) as Arc<dyn crate::domain::health::DatabasePort>, system),
      skills: SkillService::new(Arc::clone(&database) as Arc<dyn crate::domain::ports::SkillRepository>),
      repo: Arc::clone(&database) as Arc<dyn crate::domain::ports::SkillRepository>,
  });
  // ...
  .invoke_handler(tauri::generate_handler![
      health_check,
      get_skills,
      import_skill,
      delete_skill,
      update_skill_meta,
      get_categories,
      create_category,
      rename_category,
      delete_category
  ])
  ```

- [ ] **Step 5: Verify build**
  Run: `cargo test --manifest-path src-tauri/Cargo.toml`
  Expected: PASS

- [ ] **Step 6: Commit**
  ```bash
  git add src-tauri/
  git commit -m "feat: expose Tauri IPC commands and hook up AppState dependencies"
  ```

---

### Task 6: Frontend API Integration & Mock Setup

**Files:**
- Modify: `src/shared/api/types.ts`
- Modify: `src/shared/api/tauriClient.ts`

**Interfaces:**
- Consumes: Rust commands and data models.
- Produces: Frontend functions for Categories and Skills client calling.

- [ ] **Step 1: Extend typescript types**
  Modify `src/shared/api/types.ts`:
  ```typescript
  export interface SkillMetadata {
    name: string;
    description: string;
    author?: string;
    version?: string;
  }

  export interface Skill {
    id: string;
    metadata: SkillMetadata;
    html_content: string;
    category_id?: string;
    user_notes?: string;
    is_enabled: boolean;
  }

  export interface Category {
    id: string;
    name: string;
    created_at: string;
  }
  ```

- [ ] **Step 2: Add API wrappers to `tauriClient.ts`**
  Modify `src/shared/api/tauriClient.ts` to export new client calls:
  ```typescript
  import type { Skill, Category } from './types';

  export async fn getSkills(): Promise<Skill[]> {
    try {
      return await invoke<Skill[]>('get_skills');
    } catch (error) {
      throw normalizeError(error);
    }
  }

  export async fn importSkill(source: string, importType: 'folder' | 'git'): Promise<string> {
    try {
      return await invoke<string>('import_skill', { source, importType });
    } catch (error) {
      throw normalizeError(error);
    }
  }

  export async fn deleteSkill(skillId: string): Promise<void> {
    try {
      await invoke<void>('delete_skill', { skillId });
    } catch (error) {
      throw normalizeError(error);
    }
  }

  export async fn updateSkillMeta(
    skillId: string,
    categoryId: string | null,
    userNotes: string | null,
    isEnabled: boolean,
  ): Promise<void> {
    try {
      await invoke<void>('update_skill_meta', { skillId, categoryId, userNotes, isEnabled });
    } catch (error) {
      throw normalizeError(error);
    }
  }

  export async fn getCategories(): Promise<Category[]> {
    try {
      return await invoke<Category[]>('get_categories');
    } catch (error) {
      throw normalizeError(error);
    }
  }

  export async fn createCategory(name: string): Promise<Category> {
    try {
      return await invoke<Category>('create_category', { name });
    } catch (error) {
      throw normalizeError(error);
    }
  }

  export async fn renameCategory(id: string, name: string): Promise<void> {
    try {
      await invoke<void>('rename_category', { id, name });
    } catch (error) {
      throw normalizeError(error);
    }
  }

  export async fn deleteCategory(id: string): Promise<void> {
    try {
      await invoke<void>('delete_category', { id });
    } catch (error) {
      throw normalizeError(error);
    }
  }
  ```

- [ ] **Step 3: Run typescript check**
  Run: `npm run build`
  Expected: PASS

- [ ] **Step 4: Commit**
  ```bash
  git add src/shared/api/
  git commit -m "feat: add frontend TypeScript types and client API wrappers for skills"
  ```

---

### Task 7: Categories Sidebar Component & Styling

**Files:**
- Create: `src/features/skills/components/SkillsSidebar.tsx`
- Create: `src/features/skills/components/sidebar.css`

**Interfaces:**
- Consumes: Category API wrappers.
- Produces: Sidebar sidebar component containing categories, category counts, rename & delete actions, and a create category input.

- [ ] **Step 1: Implement category sidebar with categories editor**
  Create `src/features/skills/components/SkillsSidebar.tsx` using `lucide-react` for icons. Render count of skills in each category:
  ```tsx
  import React, { useState } from 'react';
  import { Plus, Trash2, Edit2, FolderOpen } from 'lucide-react';
  import { Category } from '../../../shared/api/types';
  import './sidebar.css';

  interface Props {
    categories: Category[];
    skillsCountMap: Record<string, number>; // Maps categoryId to count
    selectedCategoryId: string | null; // null = all, 'uncategorized' = uncategorized
    onSelectCategory: (id: string | null) => void;
    onCreateCategory: (name: string) => void;
    onRenameCategory: (id: string, name: string) => void;
    onDeleteCategory: (id: string) => void;
  }

  export function SkillsSidebar({
    categories,
    skillsCountMap,
    selectedCategoryId,
    onSelectCategory,
    onCreateCategory,
    onRenameCategory,
    onDeleteCategory,
  }: Props) {
    const [newCatName, setNewCatName] = useState('');
    const [editingId, setEditingId] = useState<string | null>(null);
    const [editingName, setEditingName] = useState('');

    const handleCreate = (e: React.FormEvent) => {
      e.preventDefault();
      if (newCatName.trim()) {
        onCreateCategory(newCatName.trim());
        setNewCatName('');
      }
    };

    return (
      <aside className="skills-sidebar">
        <h3>技能分类</h3>
        <ul className="sidebar-cat-list">
          <li
            data-active={selectedCategoryId === null}
            onClick={() => onSelectCategory(null)}
          >
            <FolderOpen size={16} />
            <span>全部技能</span>
            <span className="count-badge">{skillsCountMap['all'] || 0}</span>
          </li>
          <li
            data-active={selectedCategoryId === 'uncategorized'}
            onClick={() => onSelectCategory('uncategorized')}
          >
            <FolderOpen size={16} />
            <span>未分类</span>
            <span className="count-badge">{skillsCountMap['uncategorized'] || 0}</span>
          </li>
          {categories.map((cat) => (
            <li
              key={cat.id}
              data-active={selectedCategoryId === cat.id}
              onClick={() => onSelectCategory(cat.id)}
            >
              <FolderOpen size={16} />
              {editingId === cat.id ? (
                <input
                  value={editingName}
                  onChange={(e) => setEditingName(e.target.value)}
                  onBlur={() => {
                    if (editingName.trim()) onRenameCategory(cat.id, editingName.trim());
                    setEditingId(null);
                  }}
                  onKeyDown={(e) => {
                    if (e.key === 'Enter') {
                      if (editingName.trim()) onRenameCategory(cat.id, editingName.trim());
                      setEditingId(null);
                    }
                  }}
                  autoFocus
                />
              ) : (
                <>
                  <span className="cat-name">{cat.name}</span>
                  <div className="cat-actions" onClick={(e) => e.stopPropagation()}>
                    <Edit2
                      size={12}
                      onClick={() => {
                        setEditingId(cat.id);
                        setEditingName(cat.name);
                      }}
                    />
                    <Trash2 size={12} onClick={() => onDeleteCategory(cat.id)} />
                  </div>
                </>
              )}
              <span className="count-badge">{skillsCountMap[cat.id] || 0}</span>
            </li>
          ))}
        </ul>
        <form onSubmit={handleCreate} className="create-category-form">
          <input
            placeholder="新建分类..."
            value={newCatName}
            onChange={(e) => setNewCatName(e.target.value)}
          />
          <button type="submit">
            <Plus size={16} />
          </button>
        </form>
      </aside>
    );
  }
  ```

- [ ] **Step 2: Create CSS file for sidebar**
  Create `src/features/skills/components/sidebar.css` using variable tokens:
  ```css
  .skills-sidebar {
    background: var(--color-surface);
    border: 1px solid var(--color-outline);
    border-radius: var(--radius-md);
    padding: var(--space-2);
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .skills-sidebar h3 {
    margin: var(--space-1);
    font-size: 0.85rem;
    color: var(--color-muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .sidebar-cat-list {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .sidebar-cat-list li {
    display: flex;
    align-items: center;
    gap: var(--space-1);
    padding: 0.65rem 0.85rem;
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-size: 0.88rem;
    transition: background 0.15s ease;
  }

  .sidebar-cat-list li:hover {
    background: var(--color-surface-soft);
  }

  .sidebar-cat-list li[data-active='true'] {
    background: var(--color-surface-strong);
    font-weight: 700;
  }

  .sidebar-cat-list li .cat-name {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .cat-actions {
    display: none;
    gap: 8px;
    margin-left: auto;
    color: var(--color-muted);
  }

  .sidebar-cat-list li:hover .cat-actions {
    display: flex;
  }

  .cat-actions svg:hover {
    color: var(--color-ink);
  }

  .count-badge {
    margin-left: auto;
    background: var(--color-surface-soft);
    color: var(--color-muted);
    font-size: 0.72rem;
    padding: 2px 6px;
    border-radius: 99px;
  }

  .create-category-form {
    display: flex;
    border: 1px solid var(--color-outline);
    border-radius: var(--radius-sm);
    padding: 4px;
    background: var(--color-canvas);
  }

  .create-category-form input {
    border: 0;
    outline: 0;
    background: transparent;
    font-size: 0.85rem;
    padding: 4px;
    flex: 1;
    color: var(--color-ink);
  }

  .create-category-form button {
    border: 0;
    background: transparent;
    cursor: pointer;
    color: var(--color-muted);
  }

  .create-category-form button:hover {
    color: var(--color-primary);
  }
  ```

- [ ] **Step 3: Run project validation**
  Run: `npm run build`
  Expected: PASS

- [ ] **Step 4: Commit**
  ```bash
  git add src/features/skills/components/
  git commit -m "feat: create SkillsSidebar and category sidebar styles"
  ```

---

### Task 8: Cards Grid and Detail Modal with Zoom Animation

**Files:**
- Create: `src/features/skills/components/SkillCard.tsx`
- Create: `src/features/skills/components/SkillDetailModal.tsx`
- Create: `src/features/skills/components/ImportSkillModal.tsx`
- Create: `src/features/skills/components/skills.css`

**Interfaces:**
- Consumes: Sidebar filters and categories data.
- Produces: Fully interactive Card grid, detail modal with HTML markdown output, and Import modal dialog.

- [ ] **Step 1: Create SkillCard component**
  Create `src/features/skills/components/SkillCard.tsx` with enabling switch, delete, and detail trigger:
  ```tsx
  import { Trash2 } from 'lucide-react';
  import { Skill } from '../../../shared/api/types';

  interface Props {
    skill: Skill;
    categoryName: string;
    onOpenDetail: () => void;
    onToggleEnable: (e: React.MouseEvent) => void;
    onDelete: (e: React.MouseEvent) => void;
  }

  export function SkillCard({ skill, categoryName, onOpenDetail, onToggleEnable, onDelete }: Props) {
    return (
      <div className="skill-card" onClick={onOpenDetail}>
        <div className="skill-card-header">
          <span className="category-tag">{categoryName}</span>
          <div className="skill-card-controls" onClick={(e) => e.stopPropagation()}>
            <input
              type="checkbox"
              className="toggle-switch"
              checked={skill.is_enabled}
              onChange={() => {}}
              onClick={onToggleEnable}
            />
            <button className="delete-btn" onClick={onDelete}>
              <Trash2 size={14} />
            </button>
          </div>
        </div>
        <h4>{skill.metadata.name}</h4>
        <p className="skill-description">{skill.metadata.description}</p>
        <div className="skill-card-footer">
          {skill.metadata.version && <span className="version-badge">v{skill.metadata.version}</span>}
          {skill.metadata.author && <span className="author-badge">by {skill.metadata.author}</span>}
        </div>
      </div>
    );
  }
  ```

- [ ] **Step 2: Create SkillDetailModal component**
  Create `src/features/skills/components/SkillDetailModal.tsx` (using 3/4 layout):
  ```tsx
  import React, { useState } from 'react';
  import { X } from 'lucide-react';
  import { Skill, Category } from '../../../shared/api/types';

  interface Props {
    skill: Skill;
    categories: Category[];
    onClose: () => void;
    onUpdate: (categoryId: string | null, userNotes: string | null, isEnabled: boolean) => void;
  }

  export function SkillDetailModal({ skill, categories, onClose, onUpdate }: Props) {
    const [notes, setNotes] = useState(skill.user_notes || '');
    const [catId, setCatId] = useState(skill.category_id || '');
    const [enabled, setEnabled] = useState(skill.is_enabled);

    const handleSave = () => {
      onUpdate(catId || null, notes || null, enabled);
      onClose();
    };

    return (
      <div className="modal-overlay" onClick={onClose}>
        <div className="modal-body" onClick={(e) => e.stopPropagation()}>
          <div className="modal-header">
            <h3>技能详情</h3>
            <button className="close-btn" onClick={onClose}>
              <X size={20} />
            </button>
          </div>
          <div className="modal-grid-content">
            <div className="modal-markdown-area">
              <h1>{skill.metadata.name}</h1>
              <div
                className="markdown-body"
                dangerouslySetInnerHTML={{ __html: skill.html_content }}
              />
            </div>
            <div className="modal-meta-editor">
              <div className="form-group">
                <label>技能状态</label>
                <div className="toggle-container">
                  <input
                    type="checkbox"
                    checked={enabled}
                    onChange={(e) => setEnabled(e.target.checked)}
                  />
                  <span>{enabled ? '已启用' : '已停用'}</span>
                </div>
              </div>
              <div className="form-group">
                <label>设置分类</label>
                <select value={catId} onChange={(e) => setCatId(e.target.value)}>
                  <option value="">未分类</option>
                  {categories.map((c) => (
                    <option key={c.id} value={c.id}>
                      {c.name}
                    </option>
                  ))}
                </select>
              </div>
              <div className="form-group flex-fill">
                <label>个人备注</label>
                <textarea
                  placeholder="在此添加该技能的个性化使用备注或说明..."
                  value={notes}
                  onChange={(e) => setNotes(e.target.value)}
                />
              </div>
              <div className="actions-footer">
                <button className="button button--secondary" onClick={onClose}>
                  取消
                </button>
                <button className="button button--primary" onClick={handleSave}>
                  保存更改
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>
    );
  }
  ```

- [ ] **Step 3: Create ImportSkillModal component**
  Create `src/features/skills/components/ImportSkillModal.tsx` supporting directory and git URL:
  ```tsx
  import React, { useState } from 'react';
  import { X, Folder, GitBranch } from 'lucide-react';

  interface Props {
    onClose: () => void;
    onImport: (source: string, type: 'folder' | 'git') => void;
  }

  export function ImportSkillModal({ onClose, onImport }: Props) {
    const [mode, setMode] = useState<'folder' | 'git'>('folder');
    const [source, setSource] = useState('');

    const handleImport = (e: React.FormEvent) => {
      e.preventDefault();
      if (source.trim()) {
        onImport(source.trim(), mode);
        onClose();
      }
    };

    return (
      <div className="modal-overlay" onClick={onClose}>
        <div className="modal-body compact-modal" onClick={(e) => e.stopPropagation()}>
          <div className="modal-header">
            <h3>导入技能</h3>
            <button className="close-btn" onClick={onClose}>
              <X size={20} />
            </button>
          </div>
          <div className="tabs-header">
            <button
              className={mode === 'folder' ? 'active-tab' : ''}
              onClick={() => setMode('folder')}
            >
              <Folder size={16} />
              <span>文件夹导入</span>
            </button>
            <button
              className={mode === 'git' ? 'active-tab' : ''}
              onClick={() => setMode('git')}
            >
              <GitBranch size={16} />
              <span>Git 仓库导入</span>
            </button>
          </div>
          <form onSubmit={handleImport} className="import-form">
            {mode === 'folder' ? (
              <div className="form-group">
                <label>本地文件夹路径</label>
                <input
                  placeholder="/Users/dev/my-skill"
                  value={source}
                  onChange={(e) => setSource(e.target.value)}
                  required
                />
              </div>
            ) : (
              <div className="form-group">
                <label>Git Clone 仓库链接</label>
                <input
                  placeholder="https://github.com/org/my-skill-repo.git"
                  value={source}
                  onChange={(e) => setSource(e.target.value)}
                  required
                />
              </div>
            )}
            <div className="actions-footer">
              <button type="button" className="button button--secondary" onClick={onClose}>
                取消
              </button>
              <button type="submit" className="button button--primary">
                确认导入
              </button>
            </div>
          </form>
        </div>
      </div>
    );
  }
  ```

- [ ] **Step 4: Create styles for skills cards and modals**
  Create `src/features/skills/components/skills.css`:
  ```css
  .skills-main-area {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }

  .skills-toolbar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: var(--space-2);
  }

  .search-input {
    border: 1px solid var(--color-outline);
    border-radius: 99px;
    padding: 0.6rem var(--space-2);
    font-size: 0.88rem;
    width: 16rem;
    background: var(--color-surface);
    color: var(--color-ink);
  }

  .skills-cards-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
    gap: var(--space-2);
  }

  /* Card */
  .skill-card {
    border: 1px solid var(--color-outline);
    border-radius: var(--radius-md);
    background: var(--color-surface);
    box-shadow: var(--shadow-card);
    padding: var(--space-2);
    display: flex;
    flex-direction: column;
    cursor: pointer;
    transition: transform 0.22s cubic-bezier(0.16, 1, 0.3, 1), border-color 0.22s ease, box-shadow 0.22s ease;
  }

  .skill-card:hover {
    transform: translateY(-4px);
    border-color: var(--color-primary);
    box-shadow: 0 8px 24px rgba(0, 230, 118, 0.15);
  }

  .skill-card-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--space-1);
  }

  .category-tag {
    font-size: 0.68rem;
    font-weight: 700;
    color: var(--color-muted);
    background: var(--color-surface-soft);
    padding: 2px 8px;
    border-radius: 99px;
  }

  .skill-card-controls {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .toggle-switch {
    width: 28px;
    height: 16px;
    cursor: pointer;
  }

  .delete-btn {
    border: 0;
    background: transparent;
    cursor: pointer;
    color: var(--color-muted);
  }

  .delete-btn:hover {
    color: var(--color-danger);
  }

  .skill-card h4 {
    margin: 4px 0;
    font-family: 'Hanken Grotesk', sans-serif;
    font-size: 1.05rem;
    color: var(--color-ink);
  }

  .skill-description {
    margin: 4px 0 var(--space-2);
    color: var(--color-muted);
    font-size: 0.85rem;
    line-height: 1.4;
    display: -webkit-box;
    -webkit-line-clamp: 3;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  .skill-card-footer {
    margin-top: auto;
    display: flex;
    gap: 8px;
    font-size: 0.72rem;
    color: var(--color-muted);
  }

  /* Modals */
  .modal-overlay {
    position: fixed;
    z-index: 100;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.4);
    display: flex;
    justify-content: center;
    align-items: center;
    backdrop-filter: blur(8px);
  }

  @keyframes modalZoom {
    from {
      opacity: 0;
      transform: scale(0.95);
    }
    to {
      opacity: 1;
      transform: scale(1);
    }
  }

  .modal-body {
    background: var(--color-surface);
    border: 1px solid var(--color-outline);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-card);
    width: 75vw;
    height: 75vh;
    display: flex;
    flex-direction: column;
    animation: modalZoom 0.25s cubic-bezier(0.34, 1.56, 0.64, 1) forwards;
  }

  .modal-body.compact-modal {
    width: 32rem;
    height: auto;
  }

  .modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--space-2) var(--space-3);
    border-bottom: 1px solid var(--color-outline);
  }

  .modal-header h3 {
    margin: 0;
    font-family: 'Hanken Grotesk', sans-serif;
  }

  .close-btn {
    border: 0;
    background: transparent;
    cursor: pointer;
    color: var(--color-muted);
  }

  .modal-grid-content {
    display: grid;
    grid-template-columns: 2fr 1fr;
    flex: 1;
    overflow: hidden;
  }

  .modal-markdown-area {
    padding: var(--space-3);
    overflow-y: auto;
    border-right: 1px solid var(--color-outline);
  }

  .modal-markdown-area h1 {
    margin-top: 0;
    font-family: 'Hanken Grotesk', sans-serif;
  }

  .markdown-body {
    font-size: 0.95rem;
    line-height: 1.6;
    color: var(--color-ink);
  }

  .markdown-body table {
    width: 100%;
    border-collapse: collapse;
    margin: var(--space-2) 0;
  }

  .markdown-body th, .markdown-body td {
    border: 1px solid var(--color-outline);
    padding: 8px;
    text-align: left;
  }

  .markdown-body th {
    background: var(--color-surface-soft);
  }

  .modal-meta-editor {
    padding: var(--space-3);
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    background: var(--color-canvas);
  }

  .form-group {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .form-group.flex-fill {
    flex: 1;
  }

  .form-group label {
    font-size: 0.75rem;
    font-weight: 700;
    color: var(--color-muted);
    text-transform: uppercase;
  }

  .form-group select, .form-group input, .form-group textarea {
    border: 1px solid var(--color-outline);
    border-radius: var(--radius-sm);
    padding: 0.6rem;
    background: var(--color-surface);
    color: var(--color-ink);
    font-size: 0.88rem;
  }

  .form-group textarea {
    flex: 1;
    resize: none;
  }

  .toggle-container {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .actions-footer {
    display: flex;
    justify-content: flex-end;
    gap: var(--space-1);
    margin-top: auto;
  }

  .tabs-header {
    display: flex;
    border-bottom: 1px solid var(--color-outline);
  }

  .tabs-header button {
    flex: 1;
    border: 0;
    background: transparent;
    padding: 12px;
    cursor: pointer;
    display: flex;
    justify-content: center;
    align-items: center;
    gap: 8px;
    font-size: 0.9rem;
    color: var(--color-muted);
    border-bottom: 2px solid transparent;
  }

  .tabs-header button.active-tab {
    color: var(--color-primary-ink);
    border-bottom-color: var(--color-primary);
    font-weight: 700;
  }

  .import-form {
    padding: var(--space-3);
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }
  ```

- [ ] **Step 5: Run typescript checks**
  Run: `npm run build`
  Expected: PASS

- [ ] **Step 6: Commit**
  ```bash
  git add src/features/skills/components/
  git commit -m "feat: implement SkillCard, SkillDetailModal, ImportSkillModal, and CSS layouts"
  ```

---

### Task 9: Assemble SkillsPage Integration

**Files:**
- Modify: `src/features/skills/SkillsPage.tsx`

**Interfaces:**
- Consumes: All UI components (Sidebar, Card Grid, Detail Modal, Import Modal).
- Produces: Integrated stateful dashboard for skills catalog management.

- [ ] **Step 1: Wire up state, queries, and mutators in SkillsPage**
  Modify `src/features/skills/SkillsPage.tsx` using `@tanstack/react-query` to fetch categories and skills, and invoke mutations:
  ```tsx
  import React, { useState } from 'react';
  import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
  import { Plus } from 'lucide-react';
  import {
    getSkills,
    getCategories,
    createCategory,
    renameCategory,
    deleteCategory,
    updateSkillMeta,
    deleteSkill,
    importSkill,
  } from '../../shared/api/tauriClient';
  import { SkillsSidebar } from './components/SkillsSidebar';
  import { SkillCard } from './components/SkillCard';
  import { SkillDetailModal } from './components/SkillDetailModal';
  import { ImportSkillModal } from './components/ImportSkillModal';
  import { Skill } from '../../shared/api/types';
  import './components/skills.css';

  export function SkillsPage() {
    const queryClient = useQueryClient();
    const [search, setSearch] = useState('');
    const [selectedCategoryId, setSelectedCategoryId] = useState<string | null>(null);
    const [activeDetailSkill, setActiveDetailSkill] = useState<Skill | null>(null);
    const [isImportOpen, setIsImportOpen] = useState(false);

    // Queries
    const { data: skills = [], isLoading: skillsLoading } = useQuery({
      queryKey: ['skills'],
      queryFn: getSkills,
    });

    const { data: categories = [], isLoading: catsLoading } = useQuery({
      queryKey: ['categories'],
      queryFn: getCategories,
    });

    // Mutations
    const updateMetaMut = useMutation({
      mutationFn: ({ id, cat, notes, enabled }: { id: string; cat: string | null; notes: string | null; enabled: boolean }) =>
        updateSkillMeta(id, cat, notes, enabled),
      onSuccess: () => queryClient.invalidateQueries({ queryKey: ['skills'] }),
    });

    const deleteSkillMut = useMutation({
      mutationFn: deleteSkill,
      onSuccess: () => queryClient.invalidateQueries({ queryKey: ['skills'] }),
    });

    const importSkillMut = useMutation({
      mutationFn: ({ source, type }: { source: string; type: 'folder' | 'git' }) => importSkill(source, type),
      onSuccess: () => queryClient.invalidateQueries({ queryKey: ['skills'] }),
    });

    const createCatMut = useMutation({
      mutationFn: createCategory,
      onSuccess: () => queryClient.invalidateQueries({ queryKey: ['categories'] }),
    });

    const renameCatMut = useMutation({
      mutationFn: ({ id, name }: { id: string; name: string }) => renameCategory(id, name),
      onSuccess: () => {
        queryClient.invalidateQueries({ queryKey: ['categories'] });
        queryClient.invalidateQueries({ queryKey: ['skills'] });
      },
    });

    const deleteCatMut = useMutation({
      mutationFn: deleteCategory,
      onSuccess: () => {
        queryClient.invalidateQueries({ queryKey: ['categories'] });
        queryClient.invalidateQueries({ queryKey: ['skills'] });
      },
    });

    if (skillsLoading || catsLoading) {
      return (
        <div className="page-state">
          <div className="loading-dot" />
          <p>加载技能目录...</p>
        </div>
      );
    }

    // Process counts for sidebar
    const skillsCountMap: Record<string, number> = { all: skills.length };
    let uncategorizedCount = 0;
    skills.forEach((s) => {
      if (!s.category_id) {
        uncategorizedCount++;
      } else {
        skillsCountMap[s.category_id] = (skillsCountMap[s.category_id] || 0) + 1;
      }
    });
    skillsCountMap['uncategorized'] = uncategorizedCount;

    // Filter skills
    const filteredSkills = skills.filter((s) => {
      // Category filter
      if (selectedCategoryId === 'uncategorized' && s.category_id) return false;
      if (selectedCategoryId !== null && selectedCategoryId !== 'uncategorized' && s.category_id !== selectedCategoryId) return false;
      
      // Search text filter
      if (search.trim()) {
        const query = search.toLowerCase();
        const nameMatch = s.metadata.name.toLowerCase().includes(query);
        const descMatch = s.metadata.description.toLowerCase().includes(query);
        return nameMatch || descMatch;
      }
      return true;
    });

    const getCategoryName = (catId?: string) => {
      if (!catId) return '未分类';
      return categories.find((c) => c.id === catId)?.name || '未分类';
    };

    return (
      <div className="page-stack">
        <header className="page-header">
          <div>
            <p className="eyebrow">CAPABILITY CATALOG</p>
            <h1>Skills 管理</h1>
            <p className="page-description">管理全局 AI 技能，自定义分类并将其启用至平台。</p>
          </div>
        </header>

        <div className="content-grid" style={{ gridTemplateColumns: '16rem 1fr' }}>
          <SkillsSidebar
            categories={categories}
            skillsCountMap={skillsCountMap}
            selectedCategoryId={selectedCategoryId}
            onSelectCategory={setSelectedCategoryId}
            onCreateCategory={(name) => createCatMut.mutate(name)}
            onRenameCategory={(id, name) => renameCatMut.mutate({ id, name })}
            onDeleteCategory={(id) => deleteCatMut.mutate(id)}
          />

          <main className="skills-main-area">
            <div className="skills-toolbar">
              <input
                className="search-input"
                placeholder="搜索技能名称或描述..."
                value={search}
                onChange={(e) => setSearch(e.target.value)}
              />
              <button className="button button--primary" onClick={() => setIsImportOpen(true)}>
                <Plus size={16} style={{ marginRight: '8px', verticalAlign: 'middle' }} />
                导入技能
              </button>
            </div>

            {filteredSkills.length === 0 ? (
              <div className="page-state">
                <p>没有找到匹配的技能</p>
              </div>
            ) : (
              <div className="skills-cards-grid">
                {filteredSkills.map((s) => (
                  <SkillCard
                    key={s.id}
                    skill={s}
                    categoryName={getCategoryName(s.category_id)}
                    onOpenDetail={() => setActiveDetailSkill(s)}
                    onToggleEnable={(e) => {
                      e.stopPropagation();
                      updateMetaMut.mutate({
                        id: s.id,
                        cat: s.category_id || null,
                        notes: s.user_notes || null,
                        enabled: !s.is_enabled,
                      });
                    }}
                    onDelete={(e) => {
                      e.stopPropagation();
                      if (confirm(`确定要删除技能 "${s.metadata.name}" 吗？此操作物理删除本地文件且不可逆。`)) {
                        deleteSkillMut.mutate(s.id);
                      }
                    }}
                  />
                ))}
              </div>
            )}
          </main>
        </div>

        {activeDetailSkill && (
          <SkillDetailModal
            skill={activeDetailSkill}
            categories={categories}
            onClose={() => setActiveDetailSkill(null)}
            onUpdate={(cat, notes, enabled) =>
              updateMetaMut.mutate({ id: activeDetailSkill.id, cat, notes, enabled })
            }
          />
        )}

        {isImportOpen && (
          <ImportSkillModal
            onClose={() => setIsImportOpen(false)}
            onImport={(source, type) => importSkillMut.mutate({ source, type })}
          />
        )}
      </div>
    );
  }
  ```

- [ ] **Step 2: Run frontend linter and Vitest**
  Run: `npm run lint` and `npm run test:run`
  Expected: PASS

- [ ] **Step 3: Commit**
  ```bash
  git add src/features/skills/SkillsPage.tsx
  git commit -m "feat: complete frontend SkillsPage integration with state and mutations"
  ```

---

## 7. Verification Plan

### Automated Tests
- Run Rust tests: `cargo test --manifest-path src-tauri/Cargo.toml`
- Run Frontend vitest tests: `npm run test:run`

### Manual Verification
1. Open the dev server: `npm run tauri:dev`
2. Test Category Management:
   - Click "新建分类" and type "Debugging". Verify it is appended to the sidebar list.
   - Edit the newly created category. Verify you can rename it.
   - Delete the category.
3. Test Skill Imports:
   - Create a local test skill folder at `/tmp/test-skill/` containing a `SKILL.md` file with a valid YAML Frontmatter.
   - In App UI, click "导入技能", input `/tmp/test-skill` in folder path, and submit.
   - Verify that the card displays in the grid, and files copied to `~/.agent-forge/skills/test-skill/`.
   - Test importing from a public git repo (e.g. `https://github.com/chenkai/some-test-skill.git`).
4. Test Modifying Skill Metadata:
   - Click a skill card. Check that the detail modal zooms up occupying 3/4 of the viewport.
   - Change the category dropdown to "Debugging".
   - Type a personal note in the notes text area, and click save.
   - Verify that the badge on the card updates, and reopening the modal displays the same notes.
5. Test Toggle Enabled & Deletion:
   - Click the Switch checkbox on a card. Check that is_enabled state persists in database.
   - Click the trash bin delete icon. Confirm. Verify that the skill is deleted from disk and the card disappears.
