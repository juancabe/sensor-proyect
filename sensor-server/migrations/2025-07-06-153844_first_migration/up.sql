CREATE TABLE users (
    username TEXT NOT NULL PRIMARY KEY,
    api_id TEXT NOT NULL UNIQUE, -- 20 bytes of data represented as HEX on a String
    hashed_password TEXT NOT NULL,
    email TEXT NOT NULL UNIQUE,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL
);

CREATE TABLE user_places (
    id SERIAL PRIMARY KEY,
    "user" TEXT NOT NULL REFERENCES users(username) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL
);

CREATE TABLE sensor_kinds (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL, -- SensorKind enum variant value as serialized String
    description TEXT
);

CREATE TABLE user_sensors (
    api_id TEXT NOT NULL PRIMARY KEY, -- 20 bytes of data represented as HEX on a String
    place INTEGER NOT NULL REFERENCES user_places(id) ON DELETE CASCADE,
    kind INTEGER NOT NULL REFERENCES sensor_kinds(id) ON DELETE CASCADE,
    last_measurement TIMESTAMP NOT NULL,
    device_id TEXT NOT NULL -- 20 bytes of data represented as HEX on a String
);

CREATE TABLE aht10data (
    id SERIAL PRIMARY KEY,
    sensor TEXT NOT NULL REFERENCES user_sensors(api_id) ON DELETE CASCADE,
    serialized_data TEXT NOT NULL,
    added_at TIMESTAMP NOT NULL
);

CREATE TABLE scd4xdata (
    id SERIAL PRIMARY KEY,
    sensor TEXT NOT NULL REFERENCES user_sensors(api_id) ON DELETE CASCADE,
    serialized_data TEXT NOT NULL,
    added_at TIMESTAMP NOT NULL
);

INSERT INTO sensor_kinds (name, description) VALUES 
('aht10', 'Temperature and humidity sensor'),
('scd4x', 'CO2, temperature, and humidity sensor');


INSERT INTO users (username, api_id, hashed_password, email, created_at, updated_at)
VALUES (
    'testuser',
    '94a990533d76aaaaaaaaaaaaaaaaaaaaaaaaaaaa',
    'ae5deb822e0d71992900471a7199d0d95b8e7c9d05c40a8245a281fd2c1d6684',
    'testuser@example.com',
    NOW(),
    NOW()
);

INSERT INTO user_places ("user", name, description, created_at, updated_at)
VALUES
('testuser', 'Home', 'Primary residence', NOW(), NOW()),
('testuser', 'Office', 'Workplace location', NOW(), NOW());

INSERT INTO user_sensors (api_id, place, kind, last_measurement, device_id)
VALUES
('94a990533d761111111111111111111111111111', 1, 1, NOW(), '94a990533d760000000000000000000000000000'),
('94a990533d762222222222222222222222222222', 2, 2, NOW(), '94a990533d770000000000000000000000000000'),
('abc36768cf4d927e267a72ac1cb8108693bdafd1', 1, 2, NOW(), '94a990533d760000000000000000000000000000');

INSERT INTO aht10data (sensor, serialized_data, added_at) VALUES
('94a990533d761111111111111111111111111111', '{"temperature":22.5,"humidity":45.2,"sensor_id":"94a990533d760000000000000000000000000000"}', NOW()),
('94a990533d761111111111111111111111111111', '{"temperature":22.7,"humidity":44.9,"sensor_id":"94a990533d760000000000000000000000000000"}', NOW()),
('94a990533d761111111111111111111111111111', '{"temperature":22.6,"humidity":45.1,"sensor_id":"94a990533d760000000000000000000000000000"}', NOW()),
('94a990533d761111111111111111111111111111', '{"temperature":22.8,"humidity":44.8,"sensor_id":"94a990533d760000000000000000000000000000"}', NOW()),
('94a990533d761111111111111111111111111111', '{"temperature":22.9,"humidity":44.7,"sensor_id":"94a990533d760000000000000000000000000000"}', NOW()),
('94a990533d761111111111111111111111111111', '{"temperature":23.0,"humidity":44.5,"sensor_id":"94a990533d760000000000000000000000000000"}', NOW()),
('94a990533d761111111111111111111111111111', '{"temperature":23.1,"humidity":44.3,"sensor_id":"94a990533d760000000000000000000000000000"}', NOW()),
('94a990533d761111111111111111111111111111', '{"temperature":23.2,"humidity":44.2,"sensor_id":"94a990533d760000000000000000000000000000"}', NOW()),
('94a990533d761111111111111111111111111111', '{"temperature":23.3,"humidity":44.0,"sensor_id":"94a990533d760000000000000000000000000000"}', NOW()),
('94a990533d761111111111111111111111111111', '{"temperature":23.4,"humidity":43.8,"sensor_id":"94a990533d760000000000000000000000000000"}', NOW());

INSERT INTO scd4xdata (sensor, serialized_data, added_at) VALUES
('94a990533d762222222222222222222222222222', '{"co2":420,"temperature":21.5,"humidity":40.2, "sensor_id": "94a990533d770000000000000000000000000000"}', NOW()),
('94a990533d762222222222222222222222222222', '{"co2":425,"temperature":21.6,"humidity":40.1, "sensor_id": "94a990533d770000000000000000000000000000"}', NOW()),
('94a990533d762222222222222222222222222222', '{"co2":430,"temperature":21.7,"humidity":40.0, "sensor_id": "94a990533d770000000000000000000000000000"}', NOW()),
('94a990533d762222222222222222222222222222', '{"co2":435,"temperature":21.8,"humidity":39.9, "sensor_id": "94a990533d770000000000000000000000000000"}', NOW()),
('94a990533d762222222222222222222222222222', '{"co2":440,"temperature":21.9,"humidity":39.8, "sensor_id": "94a990533d770000000000000000000000000000"}', NOW()),
('94a990533d762222222222222222222222222222', '{"co2":445,"temperature":22.0,"humidity":39.7, "sensor_id": "94a990533d770000000000000000000000000000"}', NOW()),
('94a990533d762222222222222222222222222222', '{"co2":450,"temperature":22.1,"humidity":39.6, "sensor_id": "94a990533d770000000000000000000000000000"}', NOW()),
('94a990533d762222222222222222222222222222', '{"co2":455,"temperature":22.2,"humidity":39.5, "sensor_id": "94a990533d770000000000000000000000000000"}', NOW()),
('94a990533d762222222222222222222222222222', '{"co2":460,"temperature":22.3,"humidity":39.4, "sensor_id": "94a990533d770000000000000000000000000000"}', NOW()),
('94a990533d762222222222222222222222222222', '{"co2":465,"temperature":22.4,"humidity":39.3, "sensor_id": "94a990533d770000000000000000000000000000"}', NOW());