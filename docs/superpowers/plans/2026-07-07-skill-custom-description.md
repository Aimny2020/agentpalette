# Skills 自定义技能说明设计 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement a user-maintained "技能说明" (custom description) field for skill packages and sub-skills in AgentForge. This custom description is stored in a separate SQLite table to survive skill scans, updates, and uninstalls, and it overrides the author's description in the UI. Users can search by this field, edit it in the details modal, and export/import it via versioned JSON.

**Architecture:**
- **Database Layer**: A new SQLite table `skill_descriptions` mapping `target_id` (package or sub-skill) to custom description and update timestamp, with no foreign key constraints.
- **IPC Layer**: Tauri commands to get, save, export, preview import, confirm import, count, and clear unassociated descriptions.
- **Frontend Layer**: React components rendering the customized badge, details sidebar edit field, import/export controls, search filters, and an import preview modal.

**Tech Stack:**
- **Backend**: Rust, Tauri v2, SQLite (`rusqlite`), `rfd` (file dialogs), `chrono`
- **Frontend**: TypeScript, React, `@tanstack/react-query`, `lucide-react`
- **Testing**: Rust unit tests, Vitest + `@testing-library/react`

## Global Constraints
- Two-space indentation in TypeScript and four spaces in Rust.
- React components and files use `PascalCase`; hooks and utilities use `camelCase`; Rust modules and functions use `snake_case`.
- SQLite is the unique runtime datasource.
- Single description saves and batch imports must use transactions.
- Descriptions are plain text, max 2000 Unicode characters. Blank strings are deleted, not saved.
- Import conflict strategies: default "keep_newer" (compare `updated_at`), optional "keep_local" or "keep_import".

---

### Task 1: Database Migration & Domain Definition

**Files:**
- Create: `src-tauri/migrations/004_skill_descriptions.sql`
- Modify: `src-tauri/src/infrastructure/database.rs:13-50`
- Modify: `src-tauri/src/domain/skill.rs:27-34`
- Modify: `src-tauri/src/domain/skill.rs:119-148`

**Interfaces:**
- Produces: `Skill` and `SkillMember` struct fields `custom_description: Option<String>`
- Produces: `SkillDescriptionRecord` struct

- [ ] **Step 1: Create the SQL migration file**
  Create `src-tauri/migrations/004_skill_descriptions.sql` with:
  ```sql
  CREATE TABLE IF NOT EXISTS skill_descriptions (
      target_id TEXT PRIMARY KEY NOT NULL,
      target_kind TEXT NOT NULL CHECK (target_kind IN ('package', 'member')),
      custom_description TEXT NOT NULL,
      updated_at TEXT NOT NULL
  );

  INSERT OR IGNORE INTO _migrations (version) VALUES (4);
  ```

- [ ] **Step 2: Update migrations loading in `database.rs`**
  Modify `src-tauri/src/infrastructure/database.rs` to include and execute `004_skill_descriptions.sql`:
  ```rust
  const SKILL_DESCRIPTIONS_MIGRATION: &str = include_str!("../../migrations/004_skill_descriptions.sql");
  ```
  And in `initialize`:
  ```rust
          connection
              .execute_batch(SKILL_PACKAGES_MIGRATION)
              .map_err(database_error)?;
          connection
              .execute_batch(SKILL_DESCRIPTIONS_MIGRATION)
              .map_err(database_error)?;
          Ok(Self {
              connection: Mutex::new(connection),
          })
  ```

- [ ] **Step 3: Update `SkillMember` and `Skill` domain models**
  Modify `src-tauri/src/domain/skill.rs` to add `custom_description` and define `SkillDescriptionRecord` for JSON serialization:
  ```rust
  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct SkillMember {
      pub id: String,
      pub relative_path: String,
      pub metadata: SkillMetadata,
      pub html_content: String,
      pub custom_description: Option<String>,
  }

  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct Skill {
      pub id: String,
      pub kind: SkillKind,
      pub metadata: SkillMetadata,
      pub html_content: String,
      pub members: Vec<SkillMember>,
      pub category_id: Option<String>,
      pub user_notes: Option<String>,
      pub source: SkillSourceInfo,
      pub update_status: UpdateStatus,
      pub available_commit: Option<String>,
      pub has_executable_content: bool,
      pub trusted: bool,
      pub warnings: Vec<String>,
      pub custom_description: Option<String>,
  }

  #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
  pub struct SkillDescriptionRecord {
      pub target_id: String,
      pub target_kind: String, // 'package' or 'member'
      #[serde(rename = "description")]
      pub custom_description: String,
      pub updated_at: String,
  }
  ```

- [ ] **Step 4: Fix `skill_scanner.rs` compilation**
  Modify `src-tauri/src/application/skill_scanner.rs:125-133` to specify `custom_description: None` during initialization:
  ```rust
      let members = parsed
          .into_iter()
          .map(|skill| SkillMember {
              id: member_id(id, &skill.relative_directory),
              relative_path: skill.relative_directory,
              metadata: skill.metadata,
              html_content: skill.html_content,
              custom_description: None,
          })
          .collect();
  ```

---

### Task 2: Repository Port Implementation

**Files:**
- Modify: `src-tauri/src/domain/ports.rs:18-71`
- Modify: `src-tauri/src/infrastructure/database.rs:18-25` (repository impl area)
- Modify: `src-tauri/src/infrastructure/database.rs:480-515` (unit tests)

**Interfaces:**
- Produces: `SkillRepository` methods: `get_custom_description`, `save_custom_description`, `delete_custom_description`, `get_all_custom_descriptions`, `import_custom_descriptions`, `delete_descriptions`

- [ ] **Step 1: Declare repository port methods**
  Modify `src-tauri/src/domain/ports.rs` to add methods:
  ```rust
      fn get_custom_description(&self, target_id: &str) -> DomainResult<Option<String>>;
      fn save_custom_description(
          &self,
          target_id: &str,
          target_kind: &str,
          custom_description: &str,
      ) -> DomainResult<()>;
      fn delete_custom_description(&self, target_id: &str) -> DomainResult<()>;
      fn get_all_custom_descriptions(&self) -> DomainResult<Vec<crate::domain::skill::SkillDescriptionRecord>>;
      fn import_custom_descriptions(
          &self,
          records: Vec<crate::domain::skill::SkillDescriptionRecord>,
          conflict_strategy: &str,
      ) -> DomainResult<()>;
      fn delete_descriptions(&self, target_ids: &[String]) -> DomainResult<()>;
  ```

