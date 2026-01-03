use std::collections::HashMap;
use serde::de::DeserializeOwned;
use serde_json::Value as JsonValue;

use crate::error::{AppError, AppResult};
use crate::jobs::types::JobTask;

type TaskFactory = Box<dyn Fn(JsonValue) -> AppResult<Box<dyn JobTask>> + Send + Sync>;

/// Registry for mapping job types to task implementations
pub struct JobRegistry {
    factories: HashMap<String, TaskFactory>,
}

impl JobRegistry {
    pub fn new() -> Self {
        Self {
            factories: HashMap::new(),
        }
    }

    /// Register a task type with the registry
    pub fn register<T>(&mut self) -> &mut Self
    where
        T: JobTask + DeserializeOwned + 'static,
    {
        let factory: TaskFactory = Box::new(|payload: JsonValue| {
            let task: T = serde_json::from_value(payload).map_err(|e| AppError::Internal {
                source: anyhow::Error::from(e),
            })?;
            Ok(Box::new(task) as Box<dyn JobTask>)
        });

        self.factories.insert(T::task_type().to_string(), factory);
        self
    }

    /// Create a task instance from job type and payload
    pub fn create_task(&self, job_type: &str, payload: JsonValue) -> AppResult<Box<dyn JobTask>> {
        let factory = self
            .factories
            .get(job_type)
            .ok_or_else(|| AppError::NotFound {
                entity: "JobType".to_string(),
                field: "type".to_string(),
                value: job_type.to_string(),
            })?;

        factory(payload)
    }
}
