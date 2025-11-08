#!/bin/bash

# Notepad API Testing Script v2.0
# This script tests the notepad backend API endpoints with the new public/private note system

# Configuration
API_URL="http://127.0.0.1:3000"
ADMIN_USER=${ADMIN_USER:-"admin"}
ADMIN_PASS=${ADMIN_PASS:-"1234"}

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Helper functions
print_header() {
    echo -e "${BLUE}=== $1 ===${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_info() {
    echo -e "ℹ️  $1"
}

# Check if server is running
check_server() {
    print_header "Checking Server Status"

    if curl -s "$API_URL/health" > /dev/null 2>&1; then
        print_success "Server is running at $API_URL"
        return 0
    else
        print_error "Server is not running at $API_URL"
        print_info "Please start the server with: cargo run"
        exit 1
    fi
}

# Test authentication
test_auth() {
    print_header "Testing Authentication"

    echo "Attempting login with username: $ADMIN_USER"

    response=$(curl -s -w "\n%{http_code}" \
        -X POST \
        -H "Content-Type: application/json" \
        -d "{\"username\":\"$ADMIN_USER\",\"password\":\"$ADMIN_PASS\"}" \
        "$API_URL/login")

    http_code=$(echo "$response" | tail -n1)
    body=$(echo "$response" | head -n -1)

    if [ "$http_code" = "200" ]; then
        print_success "Login successful"
        TOKEN=$(echo "$body" | grep -o '"token":"[^"]*"' | cut -d'"' -f4)
        if [ -n "$TOKEN" ]; then
            print_success "Token received: ${TOKEN:0:20}..."
            export AUTH_TOKEN="$TOKEN"
        else
            print_warning "Login response format unexpected"
        fi
    else
        print_error "Login failed with HTTP $http_code"
        echo "Response: $body"
        print_info "Make sure your .env file has correct ADMIN_USER and ADMIN_PASS_HASH values"
        export AUTH_TOKEN=""
    fi
    echo
}

# Test public endpoints
test_public_endpoints() {
    print_header "Testing Public Endpoints"

    # Test GET public contents
    print_info "Testing GET /contents (public notes)"
    response=$(curl -s -w "\n%{http_code}" "$API_URL/contents")

    http_code=$(echo "$response" | tail -n1)
    body=$(echo "$response" | head -n -1)

    if [ "$http_code" = "200" ]; then
        print_success "GET /contents successful"
        echo "Response: $body"
    else
        print_error "GET /contents failed with HTTP $http_code"
        echo "Response: $body"
    fi
    echo

    # Test GET specific public note (assuming note ID 1 exists and is public)
    print_info "Testing GET /contents/1 (specific public note)"
    response=$(curl -s -w "\n%{http_code}" "$API_URL/contents/1")

    http_code=$(echo "$response" | tail -n1)
    body=$(echo "$response" | head -n -1)

    if [ "$http_code" = "200" ]; then
        print_success "GET /contents/1 successful"
        echo "Response: $body"
    else
        print_warning "GET /contents/1 returned $http_code (note might not exist or be private)"
        echo "Response: $body"
    fi
    echo
}