- [ ] **Step 2: Implement repository methods in `database.rs`**
  Modify `src-tauri/src/infrastructure/database.rs` to implement the new methods:
  ```rust
      fn get_custom_description(&self, target_id: &str) -> DomainResult<Option<String>> {
          let connection = self
              .connection
              .lock()
              .map_err(|error| DomainError::Database(error.to_string()))?;
          let row = connection
              .query_row(
                  "SELECT custom_description FROM skill_descriptions WHERE target_id = ?1",
                  [target_id],
                  |r| r.get::<_, String>(0),
              )
              .optional()
              .map_err(database_error)?;
          Ok(row)
      }

      fn save_custom_description(
          &self,
          target_id: &str,
          target_kind: &str,
          custom_description: &str,
      ) -> DomainResult<()> {
          let mut connection = self
              .connection
              .lock()
              .map_err(|error| DomainError::Database(error.to_string()))?;
          let now = chrono::Utc::now().to_rfc3339();
          let tx = connection.transaction().map_err(database_error)?;
          tx.execute(
              "INSERT INTO skill_descriptions (target_id, target_kind, custom_description, updated_at)
               VALUES (?1, ?2, ?3, ?4)
               ON CONFLICT(target_id) DO UPDATE SET
                 custom_description = excluded.custom_description,
                 updated_at = excluded.updated_at",
              rusqlite::params![target_id, target_kind, custom_description, now],
          )
          .map_err(database_error)?;
          tx.commit().map_err(database_error)?;
          Ok(())
      }

      fn delete_custom_description(&self, target_id: &str) -> DomainResult<()> {
          let mut connection = self
              .connection
              .lock()
              .map_err(|error| DomainError::Database(error.to_string()))?;
          let tx = connection.transaction().map_err(database_error)?;
          tx.execute("DELETE FROM skill_descriptions WHERE target_id = ?1", [target_id]).map_err(database_error)?;
          tx.commit().map_err(database_error)?;
          Ok(())
      }

      fn get_all_custom_descriptions(&self) -> DomainResult<Vec<crate::domain::skill::SkillDescriptionRecord>> {
          let connection = self
              .connection
              .lock()
              .map_err(|error| DomainError::Database(error.to_string()))?;
          let mut stmt = connection
              .prepare("SELECT target_id, target_kind, custom_description, updated_at FROM skill_descriptions ORDER BY target_id ASC")
              .map_err(database_error)?;
          let iter = stmt
              .query_map([], |row| {
                  Ok(crate::domain::skill::SkillDescriptionRecord {
                      target_id: row.get(0)?,
                      target_kind: row.get(1)?,
                      custom_description: row.get(2)?,
                      updated_at: row.get(3)?,
                  })
              })
              .map_err(database_error)?;
          let mut list = Vec::new();
          for item in iter {
              list.push(item.map_err(database_error)?);
          }
          Ok(list)
      }

      fn import_custom_descriptions(
          &self,
          records: Vec<crate::domain::skill::SkillDescriptionRecord>,
          conflict_strategy: &str,
      ) -> DomainResult<()> {
          let mut connection = self
              .connection
              .lock()
              .map_err(|error| DomainError::Database(error.to_string()))?;
          let tx = connection.transaction().map_err(database_error)?;
          for record in records {
              let local_updated_at: Option<String> = tx
                  .query_row(
                      "SELECT updated_at FROM skill_descriptions WHERE target_id = ?1",
                      [&record.target_id],
                      |r| r.get(0),
                  )
                  .optional()
                  .map_err(database_error)?;
              
              let write_record = match local_updated_at {
                  None => true,
                  Some(local_time) => match conflict_strategy {
                      "keep_local" => false,
                      "keep_import" => true,
                      _ => { // "keep_newer" (default)
                          if let (Ok(import_dt), Ok(local_dt)) = (
                              chrono::DateTime::parse_from_rfc3339(&record.updated_at),
                              chrono::DateTime::parse_from_rfc3339(&local_time),
                          ) {
                              import_dt > local_dt
                          } else {
                              record.updated_at > local_time
                          }
                      }
                  },
              };

              if write_record {
                  tx.execute(
                      "INSERT INTO skill_descriptions (target_id, target_kind, custom_description, updated_at)
                       VALUES (?1, ?2, ?3, ?4)
                       ON CONFLICT(target_id) DO UPDATE SET
                         custom_description = excluded.custom_description,
                         updated_at = excluded.updated_at",
                      rusqlite::params![
                          record.target_id,
                          record.target_kind,
                          record.custom_description,
                          record.updated_at
                      ],
                  )
                  .map_err(database_error)?;
              }
          }
          tx.commit().map_err(database_error)?;
          Ok(())
      }

      fn delete_descriptions(&self, target_ids: &[String]) -> DomainResult<()> {
          let mut connection = self
              .connection
              .lock()
              .map_err(|error| DomainError::Database(error.to_string()))?;
          let tx = connection.transaction().map_err(database_error)?;
          for id in target_ids {
              tx.execute("DELETE FROM skill_descriptions WHERE target_id = ?1", [id])
                  .map_err(database_error)?;
          }
          tx.commit().map_err(database_error)?;
          Ok(())
      }
  ```

- [ ] **Step 3: Write Rust unit tests for `database.rs`**
  Add unit tests at the end of `database.rs`:
  ```rust
      #[test]
      fn manages_custom_descriptions_lifecycle() {
          let database = SqliteDatabase::open_in_memory().unwrap();
          
          // Initially empty
          assert_eq!(database.get_custom_description("t-1").unwrap(), None);

          // Save new
          database.save_custom_description("t-1", "package", "My Test Package").unwrap();
          assert_eq!(database.get_custom_description("t-1").unwrap(), Some("My Test Package".into()));

          // Update existing
          database.save_custom_description("t-1", "package", "My Updated Package").unwrap();
          assert_eq!(database.get_custom_description("t-1").unwrap(), Some("My Updated Package".into()));

          // Delete
          database.delete_custom_description("t-1").unwrap();
          assert_eq!(database.get_custom_description("t-1").unwrap(), None);
      }
  ```

