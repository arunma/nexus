-- Add up migration script here
CREATE TABLE
    token_details
(
    token_uuid UUID          NOT NULL PRIMARY KEY,
    user_id    UUID          NOT NULL,
    token      VARCHAR(1000) NOT NULL,
    expires_in BIGINT        NOT NULL,
    max_age    BIGINT        NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX token_details_idx ON token_details (token_uuid);