use std::env;
use tracing::{error, info, warn};
use tracing_subscriber::{
    EnvFilter,
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
};

/// Initialize logging for the application
pub fn init_logging() {
    let log_level = env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string());
    let is_production =
        env::var("RUST_ENV").unwrap_or_else(|_| "development".to_string()) == "production";

    // Create environment filter based on log level
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&log_level));

    // Initialize the subscriber
    if is_production {
        // Production: JSON format with essential fields only
        tracing_subscriber::registry()
            .with(env_filter)
            .with(
                fmt::layer()
                    .json()
                    .with_target(false)
                    .with_thread_ids(false)
                    .with_file(false)
                    .with_line_number(false)
                    .with_span_events(FmtSpan::CLOSE),
            )
            .init();
    } else {
        // Development: Human-readable format with more details
        tracing_subscriber::registry()
            .with(env_filter)
            .with(
                fmt::layer()
                    .pretty()
                    .with_target(true)
                    .with_thread_ids(false)
                    .with_file(true)
                    .with_line_number(true)
                    .with_span_events(FmtSpan::FULL),
            )
            .init();
    }

    info!(
        "Logging initialized - Environment: {}, Level: {}",
        if is_production {
            "production"
        } else {
            "development"
        },
        log_level
    );
}

/// Log authentication events securely
pub fn log_auth_success(username: &str) {
    info!("Authentication successful for user: {}", username);
}

pub fn log_auth_failure(username: &str, reason: &str) {
    warn!("Authentication failed for user '{}' - {}", username, reason);
}

pub fn log_auth_error(operation: &str, error: &str) {
    error!("Authentication error during {}: {}", operation, error);
}

/// Log configuration issues
pub fn log_config_error(config_var: &str, additional_info: &str) {
    error!(
        "Configuration error - {} not set or invalid: {}",
        config_var, additional_info
    );
}

/// Log database operations
pub fn log_db_operation(operation: &str, table: &str) {
    info!("Database operation: {} on {}", operation, table);
}

pub fn log_db_error(operation: &str, error: &str) {
    error!("Database error during {}: {}", operation, error);
}

/// Log API operations
pub fn log_api_request(method: &str, path: &str, status: u16) {
    info!("API request: {} {} -> {}", method, path, status);
}

pub fn log_api_error(method: &str, path: &str, error: &str) {
    warn!("API error: {} {} - {}", method, path, error);
}

/// Log security events (without exposing sensitive data)
pub fn log_security_event(event_type: &str, details: &str) {
    warn!("Security event - {}: {}", event_type, details);
}

/// Log application lifecycle events
pub fn log_app_startup(host: &str, port: u16) {
    info!("Application starting up on {}:{}", host, port);
}

pub fn log_app_shutdown() {
    info!("Application shutting down gracefully");
}

/// Log note operations
pub fn log_note_operation(operation: &str, user: &str) {
    info!("Note operation: {} by user {}", operation, user);
}

pub fn log_note_error(operation: &str, user: &str, error: &str) {
    warn!(
        "Note operation error: {} by user {} - {}",
        operation, user, error
    );
}

pub mod middleware;