- [ ] **Step 4: Run Rust tests**
  Command: `cargo test --manifest-path src-tauri/Cargo.toml`
  Expected: PASS

---

### Task 3: Service Layer Integration

**Files:**
- Modify: `src-tauri/src/application/skill_service.rs:308-397` (populate custom descriptions in `get_skills`)

**Interfaces:**
- Produces: `SkillService::get_skills()` returning skills populated with custom descriptions from SQLite.

- [ ] **Step 1: Update `get_skills` in `skill_service.rs`**
  Modify `src-tauri/src/application/skill_service.rs` to read description databases when compiling lists:
  ```rust
              if let Ok(mut discovered) = scan_skill_root(&skill_id, &path) {
                  let (category_id, user_notes) = match self.repo.get_user_meta(&skill_id)? {
                      Some(meta) => (meta.category_id, meta.user_notes),
                      None => (None, None),
                  };
                  let custom_description = self.repo.get_custom_description(&skill_id)?;
                  let mut members = Vec::new();
                  for member in discovered.members {
                      let m_desc = self.repo.get_custom_description(&member.id)?;
                      members.push(SkillMember {
                          id: member.id,
                          relative_path: member.relative_path,
                          metadata: member.metadata,
                          html_content: member.html_content,
                          custom_description: m_desc,
                      });
                  }
  ```
  Update `discovered.members` in the `Skill` initializer at the end of the loop:
  ```rust
                  list.push(Skill {
                      id: discovered.id,
                      kind: discovered.kind,
                      metadata: discovered.metadata,
                      html_content: discovered.html_content,
                      members, // Uses the populated list
                      category_id,
                      user_notes,
                      source,
                      update_status,
                      available_commit: None,
                      has_executable_content: discovered.has_executable_content,
                      trusted,
                      warnings: discovered.warnings,
                      custom_description,
                  });
  ```

- [ ] **Step 2: Verify build**
  Run: `cargo check --manifest-path src-tauri/Cargo.toml`
  Expected: Success

---

### Task 4: Tauri Commands Definition & Registration

**Files:**
- Modify: `src-tauri/src/commands/skills.rs` (add description commands)
- Modify: `src-tauri/src/lib.rs:40-61` (register commands)

**Interfaces:**
- Produces: IPC handlers: `save_custom_description`, `export_custom_descriptions`, `preview_custom_descriptions_import`, `confirm_custom_descriptions_import`, `get_unassociated_descriptions_count`, `clear_unassociated_descriptions`

