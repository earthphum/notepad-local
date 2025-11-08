use axum::{
    extract::Request,
    http::{HeaderMap, HeaderName, StatusCode},
    middleware::Next,
    response::Response,
};
use std::time::Instant;
use tracing::{info, warn};
use uuid::Uuid;

/// Request logging middleware
pub async fn request_logging_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let start = Instant::now();
    let method = request.method().to_string();
    let path = request.uri().path().to_string();
    let request_id = Uuid::new_v4().to_string();

    // Extract user agent if available
    let user_agent = request
        .headers()
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown")
        .to_string();

    // Extract client IP
    let client_ip = extract_client_ip(&request.headers());

    info!(
        "Request started - ID: {}, Method: {}, Path: {}, IP: {}, User-Agent: {}",
        request_id, method, path, client_ip, user_agent
    );

    // Process the request
    let response = next.run(request).await;

    // Calculate duration
    let duration = start.elapsed();
    let status = response.status();

    // Log request completion
    if status.is_success() {
        info!(
            "Request completed - ID: {}, Method: {}, Path: {}, Status: {}, Duration: {:?}",
            request_id, method, path, status, duration
        );
    } else if status.is_client_error() {
        warn!(
            "Request failed (client error) - ID: {}, Method: {}, Path: {}, Status: {}, Duration: {:?}",
            request_id, method, path, status, duration
        );
    } else {
        warn!(
            "Request failed (server error) - ID: {}, Method: {}, Path: {}, Status: {}, Duration: {:?}",
            request_id, method, path, status, duration
        );
    }

    Ok(response)
}

/// Extract client IP from headers
fn extract_client_ip(headers: &HeaderMap) -> String {
    // Try various headers in order of preference
    let ip_headers = [
        "x-forwarded-for",
        "x-real-ip",
        "cf-connecting-ip",
        "x-client-ip",
        "x-forwarded",
        "forwarded-for",
        "forwarded",
    ];

    for header_name in ip_headers.iter().map(|s| HeaderName::from_static(s)) {
        if let Some(header_value) = headers.get(&header_name) {
            if let Ok(ip_str) = header_value.to_str() {
                // X-Forwarded-For can contain multiple IPs, take the first one
                let ip = ip_str.split(',').next().unwrap_or("").trim();
                if !ip.is_empty() {
                    return ip.to_string();
                }
            }
        }
    }

    "unknown".to_string()
}

/// Error logging middleware for handling internal server errors
pub async fn error_logging_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let method = request.method().to_string();
    let path = request.uri().path().to_string();

    // Note: Axum's next.run() returns Response, not Result
    // This middleware is mainly for logging since Axum handles errors elsewhere
    let response = next.run(request).await;

    if response.status().is_server_error() {
        warn!(
            "Internal server error - Method: {}, Path: {}, Status: {}",
            method,
            path,
            response.status()
        );
    }

    Ok(response)
}
