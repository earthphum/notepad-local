# Notepad Backend API

A secure, production-ready REST API for a note-taking service built with Rust and Axum.

## 🚀 Features

- **Secure Authentication**: JWT-based authentication with bcrypt password hashing
- **Production Logging**: Structured logging with environment-aware configurations
- **Database Integration**: MySQL with async SQLx for type-safe database operations
- **Error Handling**: Comprehensive error handling without exposing sensitive information
- **Request Tracking**: Full request lifecycle logging with unique request IDs
- **Security-First**: No sensitive data exposure in production logs

## 📋 Requirements

- Rust 1.70+
- MySQL 8.0+
- Environment variables for configuration

## 🛠️ Installation

### 1. Clone the Repository
```bash
git clone <repository-url>
cd notepad-local
```

### 2. Set Up Environment

#### Development Environment
```bash
cp .env.development.example .env
# Edit .env with your local configuration
```

#### Production Environment
```bash
cp .env.production.example .env.production
# Edit .env.production with your production configuration
# Ensure proper file permissions: chmod 600 .env.production
```

### 3. Database Setup
```sql
-- Create database (adjust credentials as needed)
CREATE DATABASE notepad_dev;
```

### 4. Generate Password Hash
```bash
# Generate bcrypt hash for your admin password
cargo run --bin hash_generator

# Example output:
# Enter password to hash: your_secure_password
# ✅ Generated hash: $2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewdBPj/RK.s5uO8W
# Add this to your .env file as ADMIN_PASS_HASH=...
```

### 5. Install Dependencies
```bash
cargo build --release
```

## 🔧 Configuration

### Environment Variables

| Variable | Required | Description | Example |
|----------|----------|-------------|---------|
| `RUST_ENV` | Yes | Environment type | `production` or `development` |
| `LOG_LEVEL` | Yes | Logging level | `info`, `debug`, `warn`, `error` |
| `HOST` | No | Server host | `0.0.0.0` |
| `PORT` | No | Server port | `3000` |
| `DATABASE_URL` | Yes | MySQL connection string | `mysql://user:pass@host:3306/db` |
| `ADMIN_USER` | Yes | Admin username | `admin` |
| `ADMIN_PASS_HASH` | Yes | bcrypt hash of admin password | `$2b$12$...` |
| `JWT_SECRET` | Yes | Secret for JWT signing (32+ chars) | `your_secure_secret_here` |

### Security Considerations

#### Production Environment
- **JWT_SECRET**: Must be at least 32 characters long
- **Logging**: Structured JSON format, no sensitive data exposure
- **Error Messages**: Generic error messages for security
- **Request IDs**: Unique tracking for each request

#### Development Environment
- **JWT_SECRET**: Can be shorter for convenience
- **Logging**: Human-readable format with debug information
- **Detailed Errors**: More verbose error messages for debugging

## 🚀 Running the Application

### Development
```bash
# Run in development mode
cargo run

# Run with specific log level
RUST_LOG=debug cargo run
```

### Production
```bash
# Build optimized binary
cargo build --release

# Run production binary
RUST_ENV=production ./target/release/backend
```

### Using Docker (Recommended for Production)
```dockerfile
# Dockerfile example
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/backend /usr/local/bin/backend
EXPOSE 3000
CMD ["backend"]
```

## 📊 Logging

### Production Logging
In production, logs are structured as JSON for better log aggregation:

```json
{
  "timestamp": "2024-01-20T10:30:45.123456Z",
  "level": "info",
  "message": "Authentication successful for user: admin",
  "span": "login_request"
}
```

### Development Logging
In development, logs are human-readable with colors and formatting:

```
2024-01-20T10:30:45.123456Z  INFO notepad::auth: Authentication successful for user: admin
                                  at src/auth.rs:95
```

### Log Categories

| Category | Purpose | Production Safe |
|----------|---------|-----------------|
| Authentication | Login attempts, token generation | ✅ |
| Database | Connection, query operations | ✅ |
| API Requests | HTTP request lifecycle | ✅ |
| Security Events | Suspicious activities | ✅ |
| Errors | Application errors | ✅ (sanitized) |

