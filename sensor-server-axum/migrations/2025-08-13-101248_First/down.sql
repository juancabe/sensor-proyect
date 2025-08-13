-- Drop tables in reverse order of creation to handle foreign key dependencies
DROP TABLE IF EXISTS sensor_data;
DROP TABLE IF EXISTS user_sensors;
DROP TABLE IF EXISTS user_places;
DROP TABLE IF EXISTS users;
DROP TABLE IF EXISTS colors;

-- Drop the function used by triggers
DROP FUNCTION IF EXISTS update_updated_at_column();

