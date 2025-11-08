# Notepad API v2.0 Migration Guide

## Overview

The Notepad API has been successfully upgraded from v1.0 to v2.0 with significant enhancements to support both public and private notes with proper access control.

## Major Changes

### 🆕 New Features

#### Public vs Private Notes
- **Public Notes**: Accessible to anyone without authentication
- **Private Notes**: Only accessible to the authenticated owner
- **Visibility Control**: Each note has an `is_public` boolean field

#### Enhanced Note Structure
- **Title Field**: Every note now has a required title
- **Content Field**: Note content remains the primary text
- **Timestamps**: Automatic `created_at` and `updated_at` tracking
- **User Ownership**: Clear ownership through the `user` field

#### Authentication Improvements
- **JWT-based Access**: All user endpoints require valid JWT token
- **User Context**: Operations are performed in the context of the authenticated user
- **Token Extraction**: Proper JWT validation and user extraction

### 🔧 API Structure Changes

#### Endpoints

**Public Endpoints (No Authentication Required)**
- `GET /` - Root endpoint with API information
- `GET /health` - Health check
- `GET /contents` - List all public notes
- `GET /contents/:id` - Get specific public note
- `POST /login` - Authentication

**User Endpoints (Authentication Required)**
- `GET /admin/contents` - List user's notes (both public & private)
- `POST /admin/contents` - Create new note
- `GET /admin/contents/:id` - Get specific note (user's own or public)
- `PUT /admin/contents/:id` - Update note
- `DELETE /admin/contents/:id` - Delete note
- `GET /admin/stats` - User statistics

#### Request/Response Formats

**Create Note Request:**
```json
{
  "title": "Note Title",
  "content": "Note content",
  "is_public": true
}
```

**Update Note Request:**
```json
{
  "title": "Updated Title",      // Optional
  "content": "Updated content",   // Optional
  "is_public": false           // Optional
}
```

**Note Response:**
```json
{
  "id": 1,
  "title": "Note Title",
  "content": "Note content",
  "user": "admin",
  "is_public": true,
  "created_at": "2024-01-20T10:30:45Z",
  "updated_at": "2024-01-20T10:30:45Z"
}
```

### 🗄️ Database Schema

#### New Table Structure
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

### 📁 File Structure Changes

#### New Files Created
- `src/content.rs` - Core content management logic
- `src/models.rs` - Updated with new note structure
- `migrate.sql` - Database migration script
- `env.example` - Updated environment configuration

#### Modified Files
- `src/main.rs` - Updated routing and middleware
- `src/auth.rs` - Enhanced authentication
- `src/utils.rs` - Added JWT extraction functions
- `src/lib.rs` - Updated module declarations
- `api.sh` - Updated testing script
- `README.md` - Comprehensive documentation update

#### Removed Files
- `src/notes.rs` - Replaced by `src/content.rs`

## 🚀 Migration Steps

### 1. Database Migration
```bash
# Backup existing data (if needed)
mysqldump -u username -p notepad_dev > backup.sql

# Run migration script
mysql -u username -p notepad_dev < migrate.sql
```

### 2. Environment Configuration
```bash
# Copy and update environment file
cp env.example .env
# Edit .env with your configuration
```

### 3. Generate Password Hash
```bash
# Generate new hash if needed
cargo run --bin hash_generator
```

### 4. Build and Run
```bash
# Build the application
cargo build

# Run in development
cargo run

# Run in production
RUST_ENV=production cargo run --release
```

### 5. Test the API
```bash
# Make test script executable
chmod +x api.sh

# Run full test suite
./api.sh
```

## 🔒 Security Improvements

### Access Control
- Public endpoints are truly public (no auth required)
- Private operations require valid JWT token
- Users can only access their own notes
- Public notes are accessible to all authenticated users

### JWT Validation
- Proper token extraction from Authorization header
- Token validation against JWT_SECRET
- User context extraction from token claims
- Production-enforced secret length requirements

### Data Protection
- No sensitive data in logs (production mode)
- Structured logging with request tracking
- Error messages sanitized for production
- Request ID correlation for debugging

## 📊 New Statistics

The `/admin/stats` endpoint provides:
- Total notes count for the user
- Public notes count
- Private notes count
- User identification

## 🧪 Testing

### Updated Test Script
The `api.sh` script now tests:
- Public note access without authentication
- Authentication flow
- Note creation (public and private)
- Note updates and deletions
- Access control violations
- Statistics endpoint
- Error handling

### Access Control Tests
- Unauthenticated access to admin endpoints → 401
- Private note access via public endpoints → 404
- Cross-user note access attempts → 404
- Invalid JWT tokens → 401

## 🔄 Breaking Changes

### API Changes
- All note creation/update endpoints moved under `/admin/` prefix
- Note structure now includes `title` and `is_public` fields
- JWT token required for all user operations
- Response format includes new timestamp fields

### Database Changes
- Existing notes table structure completely changed
- Migration script drops and recreates the table
- All existing data will be lost unless migrated manually

### Configuration Changes
- Environment variables remain the same
- JWT_SECRET must be 32+ characters in production
- Database URL must point to the updated schema

## 📈 Performance Considerations

### Database Indexes
- `idx_user` - Optimizes user-specific queries
- `idx_public` - Optimizes public note filtering
- `idx_user_public` - Optimizes combined user/visibility queries
- `idx_created_at` - Optimizes sorting by creation time

### Query Optimizations
- Public notes query: `WHERE is_public = true`
- User notes query: `WHERE user = ?`
- Access control: `WHERE id = ? AND (user = ? OR is_public = true)`

## 🛠️ Development Notes

### Code Organization
- Separation of public vs private endpoint logic
- Centralized authentication checks in route handlers
- Consistent error handling and logging
- Type-safe database operations with SQLx

### Testing Strategy
- Unit tests for all core functions
- Integration tests via API script
- Environment-specific testing (dev/prod)
- Access control validation

## 📋 Checklist for Deployment

- [ ] Run database migration
- [ ] Update environment configuration
- [ ] Set strong JWT_SECRET (32+ chars)
- [ ] Configure reverse proxy with HTTPS
- [ ] Set up log rotation
- [ ] Monitor database performance
- [ ] Test all endpoints
- [ ] Verify access control
- [ ] Update any client applications

## 🆘 Troubleshooting

### Common Issues
1. **Database Connection**: Check DATABASE_URL in .env
2. **Authentication**: Verify JWT_SECRET and ADMIN_PASS_HASH
3. **Migration Errors**: Ensure MySQL is running with proper permissions
4. **Build Issues**: Run `cargo clean && cargo build`

### Debug Mode
```bash
RUST_ENV=development LOG_LEVEL=debug cargo run
```

### Log Analysis
```bash
# Find authentication failures
grep "Authentication failed" app.log

# Find database errors
grep "Database error" app.log

# Find slow requests
grep "Duration.*ms" app.log
```

## 🎯 Summary

The v2.0 migration transforms the Notepad API from a basic note-taking service into a production-ready content management platform with proper access control, rich note structure, and comprehensive security features. The migration maintains backward compatibility in configuration while introducing powerful new capabilities for public and private note management.
