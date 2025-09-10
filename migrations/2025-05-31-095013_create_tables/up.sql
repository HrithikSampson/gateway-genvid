-- Your SQL goes here
-- USER Table
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    auth_type_or_provider VARCHAR(50) NOT NULL,
    refresh_token TEXT NOT NULL,
    credit INTEGER NOT NULL DEFAULT 0,
    name VARCHAR(255) NOT NULL,
    stripe_customer_id TEXT
);

CREATE TYPE payment_enum AS ENUM ('pending', 'finished');
CREATE TABLE payment_history(
    id SERIAL PRIMARY KEY,
    user_id INTEGER REFERENCES users(id),
    stripe_subscription_id VARCHAR,
    status payment_enum,
    created_at TIMESTAMP DEFAULT NOW()
);
-- ENUM for progress
CREATE TYPE progress_enum AS ENUM ('pending', 'finished');
CREATE TYPE job_type AS ENUM('character_forming', 'video_composition', 'finished');
-- Character Formation : Gateway -> Background Removal Service & Voice Clone -> Character Formed
-- Creating Script: Gateway -> Video Composition with script -> Video  formed
-- Script Table -> Video Composition
CREATE TABLE script (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL,
    progress progress_enum,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);
-- Job Table -> Gateway
CREATE TABLE jobs (
    id SERIAL PRIMARY KEY,
    type_of_job job_type,
    user_id INTEGER NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);


-- Characters Table -> Gateway
CREATE TABLE characters (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL,
    voice_storage_id TEXT,
    image_storage_id TEXT,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Dialogue Table -> Video Composition
CREATE TABLE dialogue (
    id SERIAL PRIMARY KEY,
    script_id INTEGER NOT NULL,
    character_id INTEGER NOT NULL,
    speech TEXT NOT NULL,
    FOREIGN KEY (character_id) REFERENCES characters(id) ON DELETE CASCADE,
    FOREIGN KEY (script_id) REFERENCES script(id) ON DELETE CASCADE
);
