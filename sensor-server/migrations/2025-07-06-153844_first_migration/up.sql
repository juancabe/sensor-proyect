-- Your SQL goes here
CREATE TABLE users (
    username TEXT NOT NULL PRIMARY KEY,
    uuid TEXT NOT NULL,
    hashed_password TEXT NOT NULL,
    email TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    UNIQUE (uuid),
    UNIQUE (email)
) STRICT;

CREATE TABLE user_places (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    user TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    FOREIGN KEY (user) REFERENCES users(username) ON DELETE CASCADE ON UPDATE CASCADE
) STRICT;

CREATE TABLE sensor_kinds (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    description TEXT
) STRICT;

CREATE TABLE user_sensors (
    api_id TEXT NOT NULL PRIMARY KEY,
    place INTEGER NOT NULL,
    kind INTEGER NOT NULL,
    last_measurement INTEGER NOT NULL,
    ble_mac_address TEXT NOT NULL,
    FOREIGN KEY (place) REFERENCES user_places(id) ON DELETE CASCADE ON UPDATE CASCADE,
    FOREIGN KEY (kind) REFERENCES sensor_kinds(id) ON DELETE CASCADE ON UPDATE CASCADE
) STRICT;

CREATE TABLE aht10data (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    sensor TEXT NOT NULL,
    serialized_data TEXT NOT NULL,
    added_at INTEGER NOT NULL,
    FOREIGN KEY (sensor) REFERENCES user_sensors(api_id) ON DELETE CASCADE ON UPDATE CASCADE
) STRICT;

CREATE TABLE scd4xdata (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    sensor TEXT NOT NULL,
    serialized_data TEXT NOT NULL,
    added_at INTEGER NOT NULL,
    FOREIGN KEY (sensor) REFERENCES user_sensors(api_id) ON DELETE CASCADE ON UPDATE CASCADE
) STRICT;

INSERT INTO sensor_kinds (name, description) VALUES 
('aht10', 'Temperature and humidity sensor'),
('scd4x', 'CO2, temperature, and humidity sensor');


INSERT INTO users (username, uuid, hashed_password, email, created_at, updated_at)
VALUES (
    'testuser',
    '693a3bd4-5f16-4de8-9fb1-31b26edf6ba9',
    'hashedpassword123',
    'testuser@example.com',
    strftime('%s','now'),
    strftime('%s','now')
);

INSERT INTO user_places (user, name, description, created_at, updated_at)
VALUES
('testuser', 'Home', 'Primary residence', strftime('%s','now'), strftime('%s','now')),
('testuser', 'Office', 'Workplace location', strftime('%s','now'), strftime('%s','now'));

INSERT INTO user_sensors (api_id, place, kind, last_measurement, ble_mac_address)
VALUES
('sensor-aht10-home', 1, 1, strftime('%s','now'), 'AA:BB:CC:DD:EE:01'),
('sensor-scd4x-office', 2, 2, strftime('%s','now'), 'AA:BB:CC:DD:EE:02');

INSERT INTO aht10data (sensor, serialized_data, added_at) VALUES
('sensor-aht10-home', '{"temperature":22.5,"humidity":45.2,"sensor_id":"1290u8e3"}', strftime('%s','now')),
('sensor-aht10-home', '{"temperature":22.7,"humidity":44.9,"sensor_id":"1290u8e3"}', strftime('%s','now')),
('sensor-aht10-home', '{"temperature":22.6,"humidity":45.1,"sensor_id":"1290u8e3"}', strftime('%s','now')),
('sensor-aht10-home', '{"temperature":22.8,"humidity":44.8,"sensor_id":"1290u8e3"}', strftime('%s','now')),
('sensor-aht10-home', '{"temperature":22.9,"humidity":44.7,"sensor_id":"1290u8e3"}', strftime('%s','now')),
('sensor-aht10-home', '{"temperature":23.0,"humidity":44.5,"sensor_id":"1290u8e3"}', strftime('%s','now')),
('sensor-aht10-home', '{"temperature":23.1,"humidity":44.3,"sensor_id":"1290u8e3"}', strftime('%s','now')),
('sensor-aht10-home', '{"temperature":23.2,"humidity":44.2,"sensor_id":"1290u8e3"}', strftime('%s','now')),
('sensor-aht10-home', '{"temperature":23.3,"humidity":44.0,"sensor_id":"1290u8e3"}', strftime('%s','now')),
('sensor-aht10-home', '{"temperature":23.4,"humidity":43.8,"sensor_id":"1290u8e3"}', strftime('%s','now'));

INSERT INTO scd4xdata (sensor, serialized_data, added_at) VALUES
('sensor-scd4x-office', '{"co2":420,"temperature":21.5,"humidity":40.2, "sensor_id": "12edq213"}', strftime('%s','now')),
('sensor-scd4x-office', '{"co2":425,"temperature":21.6,"humidity":40.1, "sensor_id": "12edq213"}', strftime('%s','now')),
('sensor-scd4x-office', '{"co2":430,"temperature":21.7,"humidity":40.0, "sensor_id": "12edq213"}', strftime('%s','now')),
('sensor-scd4x-office', '{"co2":435,"temperature":21.8,"humidity":39.9, "sensor_id": "12edq213"}', strftime('%s','now')),
('sensor-scd4x-office', '{"co2":440,"temperature":21.9,"humidity":39.8, "sensor_id": "12edq213"}', strftime('%s','now')),
('sensor-scd4x-office', '{"co2":445,"temperature":22.0,"humidity":39.7, "sensor_id": "12edq213"}', strftime('%s','now')),
('sensor-scd4x-office', '{"co2":450,"temperature":22.1,"humidity":39.6, "sensor_id": "12edq213"}', strftime('%s','now')),
('sensor-scd4x-office', '{"co2":455,"temperature":22.2,"humidity":39.5, "sensor_id": "12edq213"}', strftime('%s','now')),
('sensor-scd4x-office', '{"co2":460,"temperature":22.3,"humidity":39.4, "sensor_id": "12edq213"}', strftime('%s','now')),
('sensor-scd4x-office', '{"co2":465,"temperature":22.4,"humidity":39.3, "sensor_id": "12edq213"}', strftime('%s','now'));