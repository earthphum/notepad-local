#!/bin/bash

# Notepad API Testing Script
# This script tests the notepad backend API endpoints

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

    if curl -s "$API_URL/login" > /dev/null 2>&1; then
        print_success "Server is running at $API_URL"
        return 0
    else
        print_error "Server is not running at $API_URL"
        print_info "Please start the server with: cargo run --bin backend"
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

# Test notes endpoints
test_notes() {
    print_header "Testing Notes API"

    if [ -z "$AUTH_TOKEN" ]; then
        print_warning "No auth token available, skipping notes tests"
        return 1
    fi

    auth_header="Authorization: Bearer $AUTH_TOKEN"

    # Test GET notes (should be empty initially)
    print_info "Testing GET /notes"
    response=$(curl -s -w "\n%{http_code}" \
        -H "$auth_header" \
        "$API_URL/notes")

    http_code=$(echo "$response" | tail -n1)
    body=$(echo "$response" | head -n -1)

    if [ "$http_code" = "200" ]; then
        print_success "GET /notes successful"
        echo "Response: $body"
    else
        print_error "GET /notes failed with HTTP $http_code"
        echo "Response: $body"
        return 1
    fi
    echo

    # Test POST note creation
    print_info "Testing POST /notes"
    test_note='{"content":"This is a test note created by api.sh at '$(date)'"}'

    response=$(curl -s -w "\n%{http_code}" \
        -X POST \
        -H "Content-Type: application/json" \
        -H "$auth_header" \
        -d "$test_note" \
        "$API_URL/notes")

    http_code=$(echo "$response" | tail -n1)
    body=$(echo "$response" | head -n -1)

    if [ "$http_code" = "200" ] || [ "$http_code" = "201" ]; then
        print_success "POST /notes successful"
        echo "Response: $body"

        # Extract note ID if available
        NOTE_ID=$(echo "$body" | grep -o '"id":[0-9]*' | cut -d':' -f2)
        if [ -n "$NOTE_ID" ]; then
            export TEST_NOTE_ID="$NOTE_ID"
            print_info "Created note ID: $NOTE_ID"
        fi
    else
        print_error "POST /notes failed with HTTP $http_code"
        echo "Response: $body"
        return 1
    fi
    echo

    # Test GET notes again (should have the new note)
    print_info "Testing GET /notes after creation"
    response=$(curl -s -w "\n%{http_code}" \
        -H "$auth_header" \
        "$API_URL/notes")

    http_code=$(echo "$response" | tail -n1)
    body=$(echo "$response" | head -n -1)

    if [ "$http_code" = "200" ]; then
        print_success "GET /notes successful after creation"
        echo "Response: $body"
    else
        print_error "GET /notes failed with HTTP $http_code"
        echo "Response: $body"
        return 1
    fi
    echo
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

    # Test notes without authentication
    print_info "Testing GET /notes without authentication"
    response=$(curl -s -w "\n%{http_code}" \
        "$API_URL/notes")

    http_code=$(echo "$response" | tail -n1)

    if [ "$http_code" = "401" ]; then
        print_success "Unauthenticated notes request correctly returns 401"
    else
        print_error "Unauthenticated notes request returned $http_code instead of 401"
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

    # Test malformed JSON
    print_info "Testing malformed JSON in login"
    response=$(curl -s -w "\n%{http_code}" \
        -X POST \
        -H "Content-Type: application/json" \
        -d '{"username":"test","password":}' \
        "$API_URL/login")

    http_code=$(echo "$response" | tail -n1)

    if [ "$http_code" = "400" ] || [ "$http_code" = "422" ]; then
        print_success "Malformed JSON correctly returns error ($http_code)"
    else
        print_error "Malformed JSON returned $http_code instead of error code"
    fi
    echo
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
    echo "  ADMIN_PASS_HASH=\$argon2id\$v=19\$m=65536,t=3,p=4\$..."
    echo "  JWT_SECRET=your_jwt_secret_here"
    echo

    if [ -f ".env" ]; then
        print_success ".env file found"
    else
        print_warning ".env file not found - please create one with proper values"
    fi
    echo
}

# Main execution
main() {
    print_header "Notepad API Test Suite"
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

    if [ -n "$AUTH_TOKEN" ]; then
        test_notes
    else
        print_warning "Skipping notes tests due to authentication failure"
    fi

    test_errors

    print_header "Test Summary"
    if [ -n "$AUTH_TOKEN" ]; then
        print_success "API testing completed successfully!"
        print_info "All endpoints should be working correctly."
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
    echo "  -p, --pass PASS     Set admin password (default: password)"
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
    echo "Note: Make sure your .env file has proper Argon2 hash for ADMIN_PASS_HASH"
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