- [ ] **Step 1: Implement commands in `src-tauri/src/commands/skills.rs`**
  Add the following serialization helpers and commands to the end of `src-tauri/src/commands/skills.rs`:
  ```rust
  #[derive(serde::Serialize, serde::Deserialize)]
  pub struct SkillDescriptionsExport {
      pub schema_version: u32,
      pub exported_at: String,
      pub descriptions: Vec<crate::domain::skill::SkillDescriptionRecord>,
  }

  #[derive(serde::Serialize, serde::Deserialize)]
  pub struct InvalidRecordInfo {
      pub target_id: Option<String>,
      pub target_kind: Option<String>,
      pub description: Option<String>,
      pub reason: String,
  }

  #[derive(serde::Serialize, serde::Deserialize)]
  pub struct DescriptionsImportPreview {
      pub file_path: String,
      pub total_count: usize,
      pub new_count: usize,
      pub overwrite_count: usize,
      pub skip_count: usize,
      pub unassociated_count: usize,
      pub invalid_records: Vec<InvalidRecordInfo>,
      pub valid_records: Vec<crate::domain::skill::SkillDescriptionRecord>,
  }

  #[tauri::command]
  pub async fn save_custom_description(
      state: State<'_, AppState>,
      target_id: String,
      target_kind: String,
      description: Option<String>,
  ) -> Result<(), CommandError> {
      let trimmed = description.unwrap_or_default().trim().to_string();
      if trimmed.is_empty() {
          state.repo.delete_custom_description(&target_id).map_err(CommandError::from)?;
          return Ok(());
      }
      if trimmed.chars().count() > 2000 {
          return Err(CommandError::Database("技能说明不能超过 2000 个字".into()));
      }
      if target_kind != "package" && target_kind != "member" {
          return Err(CommandError::Database("非法的 target_kind".into()));
      }
      state.repo.save_custom_description(&target_id, &target_kind, &trimmed).map_err(CommandError::from)
  }

  #[tauri::command]
  pub async fn export_custom_descriptions(
      state: State<'_, AppState>,
  ) -> Result<Option<String>, CommandError> {
      let descriptions = state.repo.get_all_custom_descriptions().map_err(CommandError::from)?;
      let export_data = SkillDescriptionsExport {
          schema_version: 1,
          exported_at: chrono::Utc::now().to_rfc3339(),
          descriptions,
      };
      let json_str = serde_json::to_string_pretty(&export_data).map_err(|e| CommandError::Database(e.to_string()))?;

      let file_path = rfd::AsyncFileDialog::new()
          .set_title("导出技能说明")
          .add_filter("JSON", &["json"])
          .save_file()
          .await;

      if let Some(path) = file_path {
          let path_buf = path.path().to_path_buf();
          std::fs::write(&path_buf, json_str).map_err(|e| CommandError::Database(e.to_string()))?;
          Ok(Some(path_buf.to_string_lossy().to_string()))
      } else {
          Ok(None)
      }
  }

  #[tauri::command]
  pub async fn preview_custom_descriptions_import(
      state: State<'_, AppState>,
  ) -> Result<Option<DescriptionsImportPreview>, CommandError> {
      let file_handle = rfd::AsyncFileDialog::new()
          .set_title("选择导入的技能说明文件")
          .add_filter("JSON", &["json"])
          .pick_file()
          .await;

      let file = match file_handle {
          Some(f) => f,
          None => return Ok(None),
      };

      let path_buf = file.path().to_path_buf();
      let file_path_str = path_buf.to_string_lossy().to_string();
      let bytes = std::fs::read(&path_buf).map_err(|e| CommandError::Database(e.to_string()))?;
      
      let import_data: SkillDescriptionsExport = match serde_json::from_slice(&bytes) {
          Ok(data) => data,
          Err(e) => return Err(CommandError::Database(format!("解析 JSON 失败: {e}"))),
      };

      if import_data.schema_version != 1 {
          return Err(CommandError::Database(format!("不支持的 schema_version: {}，当前仅支持版本 1", import_data.schema_version)));
      }

      // Collect installed ids to determine if unassociated
      let installed_skills = state.skills.get_skills().map_err(CommandError::from)?;
      let mut installed_ids = std::collections::HashSet::new();
      for s in installed_skills {
          installed_ids.insert(s.id.clone());
          for m in s.members {
              installed_ids.insert(m.id.clone());
          }
      }

      let mut preview = DescriptionsImportPreview {
          file_path: file_path_str,
          total_count: import_data.descriptions.len(),
          new_count: 0,
          overwrite_count: 0,
          skip_count: 0,
          unassociated_count: 0,
          invalid_records: Vec::new(),
          valid_records: Vec::new(),
      };

      let mut seen_ids = std::collections::HashSet::new();

      for record in import_data.descriptions {
          let mut invalid = false;
          let mut reason = String::new();

          if record.target_id.trim().is_empty() {
              invalid = true;
              reason = "target_id 不能为空".into();
          } else if seen_ids.contains(&record.target_id) {
              invalid = true;
              reason = format!("重复的 target_id: {}", record.target_id);
          } else if record.target_kind != "package" && record.target_kind != "member" {
              invalid = true;
              reason = "target_kind 必须是 'package' 或 'member'".into();
          } else if record.custom_description.trim().is_empty() {
              invalid = true;
              reason = "说明内容不能为空".into();
          } else if record.custom_description.chars().count() > 2000 {
              invalid = true;
              reason = "说明内容超过 2000 个字符限制".into();
          } else if chrono::DateTime::parse_from_rfc3339(&record.updated_at).is_err() {
              invalid = true;
              reason = "更新时间 (updated_at) 格式非法，须为 RFC3339 格式".into();
          }

          if invalid {
              preview.invalid_records.push(InvalidRecordInfo {
                  target_id: Some(record.target_id),
                  target_kind: Some(record.target_kind),
                  description: Some(record.custom_description),
                  reason,
              });
              continue;
          }

          seen_ids.insert(record.target_id.clone());

          // Check association
          let is_associated = installed_ids.contains(&record.target_id);
          if !is_associated {
              preview.unassociated_count += 1;
          }

          // Check database conflict
          let local_updated_at = state.repo.get_custom_description(&record.target_id).map_err(CommandError::from)?;
          match local_updated_at {
              None => {
                  preview.new_count += 1;
              }
              Some(_) => {
                  preview.overwrite_count += 1; // Mark as overwrite/skip for conflict previewing in general
              }
          }
          preview.valid_records.push(record);
      }

      Ok(Some(preview))
  }

  #[tauri::command]
  pub async fn confirm_custom_descriptions_import(
      state: State<'_, AppState>,
      records: Vec<crate::domain::skill::SkillDescriptionRecord>,
      conflict_strategy: String,
  ) -> Result<(), CommandError> {
      state.repo.import_custom_descriptions(records, &conflict_strategy).map_err(CommandError::from)
  }

  #[tauri::command]
  pub async fn get_unassociated_descriptions_count(
      state: State<'_, AppState>,
  ) -> Result<usize, CommandError> {
      let descriptions = state.repo.get_all_custom_descriptions().map_err(CommandError::from)?;
      let installed_skills = state.skills.get_skills().map_err(CommandError::from)?;
      let mut installed_ids = std::collections::HashSet::new();
      for s in installed_skills {
          installed_ids.insert(s.id.clone());
          for m in s.members {
              installed_ids.insert(m.id.clone());
          }
      }

      let count = descriptions
          .iter()
          .filter(|d| !installed_ids.contains(&d.target_id))
          .count();
      Ok(count)
  }

  #[tauri::command]
  pub async fn clear_unassociated_descriptions(
      state: State<'_, AppState>,
  ) -> Result<usize, CommandError> {
      let descriptions = state.repo.get_all_custom_descriptions().map_err(CommandError::from)?;
      let installed_skills = state.skills.get_skills().map_err(CommandError::from)?;
      let mut installed_ids = std::collections::HashSet::new();
      for s in installed_skills {
          installed_ids.insert(s.id.clone());
          for m in s.members {
              installed_ids.insert(m.id.clone());
          }
      }

      let unassociated_ids: Vec<String> = descriptions
          .into_iter()
          .filter(|d| !installed_ids.contains(&d.target_id))
          .map(|d| d.target_id)
          .collect();

      let len = unassociated_ids.len();
      state.repo.delete_descriptions(&unassociated_ids).map_err(CommandError::from)?;
      Ok(len)
  }
  ```

- [ ] **Step 2: Register commands in `src-tauri/src/lib.rs`**
  Modify the `invoke_handler` call to add the 6 new commands:
  ```rust
          .invoke_handler(tauri::generate_handler![
              health_check,
              get_skills,
              import_skill,
              inspect_skill_import,
              delete_skill,
              check_skill_updates,
              update_skill,
              trust_skill,
              delete_skill_everywhere,
              update_skill_meta,
              get_project_skills,
              toggle_project_skill,
              get_categories,
              create_category,
              rename_category,
              delete_category,
              get_projects,
              add_project,
              select_directory,
              delete_project,
              // New description commands:
              save_custom_description,
              export_custom_descriptions,
              preview_custom_descriptions_import,
              confirm_custom_descriptions_import,
              get_unassociated_descriptions_count,
              clear_unassociated_descriptions
          ])
  ```

- [ ] **Step 3: Run Rust compilation & verification**
  Run: `cargo check --manifest-path src-tauri/Cargo.toml`
  Expected: Success

---

### Task 5: Frontend Shared Types & API Wrapper

**Files:**
- Modify: `src/shared/api/types.ts`
- Modify: `src/shared/api/tauriClient.ts`

**Interfaces:**
- Produces: API client client wrappers: `saveCustomDescription`, `exportCustomDescriptions`, `previewCustomDescriptionsImport`, `confirmCustomDescriptionsImport`, `getUnassociatedDescriptionsCount`, `clearUnassociatedDescriptions`

