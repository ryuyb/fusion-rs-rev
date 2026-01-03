-- Drop tables in reverse order (respecting foreign keys)
DROP TABLE IF EXISTS notification_logs;
DROP TABLE IF EXISTS notification_channels;

-- Drop ENUM types
DROP TYPE IF EXISTS notification_status;
DROP TYPE IF EXISTS channel_type;
