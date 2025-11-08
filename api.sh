#!/bin/bash

# Notepad API Testing Script
# This script tests the notepad backend API endpoints

# Configuration
API_URL="http://localhost:3000"
ADMIN_USER=${ADMIN_USER:-"admin"}
ADMIN_PASS=${ADMIN_PASS:-"password"}

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

    if curl -s "$API_URL" > /dev/null 2>&1; then
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
        export AUTH_TOKEN=""
    fi
    echo
}

# Test notes endpoints
test_notes() {
    print_header "Testing Notes API"

    if [ -z "$AUTH_TOKEN" ]; then
        print_warning "No auth token available, testing without authentication"
        auth_header=""
    else
        auth_header="Authorization: Bearer $AUTH_TOKEN"
    fi

    # Test GET notes (empty list)
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
    fi
    echo

    # Test POST note creation
    print_info "Testing POST /notes"
    test_note='{"title":"Test Note","content":"This is a test note created by api.sh"}'

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
    else
        print_error "POST /notes failed with HTTP $http_code"
        echo "Response: $body"
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
    fi
    echo
}

# Test static file serving
test_static() {
    print_header "Testing Static File Serving"

    # Test root path
    response=$(curl -s -w "\n%{http_code}" "$API_URL/")
    http_code=$(echo "$response" | tail -n1)

    if [ "$http_code" = "200" ]; then
        print_success "Static file serving working at /"
    else
        print_error "Static file serving failed at / with HTTP $http_code"
    fi

    # Test static directory
    response=$(curl -s -w "\n%{http_code}" "$API_URL/static/")
    http_code=$(echo "$response" | tail -n1)

    if [ "$http_code" = "200" ] || [ "$http_code" = "404" ]; then
        print_success "Static directory accessible"
    else
        print_warning "Static directory returned HTTP $http_code"
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
    check_server
    test_auth
    test_notes
    test_static
    test_errors

    print_header "Test Summary"
    print_success "API testing completed!"
    print_info "Review the output above for any errors or warnings."
}

# Show usage information
show_usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  -h, --help          Show this help message"
    echo "  -u, --user USER     Set admin username (default: admin)"
    echo "  -p, --pass PASS     Set admin password (default: password)"
    echo "  -u, --url URL       Set API base URL (default: http://localhost:3000)"
    echo ""
    echo "Environment variables:"
    echo "  ADMIN_USER          Admin username"
    echo "  ADMIN_PASS          Admin password"
    echo ""
    echo "Examples:"
    echo "  $0                  # Run with default settings"
    echo "  $0 -u myuser -p mypass"
    echo "  ADMIN_USER=myuser ADMIN_PASS=mypass $0"
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
        -u|--url)
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
