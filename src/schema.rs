// @generated automatically by Diesel CLI.

diesel::table! {
    notification_channels (id) {
        id -> Int4,
        user_id -> Int4,
        channel_type -> Text,
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
    notification_logs (id) {
        id -> Int8,
        channel_id -> Int4,
        message -> Text,
        status -> Text,
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
