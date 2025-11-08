-- Database migration for Notepad API v2.0
-- This script creates the new notes table with enhanced structure

-- Drop the existing notes table if it exists (for development)
-- WARNING: This will delete all existing data!
DROP TABLE IF EXISTS notes;

-- Create the new notes table with enhanced structure
CREATE TABLE notes (
    id BIGINT AUTO_INCREMENT PRIMARY KEY,
    title VARCHAR(255) NOT NULL,
    content TEXT NOT NULL,
    user VARCHAR(100) NOT NULL,
    is_public BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
);

-- Create indexes for better performance
CREATE INDEX idx_user ON notes (user);
CREATE INDEX idx_public ON notes (is_public);
CREATE INDEX idx_user_public ON notes (user, is_public);
CREATE INDEX idx_created_at ON notes (created_at);

-- Optional: Create sample data for testing
INSERT INTO notes (title, content, user, is_public) VALUES
('Welcome Note', 'This is a sample public note to test the API', 'admin', TRUE),
('Private Reminder', 'This is a private note that only the owner can see', 'admin', FALSE),
('Another Public Note', 'This is another public note for demonstration', 'admin', TRUE);

-- Verify the table structure
DESCRIBE notes;