- [ ] **Step 1: Update API TypeScript declarations in `src/shared/api/types.ts`**
  Modify `src/shared/api/types.ts` to add custom description fields and import preview models:
  ```typescript
  export interface SkillMetadata {
    name: string;
    description: string;
    author?: string;
    version?: string;
  }

  export interface SkillMember {
    id: string;
    relative_path: string;
    metadata: SkillMetadata;
    html_content: string;
    custom_description?: string;
  }

  export interface Skill {
    id: string;
    kind: SkillKind;
    metadata: SkillMetadata;
    html_content: string;
    members: SkillMember[];
    category_id?: string;
    user_notes?: string;
    source: SkillSourceInfo;
    update_status: SkillUpdateStatus;
    available_commit?: string;
    has_executable_content: boolean;
    trusted: boolean;
    warnings: string[];
    custom_description?: string;
  }

  export interface SkillDescriptionRecord {
    target_id: string;
    target_kind: 'package' | 'member';
    description: string;
    updated_at: string;
  }

  export interface InvalidRecordInfo {
    target_id?: string;
    target_kind?: string;
    description?: string;
    reason: string;
  }

  export interface DescriptionsImportPreview {
    file_path: string;
    total_count: number;
    new_count: number;
    overwrite_count: number;
    skip_count: number;
    unassociated_count: number;
    invalid_records: InvalidRecordInfo[];
    valid_records: SkillDescriptionRecord[];
  }
  ```

- [ ] **Step 2: Add client wrappers in `src/shared/api/tauriClient.ts`**
  Append these commands wrappers to `src/shared/api/tauriClient.ts`:
  ```typescript
  export async function saveCustomDescription(
    targetId: string,
    targetKind: 'package' | 'member',
    description: string | null,
  ): Promise<void> {
    try {
      await invoke<void>('save_custom_description', { targetId, targetKind, description });
    } catch (error) {
      throw normalizeError(error);
    }
  }

  export async function exportCustomDescriptions(): Promise<string | null> {
    try {
      return await invoke<string | null>('export_custom_descriptions');
    } catch (error) {
      throw normalizeError(error);
    }
  }

  export async function previewCustomDescriptionsImport(): Promise<DescriptionsImportPreview | null> {
    try {
      return await invoke<DescriptionsImportPreview | null>('preview_custom_descriptions_import');
    } catch (error) {
      throw normalizeError(error);
    }
  }

  export async function confirmCustomDescriptionsImport(
    records: SkillDescriptionRecord[],
    conflictStrategy: 'keep_newer' | 'keep_local' | 'keep_import',
  ): Promise<void> {
    try {
      await invoke<void>('confirm_custom_descriptions_import', { records, conflictStrategy });
    } catch (error) {
      throw normalizeError(error);
    }
  }

  export async function getUnassociatedDescriptionsCount(): Promise<number> {
    try {
      return await invoke<number>('get_unassociated_descriptions_count');
    } catch (error) {
      throw normalizeError(error);
    }
  }

  export async function clearUnassociatedDescriptions(): Promise<number> {
    try {
      return await invoke<number>('clear_unassociated_descriptions');
    } catch (error) {
      throw normalizeError(error);
    }
  }
  ```

---

### Task 6: Frontend Search & Card Custom Description Rendering

**Files:**
- Modify: `src/features/skills/skillCatalog.ts:19-32`
- Modify: `src/features/skills/components/SkillCard.tsx:47-51`
- Modify: `src/features/projects/pages/ProjectSkillsPage.tsx:132-205`
- Modify: `src/features/skills/components/skills.css` (add badge style)

**Interfaces:**
- UI display: displays "自定义" badge and custom description when available.
- Search behaviour: searches skill name, original description, and custom description.

- [ ] **Step 1: Update search query matching in `skillCatalog.ts`**
  Modify `src/features/skills/skillCatalog.ts` to include `custom_description` matches:
  ```typescript
    return skills.flatMap((skill): CatalogResult[] => {
      if (!categoryMatches(skill)) return [];
      if (!query) return [{ type: 'skill', skill }];

      const customDesc = skill.custom_description || '';
      const parentText = `${skill.metadata.name} ${skill.metadata.description} ${customDesc}`.toLocaleLowerCase();
      if (parentText.includes(query)) return [{ type: 'skill', skill }];

      return skill.members
        .filter((member) => {
          const memberCustomDesc = member.custom_description || '';
          return `${member.metadata.name} ${member.metadata.description} ${memberCustomDesc}`
            .toLocaleLowerCase()
            .includes(query);
        })
        .map((member) => ({ type: 'member' as const, skill, member }));
    });
  ```

- [ ] **Step 2: Update custom badge style in `src/features/skills/components/skills.css`**
  Append custom badge CSS rules to `src/features/skills/components/skills.css`:
  ```css
  .custom-badge {
    display: inline-flex;
    align-items: center;
    font-size: 0.65rem;
    font-weight: 700;
    color: var(--color-primary-ink);
    background: color-mix(in srgb, var(--color-primary) 12%, transparent);
    border: 1px solid color-mix(in srgb, var(--color-primary) 30%, transparent);
    padding: 1px 5px;
    border-radius: 4px;
    margin-right: 6px;
    vertical-align: middle;
  }
  ```

- [ ] **Step 3: Update `SkillCard.tsx` rendering**
  Modify `src/features/skills/components/SkillCard.tsx` to read the custom description and show the badge:
  ```typescript
    const metadata = member?.metadata ?? skill.metadata;
    const customDescription = member ? member.custom_description : skill.custom_description;
  ```
  And render in the description element:
  ```tsx
        <h4>{metadata.name}</h4>
        <p className="skill-description">
          {customDescription ? (
            <>
              <span className="custom-badge">自定义</span>
              {customDescription}
            </>
          ) : (
            metadata.description
          )}
        </p>
  ```

