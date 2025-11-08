-- Database migration for Notepad API v2.0
-- This script updates the notes table to support titles, visibility, and timestamps

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
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,

    -- Indexes for better performance
    INDEX idx_user (user),
    INDEX idx_public (is_public),
    INDEX idx_user_public (user, is_public),
    INDEX idx_created_at (created_at)
);

-- Optional: Create sample data for testing
INSERT INTO notes (title, content, user, is_public) VALUES
('Welcome Note', 'This is a sample public note to test the API', 'admin', TRUE),
('Private Reminder', 'This is a private note that only the owner can see', 'admin', FALSE),
('Another Public Note', 'This is another public note for demonstration', 'admin', TRUE);

-- Verify the table structure
DESCRIBE notes;
```

Now let me update the API testing script to work with the new structure:
