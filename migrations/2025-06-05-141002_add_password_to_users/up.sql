-- Your SQL goes here
ALTER TABLE users
ADD COLUMN password_hash TEXT;

ALTER TABLE users
ALTER COLUMN auth_type_or_provider DROP NOT NULL;

ALTER TABLE users
ADD CONSTRAINT users_auth_or_password_check
CHECK (
    password_hash IS NOT NULL OR auth_type_or_provider IS NOT NULL
);