## 🧪 API Endpoints

### Authentication
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

### Notes Operations

#### Get All Notes
```http
GET /notes
Authorization: Bearer <jwt_token>
```

**Response:**
```json
[
  {
    "id": 1,
    "user": "earth",
    "content": "My first note"
  }
]
```

#### Create Note
```http
POST /notes
Authorization: Bearer <jwt_token>
Content-Type: application/json

{
  "content": "This is my new note"
}
```

**Response:**
```json
{
  "message": "Note created"
}
```

## 🧪 Testing

### Unit Tests
```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture
```

### Integration Testing
Use the provided API testing script:

```bash
# Make script executable
chmod +x api.sh

# Run API tests
./api.sh

# Test with custom credentials
ADMIN_USER=myuser ADMIN_PASS=mypass ./api.sh
```

### Environment-Specific Testing
```bash
# Test production configuration
RUST_ENV=production cargo test

# Test development configuration
RUST_ENV=development cargo test
```

## 🔍 Monitoring and Observability

### Request Tracking
Every request gets a unique UUID for tracking:

```
Request started - ID: 550e8400-e29b-41d4-a716-446655440000, Method: POST, Path: /login, IP: 192.168.1.1, User-Agent: curl/7.68.0
Request completed - ID: 550e8400-e29b-41d4-a716-446655440000, Method: POST, Path: /login, Status: 200, Duration: 45.2ms
```

### Security Events
Security-related events are logged with appropriate severity:

```
Security event - failed_auth: Authentication failed for user 'admin' - invalid password
```

### Performance Metrics
Request duration is logged for performance monitoring:

```
Request completed - ID: 550e8400-e29b-41d4-a716-446655440000, Method: GET, Path: /notes, Status: 200, Duration: 12.3ms
```

## 🚨 Production Deployment Checklist

### Security
- [ ] Set strong JWT_SECRET (32+ characters)
- [ ] Use HTTPS in production
- [ ] Configure proper database credentials
- [ ] Set appropriate file permissions on .env files
- [ ] Review and harden database access controls
- [ ] Set up proper reverse proxy (nginx/caddy)

### Configuration
- [ ] Set `RUST_ENV=production`
- [ ] Configure appropriate `LOG_LEVEL` (usually `info` or `warn`)
- [ ] Set up log rotation
- [ ] Configure monitoring and alerting
- [ ] Set up database backups
- [ ] Configure health checks

### Performance
- [ ] Use release build (`cargo build --release`)
- [ ] Configure connection pooling
- [ ] Set appropriate timeouts
- [ ] Monitor memory usage
- [ ] Set up autoscaling if needed

### Logging & Monitoring
- [ ] Configure log aggregation (ELK stack, etc.)
- [ ] Set up metrics collection
- [ ] Configure alerting for errors
- [ ] Monitor database performance
- [ ] Set up uptime monitoring

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Ensure all tests pass
6. Update documentation
7. Submit a pull request

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 🆘 Troubleshooting

### Common Issues

#### Database Connection Failed
```
Error: Failed to connect to database: Access denied for user
```
**Solution**: Check your `DATABASE_URL` and ensure MySQL is running with correct credentials.

#### JWT Token Invalid
```
Error: Token generation failed
```
**Solution**: Ensure `JWT_SECRET` is set and at least 32 characters in production.

#### Password Verification Fails
```
Error: Authentication failed
```
**Solution**: Regenerate the password hash using `cargo run --bin hash_generator`.

### Debug Mode
Enable debug logging for troubleshooting:

```bash
RUST_ENV=development LOG_LEVEL=debug cargo run
```

### Log Analysis
Search for specific events in logs:

```bash
# Find authentication failures
grep "Authentication failed" app.log

# Find database errors
grep "Database error" app.log

# Find slow requests
grep "Duration.*ms" app.log | awk '$NF > 1000'
```

---

Built with ❤️ and Rust for secure, production-ready applications.