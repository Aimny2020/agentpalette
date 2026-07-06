use std::path::Path;

use crate::domain::agent::{AgentMetadata, LaunchSpec};
use crate::domain::error::DomainResult;
use crate::domain::task::{ProcessStatus, TaskRunId};

pub trait AgentAdapter: Send + Sync {
    fn metadata(&self) -> DomainResult<AgentMetadata>;
    fn build_launch(&self, project: &Path) -> DomainResult<LaunchSpec>;
}

pub trait ProcessManager: Send + Sync {
    fn spawn(&self, launch: &LaunchSpec) -> DomainResult<TaskRunId>;
    fn status(&self, id: &TaskRunId) -> DomainResult<ProcessStatus>;
    fn terminate(&self, id: &TaskRunId) -> DomainResult<()>;
}

pub trait SkillRepository: Send + Sync {
    fn get_projects(&self) -> DomainResult<Vec<crate::domain::project::Project>>;
    fn get_project_path(&self, id: &str) -> DomainResult<Option<String>>;
    fn create_project(&self, project: &crate::domain::project::Project) -> DomainResult<()>;

    fn get_user_meta(&self, skill_id: &str) -> DomainResult<Option<crate::domain::skill::UserSkillMeta>>;
    fn save_user_meta(&self, skill_id: &str, category_id: Option<&str>, user_notes: Option<&str>) -> DomainResult<()>;
    fn delete_user_meta(&self, skill_id: &str) -> DomainResult<()>;
    
    fn get_project_skills(&self, project_id: &str) -> DomainResult<Vec<String>>;
    fn save_project_skill(&self, project_id: &str, skill_id: &str, enabled: bool) -> DomainResult<()>;

    fn get_categories(&self) -> DomainResult<Vec<crate::domain::skill::Category>>;
    fn create_category(&self, id: &str, name: &str, created_at: &str) -> DomainResult<crate::domain::skill::Category>;
    fn rename_category(&self, id: &str, name: &str) -> DomainResult<()>;
    fn delete_category(&self, id: &str) -> DomainResult<()>;
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::domain::agent::{AgentKind, AgentMetadata, LaunchSpec};
    use crate::domain::error::DomainResult;
    use crate::domain::task::{ProcessStatus, TaskRunId};

    use super::{AgentAdapter, ProcessManager};

    struct FakeAgent;

    impl AgentAdapter for FakeAgent {
        fn metadata(&self) -> DomainResult<AgentMetadata> {
            Ok(AgentMetadata {
                kind: AgentKind::Custom,
                display_name: "Fake Agent".into(),
                installed: true,
                version: Some("1.0.0".into()),
            })
        }

        fn build_launch(&self, project: &Path) -> DomainResult<LaunchSpec> {
            Ok(LaunchSpec {
                program: "fake-agent".into(),
                args: vec!["start".into()],
                cwd: project.to_path_buf(),
                env: vec![],
            })
        }
    }

    #[test]
    fn agent_adapter_describes_and_builds_launch_command() {
        let adapter = FakeAgent;
        let metadata = adapter.metadata().unwrap();
        let launch = adapter.build_launch(Path::new("/tmp/project")).unwrap();

        assert_eq!(metadata.display_name, "Fake Agent");
        assert_eq!(launch.program, "fake-agent");
        assert_eq!(launch.cwd, Path::new("/tmp/project"));
    }

    struct FakeProcessManager;

    impl ProcessManager for FakeProcessManager {
        fn spawn(&self, _launch: &LaunchSpec) -> DomainResult<TaskRunId> {
            Ok(TaskRunId::new("run-1"))
        }

        fn status(&self, _id: &TaskRunId) -> DomainResult<ProcessStatus> {
            Ok(ProcessStatus::Running)
        }

        fn terminate(&self, _id: &TaskRunId) -> DomainResult<()> {
            Ok(())
        }
    }

    #[test]
    fn process_manager_owns_the_task_lifecycle_contract() {
        let manager = FakeProcessManager;
        let launch = FakeAgent.build_launch(Path::new("/tmp/project")).unwrap();
        let id = manager.spawn(&launch).unwrap();

        assert_eq!(id.as_str(), "run-1");
        assert_eq!(manager.status(&id).unwrap(), ProcessStatus::Running);
        manager.terminate(&id).unwrap();
    }
}