- [ ] **Step 4: Update `ProjectSkillsPage.tsx` rendering**
  Modify `src/features/projects/pages/ProjectSkillsPage.tsx` so top-level and sub-skills respect the fallback sequence:
  In the top-level checklist description label (around lines 133-148):
  ```tsx
                      <label 
                        htmlFor={isUntrusted ? undefined : `skill-chk-${skill.id}`}
                        style={{ cursor: isUntrusted ? 'not-allowed' : 'pointer', userSelect: 'none', display: 'flex', flexDirection: 'column' }}
                      >
                        <strong>{skill.metadata.name}</strong>
                        {skill.custom_description ? (
                          <span className="project-skill-pack-label">
                            <span className="custom-badge">自定义</span>
                            {skill.custom_description}
                          </span>
                        ) : skill.metadata.description ? (
                          <span className="project-skill-pack-label">
                            {skill.metadata.description}
                          </span>
                        ) : null}
                        {isPack ? (
                          <span className="project-skill-pack-label" style={{ fontWeight: 'normal', color: 'var(--color-muted)' }}>
                            技能扩展包 · {enabledMembersCount}/{skill.members.length} 启用
                          </span>
                        ) : null}
                      </label>
  ```
  In the member list loop (around lines 195-202):
  ```tsx
                              <label 
                                htmlFor={isUntrusted ? undefined : `skill-chk-${member.id}`}
                                style={{ 
                                  cursor: isUntrusted ? 'not-allowed' : 'pointer', 
                                  userSelect: 'none', 
                                  display: 'flex', 
                                  flexDirection: 'column',
                                  fontSize: '0.85rem' 
                                }}
                              >
                                <strong style={{ color: 'var(--color-ink)', fontWeight: 500 }}>{member.metadata.name}</strong>
                                {member.custom_description ? (
                                  <span className="project-skill-pack-label" style={{ fontSize: '0.7rem' }}>
                                    <span className="custom-badge">自定义</span>
                                    {member.custom_description}
                                  </span>
                                ) : member.metadata.description ? (
                                  <span className="project-skill-pack-label" style={{ fontSize: '0.7rem' }}>
                                    {member.metadata.description}
                                  </span>
                                ) : null}
                              </label>
  ```

---

### Task 7: Details Sidebar Editor

**Files:**
- Modify: `src/features/skills/components/SkillDetailModal.tsx`

**Interfaces:**
- UI editor: Adds editable text area for `custom_description` with character counts.
- Warning behavior: warns when switching target or closing with unsaved changes.

- [ ] **Step 1: Modify `SkillDetailModal.tsx` editing and warning flow**
  Add state and handlers to track editing, warnings, and targets:
  Include `saveCustomDescription` in imports:
  ```typescript
  import { saveCustomDescription } from '../../../shared/api/tauriClient';
  ```
  Replace states and logic inside `SkillDetailModal`:
  ```typescript
  export function SkillDetailModal({
    skill,
    categories,
    onClose,
    onUpdate,
    initialMember,
    updateStatus = skill.update_status,
    onTrust,
    onInstallUpdate,
  }: Props) {
    const [notes, setNotes] = useState(skill.user_notes || '');
    const [catId, setCatId] = useState(skill.category_id || '');
    const [selectedMember, setSelectedMember] = useState<SkillMember | undefined>(initialMember);

    // Track custom descriptions for current editing target
    const getTargetInitialDesc = (member?: SkillMember) => {
      return member ? (member.custom_description || '') : (skill.custom_description || '');
    };
    
    const [customDesc, setCustomDesc] = useState(getTargetInitialDesc(initialMember));
    const [initialCustomDesc, setInitialCustomDesc] = useState(getTargetInitialDesc(initialMember));

    const isDescDirty = customDesc !== initialCustomDesc;
    const isMetaDirty = !selectedMember && (notes !== (skill.user_notes || '') || catId !== (skill.category_id || ''));
    const isLengthExceeded = customDesc.length > 2000;

    const checkDirtyAndProceed = () => {
      if (isDescDirty) {
        return window.confirm('您对“技能说明”的修改尚未保存，确定要放弃修改吗？');
      }
      return true;
    };

    const handleSwitchTarget = (nextMember: SkillMember | undefined) => {
      if (!checkDirtyAndProceed()) return;
      setSelectedMember(nextMember);
      const desc = getTargetInitialDesc(nextMember);
      setCustomDesc(desc);
      setInitialCustomDesc(desc);
    };

    const handleCloseAttempt = () => {
      if (isDescDirty || isMetaDirty) {
        if (!window.confirm('您有未保存的更改，确定要关闭并放弃所有更改吗？')) {
          return;
        }
      }
      onClose();
    };

    const handleSave = async () => {
      if (isLengthExceeded) return;
      try {
        if (selectedMember) {
          await saveCustomDescription(selectedMember.id, 'member', customDesc || null);
          selectedMember.custom_description = customDesc || undefined;
        } else {
          await onUpdate(catId || null, notes || null);
          await saveCustomDescription(skill.id, 'package', customDesc || null);
          skill.custom_description = customDesc || undefined;
          skill.category_id = catId || undefined;
          skill.user_notes = notes || undefined;
        }
        setInitialCustomDesc(customDesc);
        onClose();
      } catch (err) {
        alert(`保存失败: ${err instanceof Error ? err.message : String(err)}`);
      }
    };
  ```
  Ensure all target-switching and close events call the warning check hooks:
  - In modal overlay click and close button: use `handleCloseAttempt`.
  - In back button: use `() => handleSwitchTarget(undefined)`.
  - In member card click: use `() => handleSwitchTarget(member)`.

  Update the metadata editor UI layout in `SkillDetailModal`:
  ```tsx
            <div className="modal-meta-editor">
              {skill.warnings.length > 0 && (
                <div className="skill-warnings">
                  <strong>检测警告</strong>
                  {skill.warnings.map((warning) => <p key={warning}>{warning}</p>)}
                </div>
              )}
              
              {!selectedMember && (
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
              )}

              <div className="form-group">
                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                  <label style={{ margin: 0 }}>技能说明</label>
                  <span style={{ fontSize: '0.8rem', color: isLengthExceeded ? 'var(--color-danger, #cf222e)' : 'var(--color-muted)' }}>
                    {customDesc.length}/2000
                  </span>
                </div>
                <textarea
                  placeholder="添加中文技能说明，以快速说明用途并支持搜索（纯文本，限2000字）..."
                  value={customDesc}
                  onChange={(e) => setCustomDesc(e.target.value)}
                  style={{ minHeight: '8rem', maxHeight: '12rem', resize: 'vertical' }}
                />
                {isLengthExceeded && (
                  <span style={{ fontSize: '0.75rem', color: 'var(--color-danger, #cf222e)', marginTop: '4px' }}>
                    技能说明不能超过 2000 个字符
                  </span>
                )}
              </div>

              {!selectedMember && (
                <div className="form-group flex-fill">
                  <label>技能使用说明与备注</label>
                  <textarea
                    placeholder="在此添加该技能的个性化使用备注或说明..."
                    value={notes}
                    onChange={(e) => setNotes(e.target.value)}
                    style={{ minHeight: '10rem' }}
                  />
                </div>
              )}
  ```

