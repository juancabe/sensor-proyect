CREATE TABLE users (
    uuid TEXT NOT NULL PRIMARY KEY,
    username TEXT NOT NULL,
    hashed_password TEXT NOT NULL,
    email TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    UNIQUE (username),
    UNIQUE (email),
    UNIQUE (uuid)
) STRICT;

CREATE TABLE user_places (
    user_place_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    user_id TEXT NOT NULL,
    place_name TEXT NOT NULL,
    place_description TEXT,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(uuid) ON DELETE CASCADE ON UPDATE CASCADE
) STRICT;

CREATE TABLE aht10data (
  data_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
  user_uuid TEXT NOT NULL,
  user_place_id INTEGER NOT NULL,
  serialized_data TEXT NOT NULL,
  added_at INTEGER NOT NULL,
  FOREIGN KEY (user_uuid) REFERENCES users(uuid) ON DELETE CASCADE ON UPDATE CASCADE,
  FOREIGN KEY (user_place_id) REFERENCES user_places(user_place_id) ON DELETE CASCADE ON UPDATE CASCADE
) STRICT;