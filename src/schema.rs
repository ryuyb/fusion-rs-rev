// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "channel_type"))]
    pub struct ChannelType;

    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "job_status"))]
    pub struct JobStatus;

    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "notification_status"))]
    pub struct NotificationStatus;
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::JobStatus;

    job_executions (id) {
        id -> Int8,
        job_id -> Int4,
        #[max_length = 255]
        job_name -> Varchar,
        execution_id -> Uuid,
        started_at -> Timestamp,
        completed_at -> Nullable<Timestamp>,
        duration_ms -> Nullable<Int8>,
        status -> JobStatus,
        retry_attempt -> Int4,
        error_message -> Nullable<Text>,
        error_details -> Nullable<Jsonb>,
        result -> Nullable<Jsonb>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::ChannelType;

    notification_channels (id) {
        id -> Int4,
        user_id -> Int4,
        channel_type -> ChannelType,
        #[max_length = 255]
        name -> Varchar,
        config -> Jsonb,
        enabled -> Bool,
        priority -> Int4,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::NotificationStatus;

    notification_logs (id) {
        id -> Int8,
        channel_id -> Int4,
        message -> Text,
        status -> NotificationStatus,
        error_message -> Nullable<Text>,
        retry_count -> Int4,
        sent_at -> Timestamp,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::JobStatus;

    scheduled_jobs (id) {
        id -> Int4,
        #[max_length = 255]
        job_name -> Varchar,
        #[max_length = 100]
        job_type -> Varchar,
        #[max_length = 255]
        cron_expression -> Varchar,
        enabled -> Bool,
        allow_concurrent -> Bool,
        max_concurrent -> Nullable<Int4>,
        max_retries -> Int4,
        retry_delay_seconds -> Int4,
        retry_backoff_multiplier -> Numeric,
        timeout_seconds -> Int4,
        payload -> Nullable<Jsonb>,
        description -> Nullable<Text>,
        last_run_at -> Nullable<Timestamp>,
        last_run_status -> Nullable<JobStatus>,
        next_run_at -> Nullable<Timestamp>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        #[max_length = 255]
        created_by -> Nullable<Varchar>,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        #[max_length = 255]
        username -> Varchar,
        #[max_length = 255]
        email -> Varchar,
        #[max_length = 255]
        password -> Varchar,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::joinable!(job_executions -> scheduled_jobs (job_id));
diesel::joinable!(notification_channels -> users (user_id));
diesel::joinable!(notification_logs -> notification_channels (channel_id));

diesel::allow_tables_to_appear_in_same_query!(
    job_executions,
    notification_channels,
    notification_logs,
    scheduled_jobs,
    users,
);