---

### Task 8: Import Preview Modal & Descriptions Management UI

**Files:**
- Create: `src/features/skills/components/ImportDescriptionsModal.tsx`
- Modify: `src/features/skills/SkillsPage.tsx` (add buttons and backup dialog handlers)

**Interfaces:**
- UI modal: Displays import validation details, warnings, conflict rules selector, and transaction confirmation.
- UI tools: "导入说明", "导出说明", "清理未关联说明" buttons added.

- [ ] **Step 1: Create `ImportDescriptionsModal.tsx`**
  Implement the file `src/features/skills/components/ImportDescriptionsModal.tsx` to handle conflicts and preview:
  ```typescript
  import React, { useState } from 'react';
  import { X, ShieldAlert } from 'lucide-react';
  import { DescriptionsImportPreview } from '../../../shared/api/types';
  import { confirmCustomDescriptionsImport } from '../../../shared/api/tauriClient';

  interface Props {
    preview: DescriptionsImportPreview;
    onClose: () => void;
    onSuccess: () => void;
  }

  export function ImportDescriptionsModal({ preview, onClose, onSuccess }: Props) {
    const [strategy, setStrategy] = useState<'keep_newer' | 'keep_local' | 'keep_import'>('keep_newer');
    const [isSaving, setIsSaving] = useState(false);

    const handleConfirm = async () => {
      setIsSaving(true);
      try {
        await confirmCustomDescriptionsImport(preview.valid_records, strategy);
        onSuccess();
        onClose();
      } catch (err) {
        alert(`导入失败: ${err instanceof Error ? err.message : String(err)}`);
      } finally {
        setIsSaving(false);
      }
    };

    return (
      <div className="modal-overlay" onClick={onClose}>
        <div className="modal-body" onClick={(e) => e.stopPropagation()} style={{ maxWidth: '36rem' }}>
          <div className="modal-header">
            <h3>导入技能说明预览</h3>
            <button className="close-btn" onClick={onClose}>
              <X size={20} />
            </button>
          </div>
          
          <div className="modal-grid-content" style={{ display: 'flex', flexDirection: 'column', gap: '1rem', maxHeight: '70vh', overflowY: 'auto' }}>
            <p style={{ fontSize: '0.9rem', color: 'var(--color-ink)' }}>
              文件路径: <code style={{ wordBreak: 'break-all' }}>{preview.file_path}</code>
            </p>

            <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: '8px', textAlign: 'center' }}>
              <div style={{ padding: '8px', border: '1px solid var(--color-outline)', borderRadius: '6px' }}>
                <div style={{ fontSize: '1.2rem', fontWeight: 'bold' }}>{preview.new_count}</div>
                <div style={{ fontSize: '0.75rem', color: 'var(--color-muted)' }}>新增记录</div>
              </div>
              <div style={{ padding: '8px', border: '1px solid var(--color-outline)', borderRadius: '6px' }}>
                <div style={{ fontSize: '1.2rem', fontWeight: 'bold' }}>{preview.overwrite_count}</div>
                <div style={{ fontSize: '0.75rem', color: 'var(--color-muted)' }}>本地存在冲突</div>
              </div>
              <div style={{ padding: '8px', border: '1px solid var(--color-outline)', borderRadius: '6px' }}>
                <div style={{ fontSize: '1.2rem', fontWeight: 'bold' }}>{preview.unassociated_count}</div>
                <div style={{ fontSize: '0.75rem', color: 'var(--color-muted)' }}>当前未安装 Skill</div>
              </div>
            </div>

            {preview.invalid_records.length > 0 && (
              <div style={{ padding: '10px', border: '1px solid #ffccc7', background: '#fff2f0', borderRadius: '6px' }}>
                <h4 style={{ color: '#ff4d4f', display: 'flex', alignItems: 'center', gap: '4px', margin: '0 0 6px 0', fontSize: '0.85rem' }}>
                  <ShieldAlert size={14} /> 忽略无效记录 ({preview.invalid_records.length})
                </h4>
                <div style={{ maxHeight: '6rem', overflowY: 'auto', fontSize: '0.75rem', color: 'var(--color-ink)' }}>
                  {preview.invalid_records.map((r, i) => (
                    <div key={i} style={{ marginBottom: '4px', borderBottom: '1px dashed #ffa39e', paddingBottom: '4px' }}>
                      <strong>ID: {r.target_id || '未知'}</strong> ({r.reason})
                    </div>
                  ))}
                </div>
              </div>
            )}

            <div style={{ borderTop: '1px solid var(--color-outline)', paddingTop: '10px' }}>
              <h4 style={{ margin: '0 0 8px 0', fontSize: '0.9rem' }}>冲突处理策略</h4>
              <div style={{ display: 'flex', flexDirection: 'column', gap: '6px', fontSize: '0.85rem' }}>
                <label style={{ display: 'flex', alignItems: 'center', gap: '6px', cursor: 'pointer' }}>
                  <input
                    type="radio"
                    name="strategy"
                    checked={strategy === 'keep_newer'}
                    onChange={() => setStrategy('keep_newer')}
                  />
                  <span>保留较新记录 (比对本地和文件中的 updated_at 时间) - 推荐</span>
                </label>
                <label style={{ display: 'flex', alignItems: 'center', gap: '6px', cursor: 'pointer' }}>
                  <input
                    type="radio"
                    name="strategy"
                    checked={strategy === 'keep_local'}
                    onChange={() => setStrategy('keep_local')}
                  />
                  <span>保留本地 (跳过冲突记录)</span>
                </label>
                <label style={{ display: 'flex', alignItems: 'center', gap: '6px', cursor: 'pointer' }}>
                  <input
                    type="radio"
                    name="strategy"
                    checked={strategy === 'keep_import'}
                    onChange={() => setStrategy('keep_import')}
                  />
                  <span>使用导入文件 (全部覆盖)</span>
                </label>
              </div>
            </div>
            
            <div className="actions-footer" style={{ borderTop: '1px solid var(--color-outline)', paddingTop: '10px', marginTop: '10px', display: 'flex', justifyContent: 'flex-end', gap: '8px' }}>
              <button className="button button--secondary" onClick={onClose} disabled={isSaving}>
                取消
              </button>
              <button className="button button--primary" onClick={handleConfirm} disabled={isSaving || preview.valid_records.length === 0}>
                {isSaving ? '导入中...' : '确定导入'}
              </button>
            </div>
          </div>
        </div>
      </div>
    );
  }
  ```

