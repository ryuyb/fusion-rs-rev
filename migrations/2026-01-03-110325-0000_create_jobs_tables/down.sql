-- This file should undo anything in `up.sql`
DROP TABLE IF EXISTS job_executions CASCADE;
DROP TABLE IF EXISTS scheduled_jobs CASCADE;
DROP TYPE IF EXISTS job_status CASCADE;
