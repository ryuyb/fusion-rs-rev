use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::AppResult;
use crate::jobs::types::{JobContext, JobTask};
use crate::repositories::JobExecutionRepository;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataCleanupTask {
    #[serde(default = "default_retention_days")]
    pub retention_days: i64,
}

fn default_retention_days() -> i64 {
    30
}

#[async_trait]
impl JobTask for DataCleanupTask {
    fn task_type() -> &'static str
    where
        Self: Sized,
    {
        "data_cleanup"
    }

    async fn execute(&self, ctx: JobContext) -> AppResult<()> {
        let execution_repo = JobExecutionRepository::new(ctx.db_pool);
        let deleted = execution_repo
            .cleanup_old_executions(self.retention_days)
            .await?;

        tracing::info!(
            deleted_count = deleted,
            retention_days = self.retention_days,
            "Data cleanup completed"
        );

        Ok(())
    }

    fn description(&self) -> Option<String> {
        Some(format!(
            "Clean up job execution history older than {} days",
            self.retention_days
        ))
    }
}
