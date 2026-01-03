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

CREATE TABLE messages (
  id BINARY(16) NOT NULL PRIMARY KEY, -- UUID
  user_id BINARY(16) NOT NULL, -- UUID
  channel_id BINARY(16) NOT NULL, -- UUID
  content TEXT NOT NULL,
  created_at DATETIME(6) NOT NULL,
  updated_at DATETIME(6) NOT NULL
);
