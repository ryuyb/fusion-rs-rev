-- Drop trigger created by diesel_manage_updated_at
DROP TRIGGER IF EXISTS set_updated_at ON users;

-- Drop the users table
DROP TABLE IF EXISTS users;
