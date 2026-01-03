-- Create enum types for job status
CREATE TYPE job_status AS ENUM ('pending', 'running', 'success', 'failed', 'timeout', 'cancelled');

-- Job definitions table
CREATE TABLE scheduled_jobs (
    id SERIAL PRIMARY KEY,
    job_name VARCHAR(255) NOT NULL UNIQUE,
    job_type VARCHAR(100) NOT NULL,
    cron_expression VARCHAR(255) NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT true,

    -- Concurrency control
    allow_concurrent BOOLEAN NOT NULL DEFAULT false,
    max_concurrent INTEGER CHECK (max_concurrent IS NULL OR max_concurrent > 0),

    -- Retry configuration
    max_retries INTEGER NOT NULL DEFAULT 3,
    retry_delay_seconds INTEGER NOT NULL DEFAULT 60,
    retry_backoff_multiplier NUMERIC(3,2) NOT NULL DEFAULT 2.0,

    -- Timeout
    timeout_seconds INTEGER NOT NULL DEFAULT 300,

    -- Payload and metadata
    payload JSONB,
    description TEXT,

    -- Tracking
    last_run_at TIMESTAMP,
    last_run_status job_status,
    next_run_at TIMESTAMP,

    -- Audit
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by VARCHAR(255)
);

-- Job execution history table
CREATE TABLE job_executions (
    id BIGSERIAL PRIMARY KEY,
    job_id INTEGER NOT NULL REFERENCES scheduled_jobs(id) ON DELETE CASCADE,
    job_name VARCHAR(255) NOT NULL,
    execution_id UUID NOT NULL DEFAULT gen_random_uuid(),

    -- Execution tracking
    started_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    completed_at TIMESTAMP,
    duration_ms BIGINT,

    -- Status and error handling
    status job_status NOT NULL,
    retry_attempt INTEGER NOT NULL DEFAULT 0,
    error_message TEXT,
    error_details JSONB,

    -- Result
    result JSONB
);

-- Indexes
CREATE INDEX idx_scheduled_jobs_enabled ON scheduled_jobs(enabled) WHERE enabled = true;
CREATE INDEX idx_scheduled_jobs_next_run ON scheduled_jobs(next_run_at) WHERE enabled = true;
CREATE INDEX idx_job_executions_job_id ON job_executions(job_id);
CREATE INDEX idx_job_executions_status ON job_executions(status);
CREATE INDEX idx_job_executions_started_at ON job_executions(started_at);

-- Trigger for updated_at
SELECT diesel_manage_updated_at('scheduled_jobs');