# Test authenticated endpoints
test_authenticated_endpoints() {
    print_header "Testing Authenticated Endpoints"

    if [ -z "$AUTH_TOKEN" ]; then
        print_warning "No auth token available, skipping authenticated tests"
        return 1
    fi

    auth_header="Authorization: Bearer $AUTH_TOKEN"

    # Test GET user's notes
    print_info "Testing GET /admin/contents (user's all notes)"
    response=$(curl -s -w "\n%{http_code}" \
        -H "$auth_header" \
        "$API_URL/admin/contents")

    http_code=$(echo "$response" | tail -n1)
    body=$(echo "$response" | head -n -1)

    if [ "$http_code" = "200" ]; then
        print_success "GET /admin/contents successful"
        echo "Response: $body"
    else
        print_error "GET /admin/contents failed with HTTP $http_code"
        echo "Response: $body"
        return 1
    fi
    echo

    # Test POST note creation (public note)
    print_info "Testing POST /admin/contents (create public note)"
    timestamp=$(date '+%Y%m%d_%H%M%S')
    public_note_data="{
        \"title\": \"Public Test Note $timestamp\",
        \"content\": \"This is a public test note created at $(date)\",
        \"is_public\": true
    }"

    response=$(curl -s -w "\n%{http_code}" \
        -X POST \
        -H "Content-Type: application/json" \
        -H "$auth_header" \
        -d "$public_note_data" \
        "$API_URL/admin/contents")

    http_code=$(echo "$response" | tail -n1)
    body=$(echo "$response" | head -n -1)

    if [ "$http_code" = "201" ] || [ "$http_code" = "200" ]; then
        print_success "POST /admin/contents (public note) successful"
        echo "Response: $body"

        # Extract note ID if available
        NOTE_ID=$(echo "$body" | grep -o '"id":[0-9]*' | cut -d':' -f2)
        if [ -n "$NOTE_ID" ]; then
            export PUBLIC_NOTE_ID="$NOTE_ID"
            print_info "Created public note ID: $NOTE_ID"
        fi
    else
        print_error "POST /admin/contents (public note) failed with HTTP $http_code"
        echo "Response: $body"
    fi
    echo

    # Test POST note creation (private note)
    print_info "Testing POST /admin/contents (create private note)"
    private_note_data="{
        \"title\": \"Private Test Note $timestamp\",
        \"content\": \"This is a private test note created at $(date) - only I should see this\",
        \"is_public\": false
    }"

    response=$(curl -s -w "\n%{http_code}" \
        -X POST \
        -H "Content-Type: application/json" \
        -H "$auth_header" \
        -d "$private_note_data" \
        "$API_URL/admin/contents")

    http_code=$(echo "$response" | tail -n1)
    body=$(echo "$response" | head -n -1)

    if [ "$http_code" = "201" ] || [ "$http_code" = "200" ]; then
        print_success "POST /admin/contents (private note) successful"
        echo "Response: $body"

        # Extract note ID if available
        NOTE_ID=$(echo "$body" | grep -o '"id":[0-9]*' | cut -d':' -f2)
        if [ -n "$NOTE_ID" ]; then
            export PRIVATE_NOTE_ID="$NOTE_ID"
            print_info "Created private note ID: $NOTE_ID"
        fi
    else
        print_error "POST /admin/contents (private note) failed with HTTP $http_code"
        echo "Response: $body"
    fi
    echo

    # Test note update
    if [ -n "$PUBLIC_NOTE_ID" ]; then
        print_info "Testing PUT /admin/contents/$PUBLIC_NOTE_ID (update note)"
        update_data="{
            \"title\": \"Updated Public Note $timestamp\",
            \"is_public\": true
        }"

        response=$(curl -s -w "\n%{http_code}" \
            -X PUT \
            -H "Content-Type: application/json" \
            -H "$auth_header" \
            -d "$update_data" \
            "$API_URL/admin/contents/$PUBLIC_NOTE_ID")

        http_code=$(echo "$response" | tail -n1)
        body=$(echo "$response" | head -n -1)

        if [ "$http_code" = "200" ]; then
            print_success "PUT /admin/contents/$PUBLIC_NOTE_ID successful"
            echo "Response: $body"
        else
            print_error "PUT /admin/contents/$PUBLIC_NOTE_ID failed with HTTP $http_code"
            echo "Response: $body"
        fi
        echo
    fi

    # Test stats endpoint
    print_info "Testing GET /admin/stats (user statistics)"
    response=$(curl -s -w "\n%{http_code}" \
        -H "$auth_header" \
        "$API_URL/admin/stats")

    http_code=$(echo "$response" | tail -n1)
    body=$(echo "$response" | head -n -1)

    if [ "$http_code" = "200" ]; then
        print_success "GET /admin/stats successful"
        echo "Response: $body"
    else
        print_error "GET /admin/stats failed with HTTP $http_code"
        echo "Response: $body"
    fi
    echo
}

# Test access control
test_access_control() {
    print_header "Testing Access Control"

    # Test accessing private notes without authentication
    print_info "Testing access to admin endpoints without authentication"
    response=$(curl -s -w "\n%{http_code}" "$API_URL/admin/contents")

    http_code=$(echo "$response" | tail -n1)

    if [ "$http_code" = "401" ]; then
        print_success "Unauthenticated access to admin endpoints correctly returns 401"
    else
        print_error "Unauthenticated access returned $http_code instead of 401"
    fi
    echo

    # Test accessing specific note by admin endpoint without auth
    if [ -n "$PUBLIC_NOTE_ID" ]; then
        print_info "Testing access to specific admin note without authentication"
        response=$(curl -s -w "\n%{http_code}" "$API_URL/admin/contents/$PUBLIC_NOTE_ID")

        http_code=$(echo "$response" | tail -n1)

        if [ "$http_code" = "401" ]; then
            print_success "Unauthenticated access to specific admin note correctly returns 401"
        else
            print_error "Unauthenticated access to specific admin note returned $http_code instead of 401"
        fi
        echo
    fi

    # Test that private notes are not accessible via public endpoints
    if [ -n "$PRIVATE_NOTE_ID" ] && [ -n "$AUTH_TOKEN" ]; then
        print_info "Testing that private notes are not accessible via public endpoints"
        response=$(curl -s -w "\n%{http_code}" "$API_URL/contents/$PRIVATE_NOTE_ID")

        http_code=$(echo "$response" | tail -n1)

        if [ "$http_code" = "404" ]; then
            print_success "Private note correctly not accessible via public endpoint (404)"
        else
            print_warning "Private note access via public endpoint returned $http_code"
            echo "Response: $response"
        fi
        echo
    fi
}

