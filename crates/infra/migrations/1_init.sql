CREATE TABLE users (
  id BINARY(16) NOT NULL PRIMARY KEY, -- UUID
  handle VARCHAR(32) NOT NULL,
  display_name VARCHAR(32) NOT NULL
);

CREATE TABLE user_tokens (
  user_id BINARY(16) NOT NULL PRIMARY KEY, -- UUID
  -- traQ's access tokens are strings of 36 characters
  -- https://github.com/traPtitech/traQ/blob/a69fd6a42f54907a9d78a6810b41f5d578a0c0a3/docs/dbSchema/oauth2_tokens.md
  access_token VARCHAR(36) NOT NULL,

  CONSTRAINT fk_user_tokens_user FOREIGN KEY (user_id)
    REFERENCES users(id) ON DELETE CASCADE
);

-- Strict foreign key constraints are not required as this is a cache table.
-- We regard traQ's API as the source of truth.
CREATE TABLE messages (
  id BINARY(16) NOT NULL PRIMARY KEY, -- UUID
  user_id BINARY(16) NOT NULL, -- UUID
  channel_id BINARY(16) NOT NULL, -- UUID
  content TEXT NOT NULL,
  created_at TIMESTAMP(6) NOT NULL,
  updated_at TIMESTAMP(6) NOT NULL,
  last_crawled_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6),

  INDEX idx_created_at (created_at DESC)
);

CREATE TABLE read_messages (
  user_id BINARY(16) NOT NULL, -- UUID
  message_id BINARY(16) NOT NULL, -- UUID
  read_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6),

  PRIMARY KEY (user_id, message_id),
  INDEX idx_read_at (read_at),
  CONSTRAINT fk_read_messages_user FOREIGN KEY (user_id)
    REFERENCES users(id) ON DELETE CASCADE,
  CONSTRAINT fk_read_messages_message FOREIGN KEY (message_id)
    REFERENCES messages(id) ON DELETE CASCADE
);

CREATE TABLE reactions (
  message_id BINARY(16) NOT NULL, -- UUID
  stamp_id BINARY(16) NOT NULL, -- UUID
  user_id BINARY(16) NOT NULL, -- UUID
  stamp_count INT NOT NULL,

  PRIMARY KEY (message_id, stamp_id, user_id),
  CONSTRAINT fk_reactions_message FOREIGN KEY (message_id)
    REFERENCES messages(id) ON DELETE CASCADE
);

CREATE TABLE stamps (
  id BINARY(16) NOT NULL PRIMARY KEY, -- UUID
  name VARCHAR(32) NOT NULL
);
