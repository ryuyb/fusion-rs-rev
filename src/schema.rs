// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "channel_type"))]
    pub struct ChannelType;

    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "notification_status"))]
    pub struct NotificationStatus;
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

diesel::joinable!(notification_channels -> users (user_id));
diesel::joinable!(notification_logs -> notification_channels (channel_id));

diesel::allow_tables_to_appear_in_same_query!(notification_channels, notification_logs, users,);
