pub mod application;
pub mod commands;
pub mod domain;
pub mod infrastructure;

use std::sync::Arc;

use application::health_service::HealthService;
use commands::health::{health_check, AppState};
use commands::projects::*;
use commands::skills::*;
use commands::harnesses::*;
use infrastructure::database::SqliteDatabase;
use infrastructure::system::PlatformSystem;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .map_err(|_| domain::error::DomainError::AppDataDirectory)?;
            std::fs::create_dir_all(&app_data_dir)
                .map_err(|_| domain::error::DomainError::AppDataDirectory)?;

            let database = Arc::new(SqliteDatabase::open(&app_data_dir.join("agentforge.db"))?);
            let system = Arc::new(PlatformSystem::current());
            let skills = application::skill_service::SkillService::new(
                Arc::clone(&database) as Arc<dyn crate::domain::ports::SkillRepository>
            );
            let repo = Arc::clone(&database) as Arc<dyn crate::domain::ports::SkillRepository>;
            let harnesses = application::harness_service::HarnessService::new(
                Arc::clone(&database) as Arc<dyn crate::domain::ports::HarnessRepository>,
                Arc::clone(&database) as Arc<dyn crate::domain::ports::SkillRepository>,
            );
            app.manage(AppState {
                health: HealthService::new(database, system),
                skills,
                repo,
                harnesses,
            });
            Ok(())
        })
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
            save_custom_description,
            export_custom_descriptions,
            preview_custom_descriptions_import,
            confirm_custom_descriptions_import,
            get_unassociated_descriptions_count,
            clear_unassociated_descriptions,
            get_harness_templates,
            inspect_harness_import,
            import_harness_from_folder,
            extract_harness_from_project,
            create_harness_template,
            get_harness_template,
            read_harness_file,
            write_harness_file,
            create_harness_file,
            delete_harness_file,
            delete_harness_template,
            validate_harness_template,
            duplicate_harness_template
        ])
        .run(tauri::generate_context!())
        .expect("failed to run AgentForge");
}