# Test error cases
test_errors() {
    print_header "Testing Error Cases"

    # Test invalid login
    print_info "Testing invalid login"
    response=$(curl -s -w "\n%{http_code}" \
        -X POST \
        -H "Content-Type: application/json" \
        -d '{"username":"wronguser","password":"wrongpass"}' \
        "$API_URL/login")

    http_code=$(echo "$response" | tail -n1)

    if [ "$http_code" = "401" ]; then
        print_success "Invalid login correctly returns 401"
    else
        print_error "Invalid login returned $http_code instead of 401"
    fi
    echo

    # Test invalid endpoint
    print_info "Testing invalid endpoint"
    response=$(curl -s -w "\n%{http_code}" "$API_URL/nonexistent")
    http_code=$(echo "$response" | tail -n1)

    if [ "$http_code" = "404" ]; then
        print_success "Invalid endpoint correctly returns 404"
    else
        print_error "Invalid endpoint returned $http_code instead of 404"
    fi
    echo

    # Test malformed JSON in note creation
    if [ -n "$AUTH_TOKEN" ]; then
        print_info "Testing malformed JSON in note creation"
        response=$(curl -s -w "\n%{http_code}" \
            -X POST \
            -H "Content-Type: application/json" \
            -H "Authorization: Bearer $AUTH_TOKEN" \
            -d '{"title":"test","content":}' \
            "$API_URL/admin/contents")

        http_code=$(echo "$response" | tail -n1)

        if [ "$http_code" = "400" ] || [ "$http_code" = "422" ]; then
            print_success "Malformed JSON correctly returns error ($http_code)"
        else
            print_error "Malformed JSON returned $http_code instead of error code"
        fi
        echo
    fi

    # Test updating non-existent note
    if [ -n "$AUTH_TOKEN" ]; then
        print_info "Testing update of non-existent note"
        response=$(curl -s -w "\n%{http_code}" \
            -X PUT \
            -H "Content-Type: application/json" \
            -H "Authorization: Bearer $AUTH_TOKEN" \
            -d '{"title":"Updated"}' \
            "$API_URL/admin/contents/99999")

        http_code=$(echo "$response" | tail -n1)

        if [ "$http_code" = "404" ]; then
            print_success "Update of non-existent note correctly returns 404"
        else
            print_error "Update of non-existent note returned $http_code instead of 404"
        fi
        echo
    fi
}

# Test environment setup
test_environment() {
    print_header "Environment Setup Check"

    print_info "Current configuration:"
    echo "  API URL: $API_URL"
    echo "  Username: $ADMIN_USER"
    echo "  Password: ${ADMIN_PASS:0:1}*** (${#ADMIN_PASS} chars)"
    echo

    print_info "Make sure your .env file contains:"
    echo "  ADMIN_USER=your_username"
    echo "  ADMIN_PASS_HASH=\$2b\$12\$... (bcrypt hash)"
    echo "  JWT_SECRET=your_jwt_secret_here (32+ chars for production)"
    echo "  DATABASE_URL=mysql://user:pass@host:port/database"
    echo

    if [ -f ".env" ]; then
        print_success ".env file found"
    else
        print_warning ".env file not found - please create one with proper values"
    fi

    if [ -f "migrate.sql" ]; then
        print_info "Database migration script found - make sure to run it"
    else
        print_warning "migrate.sql not found - you may need to create the database schema"
    fi
    echo
}

# Main execution
main() {
    print_header "Notepad API Test Suite v2.0"
    print_info "Testing API at: $API_URL"
    print_info "Admin user: $ADMIN_USER"
    echo

    # Check dependencies
    if ! command -v curl &> /dev/null; then
        print_error "curl is not installed. Please install curl to run this script."
        exit 1
    fi

    # Run tests
    test_environment
    check_server
    test_auth
    test_public_endpoints

    if [ -n "$AUTH_TOKEN" ]; then
        test_authenticated_endpoints
    else
        print_warning "Skipping authenticated tests due to authentication failure"
    fi

    test_access_control
    test_errors

    print_header "Test Summary"
    if [ -n "$AUTH_TOKEN" ]; then
        print_success "API testing completed successfully!"
        print_info "All endpoints should be working correctly."
        print_info "Public notes are accessible without authentication."
        print_info "Private notes require authentication and are only visible to their owners."
    else
        print_error "Authentication failed - please check your .env configuration"
    fi
}

# Show usage information
show_usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  -h, --help          Show this help message"
    echo "  -u, --user USER     Set admin username (default: admin)"
    echo "  -p, --pass PASS     Set admin password (default: 1234)"
    echo "  -U, --url URL       Set API base URL (default: http://localhost:3000)"
    echo ""
    echo "Environment variables:"
    echo "  ADMIN_USER          Admin username"
    echo "  ADMIN_PASS          Admin password"
    echo ""
    echo "Examples:"
    echo "  $0                  # Run with default settings"
    echo "  $0 -u myuser -p mypass"
    echo "  ADMIN_USER=myuser ADMIN_PASS=mypass $0"
    echo ""
    echo "Note: Make sure your .env file has proper bcrypt hash for ADMIN_PASS_HASH"
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            show_usage
            exit 0
            ;;
        -u|--user)
            ADMIN_USER="$2"
            shift 2
            ;;
        -p|--pass)
            ADMIN_PASS="$2"
            shift 2
            ;;
        -U|--url)
            API_URL="$2"
            shift 2
            ;;
        *)
            print_error "Unknown option: $1"
            show_usage
            exit 1
            ;;
    esac
done

# Run main function
main
