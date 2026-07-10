use std::sync::Arc;
use tauri::State;

use crate::application::health_service::HealthService;
use crate::application::skill_service::SkillService;
use crate::commands::CommandError;
use crate::domain::health::HealthReport;
use crate::domain::ports::SkillRepository;

pub struct AppState {
    pub health: HealthService,
    pub skills: SkillService,
    pub repo: Arc<dyn SkillRepository>,
    pub harnesses: crate::application::harness_service::HarnessService,
}

#[tauri::command]
pub fn health_check(state: State<'_, AppState>) -> Result<HealthReport, CommandError> {
    state.health.check().map_err(CommandError::from)
}
