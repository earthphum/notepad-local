# Notepad Backend API v2.0

A secure, production-ready REST API for a note-taking service with public and private notes support, built with Rust and Axum.

## 🚀 New Features in v2.0

- **🔐 Enhanced Security**: Full JWT-based authentication with user-specific note access
- **🌍 Public & Private Notes**: Create notes that are either publicly accessible or private to the owner
- **📝 Rich Note Structure**: Each note now has a title, content, visibility settings, and timestamps
- **🔍 Smart Access Control**: Non-authenticated users can only view public notes
- **📊 User Statistics**: Get statistics about your notes (total, public, private counts)
- **🛡️ Production-Ready**: Structured logging, error handling, and security-first design

## 📋 API Endpoints

### Public Endpoints (No Authentication Required)

#### Get All Public Notes
```http
GET /contents
```

**Response:**
```json
[
  {
    "id": 1,
    "title": "Welcome Note",
    "content": "This is a sample public note",
    "user": "admin",
    "is_public": true,
    "created_at": "2024-01-20T10:30:45Z",
    "updated_at": "2024-01-20T10:30:45Z"
  }
]
```

#### Get Specific Public Note
```http
GET /contents/:id
```

**Response:**
```json
{
  "id": 1,
  "title": "Welcome Note",
  "content": "This is a sample public note",
  "user": "admin",
  "is_public": true,
  "created_at": "2024-01-20T10:30:45Z",
  "updated_at": "2024-01-20T10:30:45Z"
}
```

### Authentication

#### Login
```http
POST /login
Content-Type: application/json

{
  "username": "admin",
  "password": "your_password"
}
```

**Response:**
```json
{
  "token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9..."
}
```

### User Endpoints (Authentication Required)

All user endpoints require the JWT token in the `Authorization` header:
```
Authorization: Bearer <your_jwt_token>
```

#### Get All User Notes
```http
GET /admin/contents
Authorization: Bearer <jwt_token>
```

**Response:**
```json
[
  {
    "id": 1,
    "title": "My Public Note",
    "content": "This note is public",
    "user": "admin",
    "is_public": true,
    "created_at": "2024-01-20T10:30:45Z",
    "updated_at": "2024-01-20T10:30:45Z"
  },
  {
    "id": 2,
    "title": "My Private Note",
    "content": "This note is private",
    "user": "admin",
    "is_public": false,
    "created_at": "2024-01-20T10:35:22Z",
    "updated_at": "2024-01-20T10:35:22Z"
  }
]
```

#### Create Note
```http
POST /admin/contents
Authorization: Bearer <jwt_token>
Content-Type: application/json

{
  "title": "My New Note",
  "content": "This is the content of my note",
  "is_public": false
}
```

**Response:**
```json
{
  "message": "Note created successfully",
  "id": 3
}
```

#### Get Specific Note (User can access their own notes and any public notes)
```http
GET /admin/contents/:id
Authorization: Bearer <jwt_token>
```

#### Update Note
```http
PUT /admin/contents/:id
Authorization: Bearer <jwt_token>
Content-Type: application/json

{
  "title": "Updated Title",
  "content": "Updated content",
  "is_public": true
}
```

**Note:** All fields are optional in the update request.

**Response:**
```json
{
  "message": "Note updated successfully"
}
```

#### Delete Note
```http
DELETE /admin/contents/:id
Authorization: Bearer <jwt_token>
```

**Response:**
```json
{
  "message": "Note deleted successfully"
}
```

#### Get User Statistics
```http
GET /admin/stats
Authorization: Bearer <jwt_token>
```

**Response:**
```json
{
  "total_notes": 5,
  "public_notes": 3,
  "private_notes": 2,
  "user": "admin"
}
```

## 🗄️ Database Schema

The system uses MySQL with the following table structure:

```sql
CREATE TABLE notes (
    id BIGINT AUTO_INCREMENT PRIMARY KEY,
    title VARCHAR(255) NOT NULL,
    content TEXT NOT NULL,
    user VARCHAR(100) NOT NULL,
    is_public BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    
    INDEX idx_user (user),
    INDEX idx_public (is_public),
    INDEX idx_user_public (user, is_public),
    INDEX idx_created_at (created_at)
);
```

## 🛠️ Installation & Setup

### 1. Clone and Setup
```bash
git clone <repository-url>
cd notepad-local
```

### 2. Environment Configuration
```bash
# Copy environment template
cp .env.example .env
# Edit .env with your configuration
```

### 3. Database Setup
```bash
# Create database
mysql -u root -p
CREATE DATABASE notepad_dev;

# Run migration
mysql -u root -p notepad_dev < migrate.sql
```

### 4. Generate Password Hash
```bash
# Generate bcrypt hash for your admin password
cargo run --bin hash_generator

# Example output:
# Enter password to hash: your_secure_password
# ✅ Generated hash: $2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewdBPj/RK.s5uO8