- [ ] **Step 2: Add Backup and Clean Up controls to `SkillsPage.tsx`**
  Modify `src/features/skills/SkillsPage.tsx` to import the new commands and preview modal:
  ```typescript
  import {
    // Existing imports...
    exportCustomDescriptions,
    previewCustomDescriptionsImport,
    getUnassociatedDescriptionsCount,
    clearUnassociatedDescriptions
  } from '../../shared/api/tauriClient';
  import { ImportDescriptionsModal } from './components/ImportDescriptionsModal';
  ```
  Add component state inside `SkillsPage`:
  ```typescript
  const [importPreview, setImportPreview] = useState<DescriptionsImportPreview | null>(null);
  ```
  Implement the event handlers:
  ```typescript
  const handleExportDescriptions = async () => {
    try {
      const savedPath = await exportCustomDescriptions();
      if (savedPath) {
        alert(`技能说明备份成功，已存至:\n${savedPath}`);
      }
    } catch (err) {
      alert(`导出失败: ${err instanceof Error ? err.message : String(err)}`);
    }
  };

  const handleImportDescriptionsClick = async () => {
    try {
      const preview = await previewCustomDescriptionsImport();
      if (preview) {
        setImportPreview(preview);
      }
    } catch (err) {
      alert(`读取备份文件失败: ${err instanceof Error ? err.message : String(err)}`);
    }
  };

  const handleCleanUnassociated = async () => {
    try {
      const count = await getUnassociatedDescriptionsCount();
      if (count === 0) {
        alert('当前没有未关联的技能说明，无需清理。');
        return;
      }
      if (window.confirm(`找到 ${count} 条未关联已安装技能的说明。它们通常源自已被卸载的技能。确定要永久清理这 ${count} 条记录吗？`)) {
        const deleted = await clearUnassociatedDescriptions();
        alert(`清理完毕，已删除 ${deleted} 条未关联的说明。`);
        queryClient.invalidateQueries({ queryKey: ['skills'] });
      }
    } catch (err) {
      alert(`清理失败: ${err instanceof Error ? err.message : String(err)}`);
    }
  };
  ```
  Add these buttons to the toolbar in `SkillsPage` next to the `导入技能` button:
  ```tsx
            <div className="skills-toolbar__actions">
              <button className="button button--secondary" onClick={() => refetchUpdates()} disabled={updatesLoading}>
                <RefreshCw size={16} className={updatesLoading ? 'is-spinning' : ''} />
                检查更新
              </button>
              
              <button className="button button--secondary" onClick={handleExportDescriptions} title="备份全局自定义技能说明">
                导出说明
              </button>
              <button className="button button--secondary" onClick={handleImportDescriptionsClick} title="恢复或导入技能说明">
                导入说明
              </button>
              <button className="button button--secondary" onClick={handleCleanUnassociated} title="清理已被卸载的技能对应的说明">
                清理未关联
              </button>

              <button className="button button--primary" onClick={() => setIsImportOpen(true)}>
                <Plus size={16} /> 导入技能
              </button>
            </div>
  ```
  Render the `ImportDescriptionsModal` if `importPreview` state is populated:
  ```tsx
      {importPreview && (
        <ImportDescriptionsModal
          preview={importPreview}
          onClose={() => setImportPreview(null)}
          onSuccess={() => {
            queryClient.invalidateQueries({ queryKey: ['skills'] });
            alert('技能说明导入成功！');
          }}
        />
      )}
  ```

---

### Task 9: Frontend Tests & Verification

**Files:**
- Modify: `src/features/skills/components/SkillCard.test.tsx` (verify custom description fallback)
- Modify: `src/features/skills/components/SkillDetailModal.test.tsx` (verify edit flow)

- [ ] **Step 1: Update `SkillCard.test.tsx` to verify fallback rules**
  Add a test to verify custom descriptions:
  ```typescript
    it('renders custom description and custom badge if provided', () => {
      const skillWithCustom = {
        ...pack,
        custom_description: '这是自定义的技能包说明'
      };
      
      render(
        <SkillCard
          skill={skillWithCustom}
          categoryName="设计"
          onOpenDetail={vi.fn()}
        />
      );

      expect(screen.getByText('自定义')).toBeInTheDocument();
      expect(screen.getByText('这是自定义的技能包说明')).toBeInTheDocument();
      expect(screen.queryByText('Design skills')).not.toBeInTheDocument();
    });
  ```

- [ ] **Step 2: Run frontend tests**
  Command: `npm run test:run`
  Expected: PASS

---

## Verification Plan

### Automated Tests
- Rust database unit tests: `cargo test --manifest-path src-tauri/Cargo.toml`
- React component tests: `npm run test:run`
- Rust warning checks: `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings`
- TypeScript lint checks: `npm run lint`

### Manual Verification
1. Start application: `npm run tauri:dev`
2. Open details for any skill. Write a custom description. Verify character count limits. Save and verify the "自定义" badge and custom description display in:
   - Skill catalog page
   - Sub-skill details view
   - Project overview/skills enablement page
3. Verify search: Search for keywords in the custom description; ensure the skill is returned. Search for keywords in the original description; ensure the skill is still returned.
4. Open a sub-skill, edit its description, switch back to the parent without saving. Verify warning prompt is shown.
5. Export descriptions to a file. Uninstall a skill (verify its description is unassociated but still kept). Confirm "清理未关联" shows `1` unassociated record and prompts for confirmation. Run cleanup.
6. Re-import the exported descriptions file. Verify preview details, conflict strategies selection, and correct import recovery.
