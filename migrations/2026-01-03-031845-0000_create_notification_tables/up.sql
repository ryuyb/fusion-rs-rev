-- ============================================================================
-- Create PostgreSQL ENUM types
-- ============================================================================
CREATE TYPE channel_type AS ENUM ('webhook', 'email', 'sms', 'discord', 'slack', 'bark');
CREATE TYPE notification_status AS ENUM ('pending', 'sent', 'failed', 'retrying');

-- ============================================================================
-- Notification Channels Table
-- ============================================================================
-- Stores user notification channel configurations
CREATE TABLE notification_channels (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    channel_type channel_type NOT NULL,
    name VARCHAR(255) NOT NULL,
    config JSONB NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT true,
    priority INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

    -- Indexes for performance
    CONSTRAINT unique_user_channel_name UNIQUE (user_id, name)
);

CREATE INDEX idx_notification_channels_user_id ON notification_channels(user_id);
CREATE INDEX idx_notification_channels_enabled ON notification_channels(enabled) WHERE enabled = true;
CREATE INDEX idx_notification_channels_priority ON notification_channels(priority DESC);

-- Add updated_at trigger
SELECT diesel_manage_updated_at('notification_channels');

-- ============================================================================
-- Notification Logs Table
-- ============================================================================
-- Stores complete send history for auditing and debugging
CREATE TABLE notification_logs (
    id BIGSERIAL PRIMARY KEY,
    channel_id INTEGER NOT NULL REFERENCES notification_channels(id) ON DELETE CASCADE,
    message TEXT NOT NULL,
    status notification_status NOT NULL,
    error_message TEXT,
    retry_count INTEGER NOT NULL DEFAULT 0,
    sent_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

    -- Indexes for querying logs
    CONSTRAINT max_retry_count CHECK (retry_count >= 0 AND retry_count <= 10)
);

CREATE INDEX idx_notification_logs_channel_id ON notification_logs(channel_id);
CREATE INDEX idx_notification_logs_status ON notification_logs(status);
CREATE INDEX idx_notification_logs_sent_at ON notification_logs(sent_at DESC);
