//! Scheduled job management
//!
//! ## Overview
//! - Configure scheduled jobs
//! - Query job history
//! - Manage job execution

use crate::error::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use typed_builder::TypedBuilder;

/// Scheduled job information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledJob {
    /// Unique identifier for the scheduled job
    pub job_id: String,
    /// Human-readable name for the job
    pub name: String,
    /// Type of job (backup, cleanup, rotation, etc.)
    pub job_type: String,
    /// Cron-style schedule expression for when the job runs
    pub schedule: String,
    /// Whether the scheduled job is currently enabled
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    /// Timestamp of the last job execution (ISO 8601 format)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_run: Option<String>,
    /// Timestamp of the next scheduled job execution (ISO 8601 format)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_run: Option<String>,
    /// Job-specific parameters and configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

/// Create scheduled job request
#[derive(Debug, Clone, Serialize, Deserialize, TypedBuilder)]
pub struct CreateScheduledJobRequest {
    /// Human-readable name for the new job
    #[builder(setter(into))]
    pub name: String,
    /// Type of job to create (backup, cleanup, rotation, etc.)
    #[builder(setter(into))]
    pub job_type: String,
    /// Cron-style schedule expression defining when the job should run
    #[builder(setter(into))]
    pub schedule: String,
    /// Whether the job should be enabled immediately upon creation
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub enabled: Option<bool>,
    /// Job-specific parameters and configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub params: Option<Value>,
}

/// Job execution history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobExecution {
    /// Unique identifier for this job execution instance
    pub execution_id: String,
    /// ID of the scheduled job that was executed
    pub job_id: String,
    /// Timestamp when the job execution started (ISO 8601 format)
    pub start_time: String,
    /// Timestamp when the job execution finished (ISO 8601 format)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<String>,
    /// Execution status (running, completed, failed, cancelled)
    pub status: String,
    /// Error description if the job execution failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

define_handler!(
    /// Job scheduler handler
    pub struct JobSchedulerHandler;
);

impl JobSchedulerHandler {
    /// Get the documented global job scheduler configuration.
    pub async fn get_config(&self) -> Result<Value> {
        self.client.get("/v1/job_scheduler").await
    }

    /// Update the documented global job scheduler configuration.
    pub async fn update_config(&self, body: Value) -> Result<Value> {
        self.client.put("/v1/job_scheduler", &body).await
    }

    /// Retired list helper whose response model did not match the global
    /// configuration resource.
    #[deprecated(note = "use get_config")]
    pub async fn list(&self) -> Result<Vec<ScheduledJob>> {
        crate::error::unsupported_operation("list scheduled jobs")
    }

    /// Retired scheduled-job lookup helper.
    #[deprecated(note = "Redis Software does not register per-job scheduler routes")]
    pub async fn get(&self, _job_id: &str) -> Result<ScheduledJob> {
        crate::error::unsupported_operation("get scheduled job")
    }

    /// Retired scheduled-job deletion helper.
    #[deprecated(note = "Redis Software does not register per-job scheduler routes")]
    pub async fn delete(&self, _job_id: &str) -> Result<()> {
        crate::error::unsupported_operation("delete scheduled job")
    }

    /// Retired scheduled-job creation helper.
    #[deprecated(note = "Redis Software does not register POST /v1/job_scheduler")]
    pub async fn create(&self, _request: CreateScheduledJobRequest) -> Result<ScheduledJob> {
        crate::error::unsupported_operation("create scheduled job")
    }

    /// Retired per-job update helper.
    #[deprecated(note = "Redis Software does not register per-job scheduler routes")]
    pub async fn update(
        &self,
        _job_id: &str,
        _request: CreateScheduledJobRequest,
    ) -> Result<ScheduledJob> {
        crate::error::unsupported_operation("update scheduled job")
    }

    /// Retired scheduled-job trigger helper.
    #[deprecated(note = "Redis Software does not register per-job scheduler routes")]
    pub async fn trigger(&self, _job_id: &str) -> Result<JobExecution> {
        crate::error::unsupported_operation("trigger scheduled job")
    }

    /// Retired scheduled-job history helper.
    #[deprecated(note = "Redis Software does not register per-job scheduler routes")]
    pub async fn history(&self, _job_id: &str) -> Result<Vec<JobExecution>> {
        crate::error::unsupported_operation("get scheduled-job history")
    }

    /// Retired global update alias whose response type was inaccurate.
    #[deprecated(note = "use update_config")]
    pub async fn update_all(&self, _body: Value) -> Result<Vec<ScheduledJob>> {
        crate::error::unsupported_operation("update scheduler with legacy response model")
    }
}
