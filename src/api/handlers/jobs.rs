//! Job scheduling request handlers.

use crate::api::doc::JOB_TAG;
use crate::api::dto::{
    CreateJobRequest, JobExecutionResponse, JobResponse, PaginationParams, UpdateJobRequest,
};
use crate::error::AppResult;
use crate::state::AppState;
use crate::utils::validate::{ValidatedJson, ValidatedQuery};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

/// Creates job-related routes.
pub fn job_routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(list_jobs))
        .routes(routes!(create_job))
        .routes(routes!(get_job))
        .routes(routes!(update_job))
        .routes(routes!(delete_job))
        .routes(routes!(pause_job))
        .routes(routes!(resume_job))
        .routes(routes!(list_job_executions))
}

/// GET /api/jobs - List all scheduled jobs
#[utoipa::path(
    get,
    path = "/",
    tag = JOB_TAG,
    params(PaginationParams),
    responses(
        (status = 200, description = "List jobs by page", body = Vec<JobResponse>)
    ),
    security(
        ("bearerAuth" = [])
    )
)]
async fn list_jobs(
    State(state): State<AppState>,
    ValidatedQuery(_params): ValidatedQuery<PaginationParams>,
) -> AppResult<Json<Vec<JobResponse>>> {
    let jobs = state.services.jobs.get_enabled_jobs().await?;
    let responses: Vec<JobResponse> = jobs.into_iter().map(JobResponse::from).collect();
    Ok(Json(responses))
}

/// POST /api/jobs - Create a new scheduled job
#[utoipa::path(
    post,
    path = "/",
    tag = JOB_TAG,
    request_body = CreateJobRequest,
    responses(
        (status = 201, description = "Job created successfully", body = JobResponse),
        (status = 400, description = "Invalid request")
    ),
    security(
        ("bearerAuth" = [])
    )
)]
async fn create_job(
    State(state): State<AppState>,
    ValidatedJson(req): ValidatedJson<CreateJobRequest>,
) -> AppResult<(StatusCode, Json<JobResponse>)> {
    let new_job = req.into_new_job();
    let job = state.services.jobs.create_job(new_job).await?;

    if let Some(scheduler) = &state.scheduler {
        scheduler.reload_jobs().await?;
    }

    Ok((StatusCode::CREATED, Json(JobResponse::from(job))))
}

/// GET /api/jobs/:id - Get job by ID
#[utoipa::path(
    get,
    path = "/{id}",
    tag = JOB_TAG,
    params(
        ("id" = i32, Path, description = "Job ID")
    ),
    responses(
        (status = 200, description = "Job found", body = JobResponse),
        (status = 404, description = "Job not found")
    ),
    security(
        ("bearerAuth" = [])
    )
)]
async fn get_job(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> AppResult<Json<JobResponse>> {
    let job = state.services.jobs.get_job(id).await?;
    Ok(Json(JobResponse::from(job)))
}

/// PUT /api/jobs/:id - Update job by ID
#[utoipa::path(
    put,
    path = "/{id}",
    tag = JOB_TAG,
    params(
        ("id" = i32, Path, description = "Job ID")
    ),
    request_body = UpdateJobRequest,
    responses(
        (status = 200, description = "Job updated successfully", body = JobResponse),
        (status = 404, description = "Job not found")
    ),
    security(
        ("bearerAuth" = [])
    )
)]
async fn update_job(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    ValidatedJson(req): ValidatedJson<UpdateJobRequest>,
) -> AppResult<Json<JobResponse>> {
    let update = req.into_update_job();
    let job = state.services.jobs.update_job(id, update).await?;

    if let Some(scheduler) = &state.scheduler {
        scheduler.reload_jobs().await?;
    }

    Ok(Json(JobResponse::from(job)))
}

/// DELETE /api/jobs/:id - Delete job by ID
#[utoipa::path(
    delete,
    path = "/{id}",
    tag = JOB_TAG,
    params(
        ("id" = i32, Path, description = "Job ID")
    ),
    responses(
        (status = 204, description = "Job deleted successfully"),
        (status = 404, description = "Job not found")
    ),
    security(
        ("bearerAuth" = [])
    )
)]
async fn delete_job(State(state): State<AppState>, Path(id): Path<i32>) -> AppResult<StatusCode> {
    state.services.jobs.delete_job(id).await?;

    if let Some(scheduler) = &state.scheduler {
        scheduler.reload_jobs().await?;
    }

    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/jobs/:id/pause - Pause (disable) a job
#[utoipa::path(
    post,
    path = "/{id}/pause",
    tag = JOB_TAG,
    params(
        ("id" = i32, Path, description = "Job ID")
    ),
    responses(
        (status = 200, description = "Job paused successfully", body = JobResponse),
        (status = 404, description = "Job not found")
    ),
    security(
        ("bearerAuth" = [])
    )
)]
async fn pause_job(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> AppResult<Json<JobResponse>> {
    let job = state.services.jobs.pause_job(id).await?;

    if let Some(scheduler) = &state.scheduler {
        scheduler.reload_jobs().await?;
    }

    Ok(Json(JobResponse::from(job)))
}

/// POST /api/jobs/:id/resume - Resume (enable) a job
#[utoipa::path(
    post,
    path = "/{id}/resume",
    tag = JOB_TAG,
    params(
        ("id" = i32, Path, description = "Job ID")
    ),
    responses(
        (status = 200, description = "Job resumed successfully", body = JobResponse),
        (status = 404, description = "Job not found")
    ),
    security(
        ("bearerAuth" = [])
    )
)]
async fn resume_job(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> AppResult<Json<JobResponse>> {
    let job = state.services.jobs.resume_job(id).await?;

    if let Some(scheduler) = &state.scheduler {
        scheduler.reload_jobs().await?;
    }

    Ok(Json(JobResponse::from(job)))
}

/// GET /api/jobs/:id/executions - List execution history for a job
#[utoipa::path(
    get,
    path = "/{id}/executions",
    tag = JOB_TAG,
    params(
        ("id" = i32, Path, description = "Job ID"),
        PaginationParams
    ),
    responses(
        (status = 200, description = "List job executions", body = Vec<JobExecutionResponse>)
    ),
    security(
        ("bearerAuth" = [])
    )
)]
async fn list_job_executions(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    ValidatedQuery(params): ValidatedQuery<PaginationParams>,
) -> AppResult<Json<Vec<JobExecutionResponse>>> {
    let params = params.normalize();
    let executions = state
        .services
        .jobs
        .list_job_executions(id, params.limit() as i64, params.offset() as i64)
        .await?;

    let responses: Vec<JobExecutionResponse> = executions
        .into_iter()
        .map(JobExecutionResponse::from)
        .collect();

    Ok(Json(responses))
}
