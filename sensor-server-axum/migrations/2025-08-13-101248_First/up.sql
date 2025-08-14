CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
   NEW.updated_at = NOW();
   RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TABLE colors (
    id SERIAL PRIMARY KEY,
    hex_value TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL UNIQUE
);

INSERT INTO colors (hex_value, name) VALUES
('#FF0000', 'Red'),
('#0000FF', 'Blue'),
('#FFFF00', 'Yellow'),
('#008000', 'Green'), 
('#FFA500', 'Orange'),
('#800080', 'Purple'),
('#FFFFFF', 'White'),
('#000000', 'Black'),
('#808080', 'Gray');


CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    api_id TEXT NOT NULL UNIQUE,
    hashed_password TEXT NOT NULL,
    email TEXT NOT NULL UNIQUE,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TRIGGER update_users_updated_at
BEFORE UPDATE ON users
FOR EACH ROW
EXECUTE PROCEDURE update_updated_at_column();

CREATE TABLE user_places (
    id SERIAL PRIMARY KEY,
    api_id TEXT NOT NULL UNIQUE,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT,
    color_id INTEGER NOT NULL REFERENCES colors(id) ON DELETE RESTRICT,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TRIGGER update_user_places_updated_at
BEFORE UPDATE ON user_places
FOR EACH ROW
EXECUTE PROCEDURE update_updated_at_column();


CREATE TABLE user_sensors (
    id SERIAL PRIMARY KEY,
    api_id TEXT NOT NULL UNIQUE,
    place_id INTEGER NOT NULL REFERENCES user_places(id) ON DELETE CASCADE,
    device_id TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    color_id INTEGER NOT NULL REFERENCES colors(id) ON DELETE RESTRICT,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TRIGGER update_user_sensors_updated_at
BEFORE UPDATE ON user_sensors
FOR EACH ROW
EXECUTE PROCEDURE update_updated_at_column();

CREATE TABLE sensor_data (
    id BIGSERIAL PRIMARY KEY,
    sensor_id INTEGER NOT NULL REFERENCES user_sensors(id) ON DELETE CASCADE,
    data JSONB NOT NULL,
    added_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_sensor_data_latest ON sensor_data (sensor_id, added_at DESC);
